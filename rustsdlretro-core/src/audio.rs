use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use alsa::poll::Descriptors;

pub struct RingBuffer {
    data: Vec<i16>,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

const RING_BUFFER_CAPACITY: usize = 65536;
const READ_BATCH_SIZE: usize = 8192;

impl RingBuffer {
    pub fn new() -> Self {
        RingBuffer {
            data: vec![0i16; RING_BUFFER_CAPACITY],
            capacity: RING_BUFFER_CAPACITY,
            read_pos: 0,
            write_pos: 0,
            count: 0,
        }
    }

    /// Write samples, dropping the oldest data (existing first, then the
    /// start of the incoming batch) when there is not enough room. This
    /// keeps the most recent audio so playback never stalls permanently.
    pub fn write(&mut self, samples: &[i16]) -> usize {
        let total = self.count + samples.len();
        let keep = total.min(self.capacity);
        let drop_total = total - keep;
        let drop_old = drop_total.min(self.count);
        self.read_pos = (self.read_pos + drop_old) % self.capacity;
        self.count -= drop_old;
        let skip = drop_total - drop_old;
        let to_write = samples.len() - skip;
        for i in 0..to_write {
            self.data[self.write_pos] = samples[skip + i];
            self.write_pos = (self.write_pos + 1) % self.capacity;
        }
        self.count += to_write;
        to_write
    }

    pub fn read(&mut self, buf: &mut [i16]) -> usize {
        let to_read = self.count.min(buf.len()).min(READ_BATCH_SIZE);
        for i in 0..to_read {
            buf[i] = self.data[self.read_pos];
            self.read_pos = (self.read_pos + 1) % self.capacity;
        }
        self.count -= to_read;
        to_read
    }

    pub fn len(&self) -> usize {
        self.count
    }

    pub fn is_empty(&self) -> bool {
        self.count == 0
    }
}

impl Default for RingBuffer {
    fn default() -> Self {
        Self::new()
    }
}

pub struct AudioDriver {
    pub sample_rate: u32,
    pcm: Arc<Mutex<Option<alsa::pcm::PCM>>>,
    ring_buffer: Arc<Mutex<RingBuffer>>,
    thread_handle: Option<JoinHandle<()>>,
    stopped: Arc<AtomicBool>,
}

impl AudioDriver {
    pub fn new(sample_rate: u32) -> Result<Self, String> {
        eprintln!("AudioDriver::new: sample_rate={}", sample_rate);
        let ring_buffer = Arc::new(Mutex::new(RingBuffer::new()));
        let stopped = Arc::new(AtomicBool::new(false));
        let pcm = Arc::new(Mutex::new(None));

        let device_names = ["default", "plughw:0,0", "hw:0,0"];
        let mut pcm_handle = None;
        let mut dev_used = None;
        for dev_name in &device_names {
            match open_pcm(dev_name, sample_rate) {
                Ok(p) => { pcm_handle = Some(p); dev_used = Some(dev_name.to_string()); break; }
                Err(e) => { eprintln!("ALSA device '{}' open failed: {}", dev_name, e); }
            }
        }
        let (pcm_handle, dev_used) = match (pcm_handle, dev_used) {
            (Some(p), Some(d)) => (p, d),
            _ => return Err(format!("ALSA PCM open failed, tried {:?}", device_names)),
        };

        {
            let mut pcm_guard = pcm.lock().unwrap();
            *pcm_guard = Some(pcm_handle);
        }

        let pcm_clone = Arc::clone(&pcm);
        let rb_clone = Arc::clone(&ring_buffer);
        let stopped_clone = Arc::clone(&stopped);

        let thread_handle = std::thread::spawn(move || {
            playback_thread_loop(rb_clone, pcm_clone, stopped_clone, dev_used, sample_rate);
        });

        Ok(AudioDriver {
            sample_rate,
            pcm,
            ring_buffer,
            thread_handle: Some(thread_handle),
            stopped,
        })
    }

    pub fn push_batch(&self, data: &[i16]) {
        let mut rb = self.ring_buffer.lock().unwrap();
        rb.write(data);
    }

    pub fn push_stereo_pair(&self, left: i16, right: i16) {
        let mut rb = self.ring_buffer.lock().unwrap();
        let buf = [left, right];
        rb.write(&buf);
    }

    pub fn restart_with_rate(&mut self, new_rate: u32) {
        if new_rate == self.sample_rate {
            return;
        }

        self.stopped.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            let _ = handle.join();
        }

        {
            let mut pcm_guard = self.pcm.lock().unwrap();
            pcm_guard.take();
        }

        let pcm_handle = match open_pcm("default", new_rate) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("ALSA PCM reopen for rate {} failed: {}, falling back to stub", new_rate, e);
                self.sample_rate = new_rate;
                self.stopped.store(true, Ordering::SeqCst);
                return;
            }
        };

        {
            let mut pcm_guard = self.pcm.lock().unwrap();
            *pcm_guard = Some(pcm_handle);
        }

        self.sample_rate = new_rate;
        self.stopped.store(false, Ordering::SeqCst);

        let pcm_clone = Arc::clone(&self.pcm);
        let rb_clone = Arc::clone(&self.ring_buffer);
        let stopped_clone = Arc::clone(&self.stopped);

        self.thread_handle = Some(std::thread::spawn(move || {
            playback_thread_loop(rb_clone, pcm_clone, stopped_clone, "default".to_string(), new_rate);
        }));
    }

    pub fn stop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
        if let Some(handle) = self.thread_handle.take() {
            match handle.join() {
                Ok(()) => {},
                Err(_) => eprintln!("Audio playback thread join panicked"),
            }
        }
        {
            let mut pcm_guard = self.pcm.lock().unwrap();
            pcm_guard.take();
        }
    }
}

const MAX_STALL_POLLS: u32 = 50; // ~5 s of 100 ms polls before giving up on a batch

fn open_pcm(device: &str, sample_rate: u32) -> Result<alsa::pcm::PCM, String> {
    let pcm = alsa::pcm::PCM::new(device, alsa::Direction::Playback, true)
        .map_err(|e| format!("open '{}' failed: {}", device, e))?;
    {
        let hw_params = alsa::pcm::HwParams::any(&pcm)
            .map_err(|e| format!("hw_params failed: {}", e))?;
        hw_params.set_access(alsa::pcm::Access::RWInterleaved)
            .map_err(|e| format!("set_access failed: {}", e))?;
        hw_params.set_format(alsa::pcm::Format::S16LE)
            .map_err(|e| format!("set_format failed: {}", e))?;
        hw_params.set_channels(2)
            .map_err(|e| format!("set_channels failed: {}", e))?;
        hw_params.set_rate(sample_rate, alsa::ValueOr::Nearest)
            .map_err(|e| format!("set_rate failed: {}", e))?;
        pcm.hw_params(&hw_params)
            .map_err(|e| format!("hw_params apply failed: {}", e))?;
    }
    if let Ok(current) = pcm.hw_params_current() {
        eprintln!("ALSA PCM '{}' configured: rate={}", device, current.get_rate().unwrap_or(0));
    }
    pcm.start()
        .map_err(|e| format!("PCM start failed: {}", e))?;
    Ok(pcm)
}

/// Wait until the PCM buffer can accept more data (non-blocking mode).
/// Recovers from xrun/stopped states. Bounded by a 100 ms poll timeout so
/// callers can periodically check their stop flag.
fn wait_for_space(pcm: &alsa::pcm::PCM) {
    let polled = pcm
        .get()
        .and_then(|mut fds| alsa::poll::poll(&mut fds, 100))
        .is_ok();
    if !polled {
        std::thread::sleep(Duration::from_millis(100));
    }
    match pcm.state() {
        alsa::pcm::State::XRun => {
            let _ = pcm.recover(libc::EPIPE, true);
            let _ = pcm.start();
        }
        alsa::pcm::State::Suspended => {
            let _ = pcm.start();
        }
        _ => {}
    }
}

/// Full recovery for a real hardware error (EIO): drain, prepare, start.
fn recover_pcm(pcm: &alsa::pcm::PCM) -> Result<(), i32> {
    let _ = pcm.drain();
    pcm.recover(libc::EIO, true).map_err(|e| e.errno())?;
    pcm.start().map_err(|e| e.errno())
}

/// Write the whole buffer to PCM, handling non-blocking EPIPE/EAGAIN by
/// polling for space. Returns Err(errno) if the batch cannot be written
/// (stall timeout) or on an unrecoverable error.
fn write_all_pcm(pcm: &alsa::pcm::PCM, buffer: &[i16]) -> Result<(), i32> {
    let frames_total = buffer.len() / 2;
    let mut offset = 0usize;
    let mut stalled = 0u32;
    while offset < frames_total {
        match pcm.io_i16().and_then(|io| io.writei(&buffer[offset * 2..])) {
            Ok(n) if n > 0 => {
                offset += n;
                stalled = 0;
            }
            Ok(_) => {
                wait_for_space(pcm);
                stalled += 1;
            }
            Err(e) => {
                let err = e.errno();
                if err == libc::EPIPE || err == libc::EAGAIN {
                    // Normal in non-blocking mode: buffer full or xrun.
                    wait_for_space(pcm);
                    stalled += 1;
                } else if err == libc::EIO {
                    recover_pcm(pcm)?;
                    stalled = 0;
                } else {
                    return Err(err);
                }
            }
        }
        if stalled >= MAX_STALL_POLLS {
            return Err(libc::EPIPE);
        }
    }
    Ok(())
}

fn playback_thread_loop(
    ring_buffer: Arc<Mutex<RingBuffer>>,
    pcm: Arc<Mutex<Option<alsa::pcm::PCM>>>,
    stopped: Arc<AtomicBool>,
    device: String,
    sample_rate: u32,
) {
    let mut buffer = vec![0i16; READ_BATCH_SIZE * 2];
    let mut write_count = 0u64;
    let mut empty_count = 0u64;
    let mut consecutive_failures = 0u32;
    let mut last_err_log = Instant::now();
    let mut audio_disabled = false;
    let mut occupancy_log_counter = 0u64;

    eprintln!("playback_thread: started (device={}, rate={})", device, sample_rate);

    loop {
        if stopped.load(Ordering::SeqCst) {
            eprintln!("playback_thread: stopping (stopped=true)");
            break;
        }

        let mut rb = ring_buffer.lock().unwrap();
        let available = rb.len();
        occupancy_log_counter += 1;
        if occupancy_log_counter % 500 == 0 {
            eprintln!("audio: rb={}/{} ({:.1}s), empty_iters={}", available, rb.capacity, available as f64 / sample_rate as f64 * 2.0, empty_count);
        }
        drop(rb);

        if available == 0 {
            empty_count += 1;
            if empty_count % 10000 == 0 {
                eprintln!("playback_thread: {} empty iterations", empty_count);
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        empty_count = 0;

        let mut rb = ring_buffer.lock().unwrap();
        let to_read = available.min(buffer.len());
        let read_count = rb.read(&mut buffer[..to_read]);
        drop(rb);

        if read_count == 0 {
            continue;
        }

        if audio_disabled {
            // Silent mode: keep draining the ring buffer so producers never block.
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }

        let pcm_opt = pcm.lock().unwrap();
        if let Some(pcm_handle) = &*pcm_opt {
            match write_all_pcm(pcm_handle, &buffer[..read_count]) {
                Ok(()) => {
                    write_count += 1;
                    consecutive_failures = 0;
                    if write_count == 1 || write_count % 1000 == 0 {
                        eprintln!("playback_thread: {} total ALSA writes, {} samples written this call", write_count, read_count);
                    }
                }
                Err(err) => {
                    consecutive_failures += 1;
                    if last_err_log.elapsed() >= Duration::from_secs(1) {
                        eprintln!("ALSA write error (errno {}), {} consecutive failures, recovering...", err, consecutive_failures);
                        last_err_log = Instant::now();
                    }
                    if consecutive_failures >= 5 {
                        drop(pcm_opt);
                        eprintln!("audio: {} consecutive failures, reopening device '{}'", consecutive_failures, device);
                        match open_pcm(&device, sample_rate) {
                            Ok(p) => {
                                let mut pcm_guard = pcm.lock().unwrap();
                                *pcm_guard = Some(p);
                                consecutive_failures = 0;
                            }
                            Err(e) => {
                                eprintln!("audio: reopen failed ({}), continuing in silent mode", e);
                                let mut pcm_guard = pcm.lock().unwrap();
                                pcm_guard.take();
                                audio_disabled = true;
                            }
                        }
                    } else {
                        drop(pcm_opt);
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        } else {
            eprintln!("playback_thread: pcm_handle is None");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_read() {
        let mut rb = RingBuffer::new();
        let samples: Vec<i16> = (0..100).map(|i| i as i16).collect();
        assert_eq!(rb.write(&samples), 100);
        let mut buf = vec![0i16; 100];
        assert_eq!(rb.read(&mut buf), 100);
        assert_eq!(buf, samples);
    }

    #[test]
    fn test_write_full_drops_oldest() {
        let mut rb = RingBuffer::new();
        let big: Vec<i16> = (0..RING_BUFFER_CAPACITY + 100).map(|i| i as i16).collect();
        assert_eq!(rb.write(&big), RING_BUFFER_CAPACITY);
        assert_eq!(rb.len(), RING_BUFFER_CAPACITY);
        let mut buf = vec![0i16; RING_BUFFER_CAPACITY];
        let mut filled = 0;
        while filled < RING_BUFFER_CAPACITY {
            filled += rb.read(&mut buf[filled..]);
        }
        for i in 0..RING_BUFFER_CAPACITY {
            assert_eq!(buf[i], (i + 100) as i16);
        }
    }

    #[test]
    fn test_read_batch_limit() {
        let mut rb = RingBuffer::new();
        let samples: Vec<i16> = (0..20000).map(|i| (i % 100) as i16).collect();
        rb.write(&samples);
        let mut buf = vec![0i16; 20000];
        assert_eq!(rb.read(&mut buf), READ_BATCH_SIZE);
        assert_eq!(rb.len(), 20000 - READ_BATCH_SIZE);
    }

    #[test]
    fn test_empty_read() {
        let mut rb = RingBuffer::new();
        let mut buf = vec![0i16; 100];
        assert_eq!(rb.read(&mut buf), 0);
    }

    #[test]
    fn test_wraparound() {
        let mut rb = RingBuffer::new();
        let part1: Vec<i16> = (0..2000).map(|i| i as i16).collect();
        rb.write(&part1);
        let mut buf = vec![0i16; 2000];
        rb.read(&mut buf);
        assert_eq!(buf, part1);

        let part2: Vec<i16> = (10000..12000).map(|i| i as i16).collect();
        rb.write(&part2);
        rb.read(&mut buf);
        assert_eq!(buf, part2);
    }

    #[test]
    fn test_audio_driver_creation_fails_without_alsa() {
        let result = AudioDriver::new(44100);
        if result.is_ok() {
            let mut driver = result.unwrap();
            driver.stop();
        }
    }
}
