use std::io::Write;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Instant;

pub fn short_host(url: &str) -> &str {
    let s = url.strip_prefix("wss://").unwrap_or(url);
    s.split('.').next().unwrap_or(s)
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    }
}

fn format_speed(mbps: f64) -> String {
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{:.2} Mbps", mbps)
    }
}

fn bar(val: f64, max: f64, width: usize) -> String {
    let frac = if max > 0.0 { (val / max).min(1.0) } else { 0.0 };
    let filled = (frac * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);
    format!(
        "\x1b[36m{}\x1b[90m{}\x1b[0m",
        "█".repeat(filled),
        "░".repeat(empty)
    )
}

pub struct TickState {
    server_labels: Vec<String>,
    server_counters: Vec<Arc<AtomicU64>>,
    total_counter: Arc<AtomicU64>,
    prev_server_bytes: Vec<u64>,
    prev_total_bytes: u64,
    start: Instant,
    duration_secs: u64,
    /// Number of \n we printed last frame (= how many lines to cursor-up)
    newlines_printed: usize,
    speed_color: &'static str,
    label: &'static str,
}

impl TickState {
    pub fn new(
        urls: &[String],
        server_counters: Vec<Arc<AtomicU64>>,
        total_counter: Arc<AtomicU64>,
        duration_secs: u64,
        speed_color: &'static str,
        label: &'static str,
    ) -> Self {
        let n = urls.len();
        Self {
            server_labels: urls.iter().map(|u| short_host(u).to_string()).collect(),
            server_counters,
            total_counter,
            prev_server_bytes: vec![0; n],
            prev_total_bytes: 0,
            start: Instant::now(),
            duration_secs,
            newlines_printed: 0,
            speed_color,
            label,
        }
    }

    pub fn render(&mut self) -> f64 {
        let n = self.server_labels.len();
        let elapsed = self.start.elapsed().as_secs_f64();
        let total = self.total_counter.load(Ordering::Relaxed);
        let total_tick = total - self.prev_total_bytes;
        self.prev_total_bytes = total;
        let total_instant_mbps = (total_tick as f64 * 8.0) / 0.25 / 1_000_000.0;
        let total_avg_mbps = if elapsed > 0.0 {
            (total as f64 * 8.0) / elapsed / 1_000_000.0
        } else {
            0.0
        };
        let total_instant_bps = total_tick as f64 * 8.0 / 0.25;

        let mut srv_mbps = Vec::with_capacity(n);
        let mut srv_bytes = Vec::with_capacity(n);
        for i in 0..n {
            let b = self.server_counters[i].load(Ordering::Relaxed);
            let tick = b - self.prev_server_bytes[i];
            self.prev_server_bytes[i] = b;
            srv_mbps.push((tick as f64 * 8.0) / 0.25 / 1_000_000.0);
            srv_bytes.push(b);
        }

        let max_mbps = srv_mbps.iter().cloned().fold(1.0f64, f64::max);
        let c = self.speed_color;

        let mut buf = String::with_capacity(1024);

        // Move cursor to overwrite previous frame:
        // \r goes to column 0, then \x1b[<N>A moves up N lines,
        // then \x1b[J clears everything below (kills ghost lines)
        if self.newlines_printed > 0 {
            buf.push_str(&format!("\r\x1b[{}A\x1b[J", self.newlines_printed));
        }

        let mut newlines = 0;

        // Per-server lines
        for i in 0..n {
            let b = bar(srv_mbps[i], max_mbps, 10);
            buf.push_str(&format!(
                "    \x1b[94m{:>24}\x1b[0m {} {}{}\x1b[0m  \x1b[90m{}\x1b[0m\n",
                self.server_labels[i],
                b,
                c,
                format_speed(srv_mbps[i]),
                format_bytes(srv_bytes[i]),
            ));
            newlines += 1;
        }

        // Separator
        buf.push_str(&format!(
            "    \x1b[90m{}\x1b[0m\n",
            "─".repeat(58)
        ));
        newlines += 1;

        // Total line - NO trailing \n
        buf.push_str(&format!(
            "    \x1b[36m{:.1}s/{:.0}s\x1b[0m  {}  {}\x1b[1m{}\x1b[0m  \x1b[90m(avg: {})\x1b[0m  \x1b[90m{}\x1b[0m",
            elapsed,
            self.duration_secs,
            self.label,
            c,
            format_speed(total_instant_mbps),
            format_speed(total_avg_mbps),
            format_bytes(total),
        ));

        self.newlines_printed = newlines;

        let stderr = std::io::stderr();
        let mut handle = stderr.lock();
        let _ = handle.write_all(buf.as_bytes());
        let _ = handle.flush();

        total_instant_bps
    }

    pub fn finish(&self) {
        eprintln!();
    }
}
