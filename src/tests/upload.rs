use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::Message;

use crate::display::TickState;
use crate::stats::ThroughputState;
use crate::ws;

pub struct ServerResult {
    pub url: String,
    pub bytes: u64,
    pub threads: u32,
}

pub struct UploadResult {
    pub avg_bps: f64,
    pub avg_no_ss_bps: f64,
    pub peak_bps: f64,
    pub bytes: u64,
    pub duration_secs: f64,
    pub threads: u32,
    pub servers: Vec<ServerResult>,
}

pub async fn run_multi(
    urls: &[String],
    insecure: bool,
    total_threads: u32,
    duration_secs: u64,
    slow_start_secs: u64,
    debug: bool,
) -> Result<UploadResult> {
    let per_server = distribute_threads(total_threads, urls.len());

    let _conn_start = Instant::now();
    let mut stream_assignments: Vec<(usize, crate::ws::WsStream)> = Vec::new();
    let mut server_thread_counts: Vec<u32> = vec![0; urls.len()];

    for (idx, (url, &n)) in urls.iter().zip(per_server.iter()).enumerate() {
        if n == 0 { continue; }
        match ws::connect_nperf_pool(url, n, insecure).await {
            Ok(streams) => {
                server_thread_counts[idx] = streams.len() as u32;
                for s in streams { stream_assignments.push((idx, s)); }
            }
            Err(e) => {
                if debug { eprintln!("    Warning: {}: {}", url, e); }
            }
        }
    }
    if stream_assignments.is_empty() {
        anyhow::bail!("No connections established");
    }
    let actual_threads = stream_assignments.len() as u32;

    let state = Arc::new(ThroughputState::new(Duration::from_secs(slow_start_secs)));
    let server_counters: Vec<Arc<AtomicU64>> = (0..urls.len())
        .map(|_| Arc::new(AtomicU64::new(0)))
        .collect();
    let (stop_tx, stop_rx) = watch::channel(false);

    let duration_ms = duration_secs * 1000;
    let buf_size: u64 = 10_737_418_240;
    let random_id = gen_random_id();
    let chunk: Vec<u8> = (0..65536u32).map(|i| (i % 256) as u8).collect();

    let mut worker_handles = Vec::new();
    for (i, (server_idx, stream)) in stream_assignments.into_iter().enumerate() {
        let counter = state.total_bytes.clone();
        let srv_counter = server_counters[server_idx].clone();
        let stop = stop_rx.clone();
        let data = chunk.clone();
        let dbg = debug;
        let cmd = format!("UL {} {} {}", buf_size, duration_ms, random_id);

        worker_handles.push(tokio::spawn(async move {
            let (mut sink, _source) = stream.split();
            if let Err(e) = sink.send(Message::Text(cmd.into())).await {
                if dbg { eprintln!("    [ul-{}] cmd error: {}", i, e); }
                return;
            }
            loop {
                if *stop.borrow() { break; }
                let msg = Message::Binary(data.clone().into());
                match sink.send(msg).await {
                    Ok(()) => {
                        let len = data.len() as u64;
                        counter.fetch_add(len, Ordering::Relaxed);
                        srv_counter.fetch_add(len, Ordering::Relaxed);
                    }
                    Err(_) => break,
                }
            }
            let _ = sink.send(Message::Text("CLOSE".into())).await;
        }));
    }

    let tick_state = state.clone();
    let tick_counters = server_counters.clone();
    let tick_urls = urls.to_vec();
    let tick_stop = stop_rx.clone();
    let tick_handle = tokio::spawn(async move {
        let mut display = TickState::new(
            &tick_urls,
            tick_counters,
            tick_state.total_bytes.clone(),
            duration_secs,
            "\x1b[35m", // magenta
            "↑ UL",
        );
        loop {
            tokio::time::sleep(Duration::from_millis(250)).await;
            if *tick_stop.borrow() { break; }
            let instant_bps = display.render();
            tick_state.record_tick(instant_bps);
        }
        let instant_bps = display.render();
        tick_state.record_tick(instant_bps);
        display.finish();
    });

    tokio::time::sleep(Duration::from_secs(duration_secs)).await;
    let _ = stop_tx.send(true);
    for h in worker_handles { let _ = h.await; }
    let _ = tick_handle.await;

    let servers: Vec<ServerResult> = urls.iter().enumerate()
        .map(|(idx, url)| ServerResult {
            url: url.clone(),
            bytes: server_counters[idx].load(Ordering::Relaxed),
            threads: server_thread_counts[idx],
        })
        .collect();

    Ok(UploadResult {
        avg_bps: state.average_including_slow_start(),
        avg_no_ss_bps: state.average_excluding_slow_start(),
        peak_bps: state.peak(),
        bytes: state.total_bytes(),
        duration_secs: state.duration_secs(),
        threads: actual_threads,
        servers,
    })
}

fn distribute_threads(total: u32, n: usize) -> Vec<u32> {
    let n = n as u32;
    let base = (total / n).max(1);
    let mut extra = total.saturating_sub(base * n);
    (0..n).map(|_| { if extra > 0 { extra -= 1; base + 1 } else { base } }).collect()
}

fn gen_random_id() -> String {
    (0..8).map(|_| {
        let c = rand::random::<u8>() % 62;
        (if c < 10 { b'0' + c } else if c < 36 { b'A' + c - 10 } else { b'a' + c - 36 }) as char
    }).collect()
}
