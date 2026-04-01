use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Shared state for throughput measurement across threads.
pub struct ThroughputState {
    pub total_bytes: Arc<AtomicU64>,
    pub bytes_at_slow_start: Arc<AtomicU64>,
    pub slow_start_recorded: Arc<std::sync::atomic::AtomicBool>,
    pub start: Instant,
    pub slow_start: Duration,
    pub peak_bps: Arc<std::sync::Mutex<f64>>,
}

impl ThroughputState {
    pub fn new(slow_start: Duration) -> Self {
        Self {
            total_bytes: Arc::new(AtomicU64::new(0)),
            bytes_at_slow_start: Arc::new(AtomicU64::new(0)),
            slow_start_recorded: Arc::new(std::sync::atomic::AtomicBool::new(false)),
            start: Instant::now(),
            slow_start,
            peak_bps: Arc::new(std::sync::Mutex::new(0.0)),
        }
    }

    pub fn record_tick(&self, instant_bps: f64) {
        let mut peak = self.peak_bps.lock().unwrap();
        if instant_bps > *peak {
            *peak = instant_bps;
        }

        // Record slow start boundary
        if !self.slow_start_recorded.load(Ordering::Relaxed)
            && self.start.elapsed() >= self.slow_start
        {
            self.bytes_at_slow_start
                .store(self.total_bytes.load(Ordering::Relaxed), Ordering::Relaxed);
            self.slow_start_recorded
                .store(true, Ordering::Relaxed);
        }
    }

    pub fn total_bytes(&self) -> u64 {
        self.total_bytes.load(Ordering::Relaxed)
    }

    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    pub fn peak(&self) -> f64 {
        *self.peak_bps.lock().unwrap()
    }

    pub fn average_including_slow_start(&self) -> f64 {
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed > 0.0 {
            (self.total_bytes() as f64 * 8.0) / elapsed
        } else {
            0.0
        }
    }

    pub fn average_excluding_slow_start(&self) -> f64 {
        let ss = self.slow_start.as_secs_f64();
        let elapsed = self.elapsed().as_secs_f64();
        if elapsed <= ss {
            return self.average_including_slow_start();
        }

        let bytes_at_ss = self.bytes_at_slow_start.load(Ordering::Relaxed);
        let total = self.total_bytes();
        let bytes_after = total.saturating_sub(bytes_at_ss);
        let duration_after = elapsed - ss;

        if duration_after > 0.0 {
            (bytes_after as f64 * 8.0) / duration_after
        } else {
            0.0
        }
    }

    pub fn duration_secs(&self) -> f64 {
        self.elapsed().as_secs_f64()
    }
}

/// Tracks latency samples.
pub struct LatencyTracker {
    samples: Vec<Duration>,
}

impl LatencyTracker {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn record(&mut self, rtt: Duration) {
        self.samples.push(rtt);
    }

    pub fn count(&self) -> usize {
        self.samples.len()
    }

    pub fn min_ms(&self) -> f64 {
        self.samples
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .fold(f64::MAX, f64::min)
    }

    pub fn max_ms(&self) -> f64 {
        self.samples
            .iter()
            .map(|d| d.as_secs_f64() * 1000.0)
            .fold(0.0f64, f64::max)
    }

    pub fn avg_ms(&self) -> f64 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f64 = self.samples.iter().map(|d| d.as_secs_f64() * 1000.0).sum();
        sum / self.samples.len() as f64
    }

    pub fn jitter_ms(&self) -> f64 {
        if self.samples.len() < 2 {
            return 0.0;
        }
        let diffs: Vec<f64> = self
            .samples
            .windows(2)
            .map(|w| {
                let a = w[0].as_secs_f64() * 1000.0;
                let b = w[1].as_secs_f64() * 1000.0;
                (a - b).abs()
            })
            .collect();
        diffs.iter().sum::<f64>() / diffs.len() as f64
    }
}
