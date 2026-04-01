use owo_colors::OwoColorize;

const SERVER_CSV: &str = include_str!("../nperf-server.csv");

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct Server {
    pub pool_id: u32,
    pub country: String,
    pub city: String,
    pub provider: String,
    pub speed: String,
    pub hostname: String,
    pub ip: String,
    pub url: String,
}

impl Server {
    pub fn short_name(&self) -> &str {
        self.hostname.split('.').next().unwrap_or(&self.hostname)
    }
}

/// Parse the embedded CSV into a list of servers.
pub fn all() -> Vec<Server> {
    let mut servers = Vec::new();
    for line in SERVER_CSV.lines().skip(1) {
        let fields: Vec<&str> = line.splitn(10, ',').collect();
        if fields.len() < 10 {
            continue;
        }
        servers.push(Server {
            pool_id: fields[0].parse().unwrap_or(0),
            country: fields[1].to_string(),
            city: fields[2].to_string(),
            provider: fields[3].to_string(),
            speed: fields[4].to_string(),
            hostname: fields[5].to_string(),
            ip: fields[6].to_string(),
            url: fields[9].to_string(),
        });
    }
    servers
}

/// Filter servers by city or provider (case-insensitive substring match).
pub fn filter(servers: &[Server], query: &str) -> Vec<Server> {
    let q = query.to_lowercase();
    servers
        .iter()
        .filter(|s| {
            s.city.to_lowercase().contains(&q)
                || s.provider.to_lowercase().contains(&q)
                || s.hostname.to_lowercase().contains(&q)
                || s.short_name().to_lowercase().contains(&q)
        })
        .cloned()
        .collect()
}

/// Pick `n` random servers from the list.
pub fn pick_random(servers: &[Server], n: usize) -> Vec<Server> {
    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();
    let mut pool = servers.to_vec();
    pool.shuffle(&mut rng);
    pool.truncate(n);
    pool
}

/// Print server list in a formatted table.
pub fn print_list(servers: &[Server]) {
    eprintln!(
        "  {:>4}  {:<14} {:<22} {:<10} {:<9} {}",
        "#".bright_black(),
        "Provider".bright_black(),
        "City".bright_black(),
        "Speed".bright_black(),
        "Country".bright_black(),
        "Hostname".bright_black(),
    );
    eprintln!(
        "  {}",
        "─".repeat(90).bright_black()
    );
    for (i, s) in servers.iter().enumerate() {
        eprintln!(
            "  {:>4}  {:<14} {:<22} {:<10} {:<9} {}",
            (i + 1).to_string().cyan(),
            s.provider.green(),
            s.city.white(),
            s.speed.yellow(),
            s.country.bright_black(),
            s.short_name().bright_blue(),
        );
    }
    eprintln!();
    eprintln!("  {} servers total", servers.len().to_string().bold());
}
