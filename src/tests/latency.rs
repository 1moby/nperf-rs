use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::time::Instant;
use tokio_tungstenite::tungstenite::Message;

use crate::stats::LatencyTracker;
use crate::ws;

pub struct ServerLatency {
    pub url: String,
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub jitter_ms: f64,
    pub samples: u32,
}

pub struct LatencyResult {
    pub min_ms: f64,
    pub max_ms: f64,
    pub avg_ms: f64,
    pub jitter_ms: f64,
    pub samples: u32,
    pub servers: Vec<ServerLatency>,
}

/// Run latency test for a single server. Returns per-server result.
pub async fn run_single(
    url: &str,
    insecure: bool,
    num_samples: u32,
    _debug: bool,
) -> Result<ServerLatency> {
    let ws = ws::connect_nperf(url, insecure).await?;
    let (mut sink, mut stream) = ws::split(ws);
    let mut tracker = LatencyTracker::new();

    for _i in 0..num_samples {
        let t0 = Instant::now();
        sink.send(Message::Text("CONNECT".into())).await?;

        match tokio::time::timeout(std::time::Duration::from_secs(5), stream.next()).await {
            Ok(Some(Ok(Message::Text(text)))) if text.starts_with("CONNECTED") => {
                tracker.record(t0.elapsed());
            }
            Ok(Some(Ok(_))) => {}
            _ => break,
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let _ = sink.send(Message::Text("CLOSE".into())).await;

    Ok(ServerLatency {
        url: url.to_string(),
        min_ms: tracker.min_ms(),
        max_ms: tracker.max_ms(),
        avg_ms: tracker.avg_ms(),
        jitter_ms: tracker.jitter_ms(),
        samples: tracker.count() as u32,
    })
}

/// Run latency across all servers in parallel, return aggregate + per-server.
pub async fn run_all(
    urls: &[String],
    insecure: bool,
    total_samples: u32,
    debug: bool,
) -> Result<LatencyResult> {
    let samples_per = (total_samples as usize / urls.len()).max(1) as u32;

    let mut handles = Vec::new();
    for url in urls {
        let url = url.clone();
        handles.push(tokio::spawn(async move {
            run_single(&url, insecure, samples_per, debug).await
        }));
    }

    let mut servers = Vec::new();
    let mut agg_min = f64::MAX;
    let mut agg_max = 0.0f64;
    let mut sum = 0.0f64;
    let mut jitter_sum = 0.0f64;
    let mut count = 0u32;

    for h in handles {
        if let Ok(Ok(s)) = h.await {
            if s.samples > 0 {
                agg_min = agg_min.min(s.min_ms);
                agg_max = agg_max.max(s.max_ms);
                sum += s.avg_ms * s.samples as f64;
                jitter_sum += s.jitter_ms * s.samples as f64;
                count += s.samples;
            }
            servers.push(s);
        }
    }

    if count == 0 {
        anyhow::bail!("All latency tests failed");
    }

    Ok(LatencyResult {
        min_ms: agg_min,
        max_ms: agg_max,
        avg_ms: sum / count as f64,
        jitter_ms: jitter_sum / count as f64,
        samples: count,
        servers,
    })
}
