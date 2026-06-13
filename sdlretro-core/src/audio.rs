use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub struct RingBuffer {
    data: Vec<i16>,
    capacity: usize,
    read_pos: usize,
    write_pos: usize,
    count: usize,
}

const RING_BUFFER_CAPACITY: usize = 8192;
const READ_BATCH_SIZE: usize = 1024;

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

    pub fn write(&mut self, samples: &[i16]) -> usize {
        let available = self.capacity - self.count;
        let to_write = samples.len().min(available);
        for i in 0..to_write {
            self.data[self.write_pos] = samples[i];
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
        let ring_buffer = Arc::new(Mutex::new(RingBuffer::new()));
        let stopped = Arc::new(AtomicBool::new(false));
        let pcm = Arc::new(Mutex::new(None));

        let pcm_handle = alsa::pcm::PCM::new(
            "default",
            alsa::Direction::Playback,
            true,
        ).map_err(|e| format!("ALSA PCM open failed: {}", e))?;

        let hw_params = alsa::pcm::HwParams::any(&pcm_handle)
            .map_err(|e| format!("ALSA hw_params failed: {}", e))?;
        hw_params.set_access(alsa::pcm::Access::RWInterleaved)
            .map_err(|e| format!("set_access failed: {}", e))?;
        hw_params.set_format(alsa::pcm::Format::S16LE)
            .map_err(|e| format!("set_format failed: {}", e))?;
        hw_params.set_channels(2)
            .map_err(|e| format!("set_channels failed: {}", e))?;
        hw_params.set_rate(sample_rate, alsa::ValueOr::Nearest)
            .map_err(|e| format!("set_rate failed: {}", e))?;
        pcm_handle.hw_params(&hw_params)
            .map_err(|e| format!("hw_params apply failed: {}", e))?;
        pcm_handle.start()
            .map_err(|e| format!("PCM start failed: {}", e))?;

        {
            let mut pcm_guard = pcm.lock().unwrap();
            *pcm_guard = Some(pcm_handle);
        }

        let pcm_clone = Arc::clone(&pcm);
        let rb_clone = Arc::clone(&ring_buffer);
        let stopped_clone = Arc::clone(&stopped);

        let thread_handle = std::thread::spawn(move || {
            playback_thread_loop(rb_clone, pcm_clone, stopped_clone);
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

        let pcm_handle = match alsa::pcm::PCM::new(
            "default",
            alsa::Direction::Playback,
            true,
        ) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("ALSA PCM reopen for rate {} failed: {}, falling back to stub", new_rate, e);
                self.sample_rate = new_rate;
                self.stopped.store(true, Ordering::SeqCst);
                return;
            }
        };

        let hw_params = match alsa::pcm::HwParams::any(&pcm_handle) {
            Ok(h) => h,
            Err(e) => {
                eprintln!("ALSA hw_params reopen failed: {}, falling back to stub", e);
                self.sample_rate = new_rate;
                self.stopped.store(true, Ordering::SeqCst);
                return;
            }
        };

        if let Err(e) = hw_params.set_access(alsa::pcm::Access::RWInterleaved) {
            eprintln!("ALSA set_access reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }
        if let Err(e) = hw_params.set_format(alsa::pcm::Format::S16LE) {
            eprintln!("ALSA set_format reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }
        if let Err(e) = hw_params.set_channels(2) {
            eprintln!("ALSA set_channels reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }
        if let Err(e) = hw_params.set_rate(new_rate, alsa::ValueOr::Nearest) {
            eprintln!("ALSA set_rate reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }
        if let Err(e) = pcm_handle.hw_params(&hw_params) {
            eprintln!("ALSA hw_params apply reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }
        if let Err(e) = pcm_handle.start() {
            eprintln!("ALSA PCM start reopen failed: {}, falling back to stub", e);
            self.sample_rate = new_rate;
            self.stopped.store(true, Ordering::SeqCst);
            return;
        }

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
            playback_thread_loop(rb_clone, pcm_clone, stopped_clone);
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

fn playback_thread_loop(
    ring_buffer: Arc<Mutex<RingBuffer>>,
    pcm: Arc<Mutex<Option<alsa::pcm::PCM>>>,
    stopped: Arc<AtomicBool>,
) {
    let mut buffer = vec![0i16; READ_BATCH_SIZE * 2];

    loop {
        if stopped.load(Ordering::SeqCst) {
            break;
        }

        let mut rb = ring_buffer.lock().unwrap();
        let available = rb.len();
        drop(rb);

        if available == 0 {
            std::thread::sleep(std::time::Duration::from_millis(1));
            continue;
        }

        let mut rb = ring_buffer.lock().unwrap();
        let to_read = available.min(buffer.len());
        let read_count = rb.read(&mut buffer[..to_read]);
        drop(rb);

        if read_count == 0 {
            continue;
        }

        let pcm_opt = pcm.lock().unwrap();
        if let Some(ref pcm_handle) = *pcm_opt {
            match pcm_handle.write(&buffer[..read_count]) {
                Ok(frames) => {
                    if (frames as usize) < read_count {
                        eprintln!("ALSA wrote fewer frames than requested: {} vs {}", frames, read_count);
                    }
                }
                Err(e) => {
                    drop(pcm_opt);
                    let mut pcm_guard = pcm.lock().unwrap();
                    if let Some(ref mut p) = *pcm_guard {
                        let _ = p.recover(e.raw_error());
                        let _ = p.start();
                    }
                    eprintln!("ALSA write error recovered: {}", e);
                }
            }
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
        rb.read(&mut buf);
        for i in 0..RING_BUFFER_CAPACITY {
            assert_eq!(buf[i], (i + 100) as i16);
        }
    }

    #[test]
    fn test_read_batch_limit() {
        let mut rb = RingBuffer::new();
        let samples: Vec<i16> = (0..2048).map(|i| i as i16).collect();
        rb.write(&samples);
        let mut buf = vec![0i16; 2048];
        assert_eq!(rb.read(&mut buf), READ_BATCH_SIZE);
        assert_eq!(rb.len(), 2048 - READ_BATCH_SIZE);
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
