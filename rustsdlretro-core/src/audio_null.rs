//! Null audio driver - silently discards all audio samples.
//! Use this when no audio hardware is available (testing, headless environments).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct NullAudioDriver {
    pub sample_rate: u32,
    stopped: Arc<AtomicBool>,
}

impl NullAudioDriver {
    pub fn new(sample_rate: u32) -> Self {
        eprintln!("NullAudioDriver: audio will be silently discarded");
        NullAudioDriver {
            sample_rate,
            stopped: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn push_batch(&self, _data: &[i16]) {
        // Silently discard all audio samples
    }

    pub fn push_stereo_pair(&self, _left: i16, _right: i16) {
        // Silently discard audio
    }

    pub fn stop(&mut self) {
        self.stopped.store(true, Ordering::SeqCst);
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}
