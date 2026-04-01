use clap::Parser;

pub const DEFAULT_URLS: &[&str] = &[
    "wss://th-true-bangkok-01-10g.nperf.net/wsock",
    "wss://th-3bb-bangkok-01-10g.nperf.net/wsock",
    "wss://th-ais-bangkok-01-10g.nperf.net/wsock",
];

#[derive(Parser, Debug)]
#[command(name = "nperf-rs", about = "Network performance benchmark via nperf WebSocket servers")]
pub struct Cli {
    /// Server URL(s) (e.g., wss://host/wsock). Can be specified multiple times.
    #[arg(short = 'u', long = "url")]
    pub urls: Vec<String>,

    /// Server hostname (shorthand: builds wss://HOST:PORT/wsock)
    #[arg(short = 'H', long)]
    pub host: Option<String>,

    /// WebSocket secure port, used with --host (0 or 443 = no port in URL)
    #[arg(short, long, default_value_t = 8443)]
    pub port: u16,

    /// Total number of parallel WebSocket connections (spread across all servers)
    #[arg(short, long, default_value_t = 3)]
    pub threads: u32,

    /// Pick N random servers from the server list
    #[arg(long, value_name = "N")]
    pub random: Option<usize>,

    /// Filter servers by city/provider/hostname (substring match)
    #[arg(short, long, value_name = "QUERY")]
    pub filter: Option<String>,

    /// List all available servers and exit
    #[arg(short, long)]
    pub list: bool,

    /// Download test duration in seconds
    #[arg(long, default_value_t = 10)]
    pub download_duration: u64,

    /// Upload test duration in seconds
    #[arg(long, default_value_t = 10)]
    pub upload_duration: u64,

    /// Latency test sample count
    #[arg(long, default_value_t = 30)]
    pub latency_samples: u32,

    /// Slow start exclusion period in seconds
    #[arg(long, default_value_t = 3)]
    pub slow_start: u64,

    /// Skip latency test
    #[arg(long)]
    pub no_latency: bool,

    /// Skip download test
    #[arg(long)]
    pub no_download: bool,

    /// Skip upload test
    #[arg(long)]
    pub no_upload: bool,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Accept invalid TLS certificates
    #[arg(long)]
    pub insecure: bool,

    /// Debug mode: dump raw WebSocket frames
    #[arg(long)]
    pub debug: bool,
}

impl Cli {
    /// Resolve the list of WebSocket URLs to test against.
    pub fn resolved_urls(&self) -> Vec<String> {
        // Explicit URLs take priority
        if !self.urls.is_empty() {
            return self.urls.clone();
        }
        // Single host shorthand
        if let Some(ref host) = self.host {
            let url = if self.port == 0 || self.port == 443 {
                format!("wss://{}/wsock", host)
            } else {
                format!("wss://{}:{}/wsock", host, self.port)
            };
            return vec![url];
        }
        // --random N: pick N from full server list
        if let Some(n) = self.random {
            let all = crate::servers::all();
            let filtered = if let Some(ref q) = self.filter {
                crate::servers::filter(&all, q)
            } else {
                all
            };
            let picked = crate::servers::pick_random(&filtered, n);
            return picked.iter().map(|s| s.url.clone()).collect();
        }
        // --filter without --random: use all matching servers
        if let Some(ref q) = self.filter {
            let all = crate::servers::all();
            let filtered = crate::servers::filter(&all, q);
            return filtered.iter().map(|s| s.url.clone()).collect();
        }
        // Default servers
        DEFAULT_URLS.iter().map(|s| s.to_string()).collect()
    }
}
