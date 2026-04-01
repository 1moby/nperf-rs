use owo_colors::OwoColorize;
use serde::Serialize;

use crate::tests::download::DownloadResult;
use crate::tests::latency::LatencyResult;
use crate::tests::upload::UploadResult;

#[derive(Serialize)]
pub struct FullReport {
    pub servers: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<LatencyReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download: Option<ThroughputReport>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upload: Option<ThroughputReport>,
}

#[derive(Serialize)]
pub struct LatencyReport {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub jitter_ms: f64,
    pub samples: u32,
    pub per_server: Vec<ServerLatencyReport>,
}

#[derive(Serialize)]
pub struct ServerLatencyReport {
    pub server: String,
    pub avg_ms: f64,
    pub min_ms: f64,
    pub max_ms: f64,
    pub jitter_ms: f64,
    pub samples: u32,
}

#[derive(Serialize)]
pub struct ThroughputReport {
    pub avg_mbps: f64,
    pub avg_no_slowstart_mbps: f64,
    pub peak_mbps: f64,
    pub bytes_transferred: u64,
    pub duration_secs: f64,
    pub threads: u32,
    pub per_server: Vec<ServerThroughputReport>,
}

#[derive(Serialize)]
pub struct ServerThroughputReport {
    pub server: String,
    pub bytes: u64,
    pub mbps: f64,
    pub threads: u32,
}

fn bps_to_mbps(bps: f64) -> f64 {
    bps / 1_000_000.0
}

fn format_speed(mbps: f64) -> String {
    if mbps >= 1000.0 {
        format!("{:.2} Gbps", mbps / 1000.0)
    } else {
        format!("{:.2} Mbps", mbps)
    }
}

fn format_bytes(bytes: u64) -> String {
    if bytes >= 1_000_000_000 {
        format!("{:.2} GB", bytes as f64 / 1_000_000_000.0)
    } else {
        format!("{:.1} MB", bytes as f64 / 1_000_000.0)
    }
}

fn short_host(url: &str) -> &str {
    let s = url.strip_prefix("wss://").unwrap_or(url);
    s.split('.').next().unwrap_or(s)
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

impl FullReport {
    pub fn new(servers: &[String]) -> Self {
        Self {
            servers: servers.to_vec(),
            latency: None,
            download: None,
            upload: None,
        }
    }

    pub fn set_latency(&mut self, r: &LatencyResult) {
        self.latency = Some(LatencyReport {
            min_ms: round2(r.min_ms),
            max_ms: round2(r.max_ms),
            avg_ms: round2(r.avg_ms),
            jitter_ms: round2(r.jitter_ms),
            samples: r.samples,
            per_server: r
                .servers
                .iter()
                .map(|s| ServerLatencyReport {
                    server: s.url.clone(),
                    avg_ms: round2(s.avg_ms),
                    min_ms: round2(s.min_ms),
                    max_ms: round2(s.max_ms),
                    jitter_ms: round2(s.jitter_ms),
                    samples: s.samples,
                })
                .collect(),
        });
    }

    pub fn set_download(&mut self, r: &DownloadResult) {
        let dur = r.duration_secs;
        self.download = Some(ThroughputReport {
            avg_mbps: round2(bps_to_mbps(r.avg_bps)),
            avg_no_slowstart_mbps: round2(bps_to_mbps(r.avg_no_ss_bps)),
            peak_mbps: round2(bps_to_mbps(r.peak_bps)),
            bytes_transferred: r.bytes,
            duration_secs: round2(dur),
            threads: r.threads,
            per_server: r
                .servers
                .iter()
                .map(|s| ServerThroughputReport {
                    server: s.url.clone(),
                    bytes: s.bytes,
                    mbps: round2(bps_to_mbps(if dur > 0.0 {
                        (s.bytes as f64 * 8.0) / dur
                    } else {
                        0.0
                    })),
                    threads: s.threads,
                })
                .collect(),
        });
    }

    pub fn set_upload(&mut self, r: &UploadResult) {
        let dur = r.duration_secs;
        self.upload = Some(ThroughputReport {
            avg_mbps: round2(bps_to_mbps(r.avg_bps)),
            avg_no_slowstart_mbps: round2(bps_to_mbps(r.avg_no_ss_bps)),
            peak_mbps: round2(bps_to_mbps(r.peak_bps)),
            bytes_transferred: r.bytes,
            duration_secs: round2(dur),
            threads: r.threads,
            per_server: r
                .servers
                .iter()
                .map(|s| ServerThroughputReport {
                    server: s.url.clone(),
                    bytes: s.bytes,
                    mbps: round2(bps_to_mbps(if dur > 0.0 {
                        (s.bytes as f64 * 8.0) / dur
                    } else {
                        0.0
                    })),
                    threads: s.threads,
                })
                .collect(),
        });
    }

    pub fn print_text(&self) {
        println!();
        let title = " nperf-rs Speed Test Results ";
        println!(
            "{}{}{}",
            "╔══".bright_black(),
            title.bold().on_blue().white(),
            "══════════════════════════════════╗".bright_black()
        );
        println!(
            "{}",
            "║                                                              ║".bright_black()
        );

        // Latency
        if let Some(ref l) = self.latency {
            println!(
                "{}  {} {}",
                "║".bright_black(),
                "LATENCY".bold().yellow(),
                "                                                    ║".bright_black()
            );
            println!(
                "{}  {}                                                 {}",
                "║".bright_black(),
                format!(
                    "{:.1} ms",
                    l.avg_ms
                )
                .bold()
                .cyan(),
                "║".bright_black()
            );
            println!(
                "{}  {} {:.1} ms  {} {:.1} ms  {} {:.1} ms             {}",
                "║".bright_black(),
                "min".bright_black(),
                l.min_ms,
                "max".bright_black(),
                l.max_ms,
                "jitter".bright_black(),
                l.jitter_ms,
                "║".bright_black()
            );
            for s in &l.per_server {
                let host = short_host(&s.server);
                let bar = make_latency_bar(s.avg_ms, l.max_ms);
                println!(
                    "{}  {:>22} {} {:.1} ms                    {}",
                    "║".bright_black(),
                    host.bright_blue(),
                    bar,
                    s.avg_ms,
                    "║".bright_black()
                );
            }
            println!(
                "{}                                                              {}",
                "║".bright_black(),
                "║".bright_black()
            );
        }

        // Download
        if let Some(ref d) = self.download {
            println!(
                "{}  {} {}",
                "║".bright_black(),
                "DOWNLOAD".bold().green(),
                "                                                   ║".bright_black()
            );
            println!(
                "{}  {}  {} {}               {}",
                "║".bright_black(),
                format_speed(d.avg_no_slowstart_mbps).bold().green(),
                format!("peak {}", format_speed(d.peak_mbps)).bright_black(),
                format!("{}", format_bytes(d.bytes_transferred)).bright_black(),
                "║".bright_black()
            );
            for s in &d.per_server {
                if s.bytes == 0 { continue; }
                let host = short_host(&s.server);
                let bar = make_speed_bar(s.mbps, d.peak_mbps);
                println!(
                    "{}  {:>22} {} {} {}      {}",
                    "║".bright_black(),
                    host.bright_blue(),
                    bar,
                    format_speed(s.mbps).green(),
                    format_bytes(s.bytes).bright_black(),
                    "║".bright_black()
                );
            }
            println!(
                "{}                                                              {}",
                "║".bright_black(),
                "║".bright_black()
            );
        }

        // Upload
        if let Some(ref u) = self.upload {
            println!(
                "{}  {} {}",
                "║".bright_black(),
                "UPLOAD".bold().magenta(),
                "                                                     ║".bright_black()
            );
            println!(
                "{}  {}  {} {}               {}",
                "║".bright_black(),
                format_speed(u.avg_no_slowstart_mbps).bold().magenta(),
                format!("peak {}", format_speed(u.peak_mbps)).bright_black(),
                format!("{}", format_bytes(u.bytes_transferred)).bright_black(),
                "║".bright_black()
            );
            for s in &u.per_server {
                if s.bytes == 0 { continue; }
                let host = short_host(&s.server);
                let bar = make_speed_bar(s.mbps, u.peak_mbps);
                println!(
                    "{}  {:>22} {} {} {}      {}",
                    "║".bright_black(),
                    host.bright_blue(),
                    bar,
                    format_speed(s.mbps).magenta(),
                    format_bytes(s.bytes).bright_black(),
                    "║".bright_black()
                );
            }
            println!(
                "{}                                                              {}",
                "║".bright_black(),
                "║".bright_black()
            );
        }

        println!(
            "{}",
            "╚══════════════════════════════════════════════════════════════╝".bright_black()
        );
        println!();
    }

    pub fn print_json(&self) {
        println!("{}", serde_json::to_string_pretty(self).unwrap());
    }
}

fn make_latency_bar(val: f64, max: f64) -> String {
    let width = 12;
    let filled = if max > 0.0 {
        ((val / max) * width as f64).round() as usize
    } else {
        0
    };
    let filled = filled.min(width);
    let empty = width - filled;
    format!(
        "{}{}",
        "█".repeat(filled).yellow(),
        "░".repeat(empty).bright_black()
    )
}

fn make_speed_bar(val: f64, max: f64) -> String {
    let width = 12;
    let filled = if max > 0.0 {
        ((val / max) * width as f64).round() as usize
    } else {
        0
    };
    let filled = filled.min(width);
    let empty = width - filled;
    format!(
        "{}{}",
        "█".repeat(filled).cyan(),
        "░".repeat(empty).bright_black()
    )
}
