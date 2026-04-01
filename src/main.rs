mod cli;
mod display;
mod results;
mod servers;
mod stats;
mod tests;
mod ws;

use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;

use cli::Cli;
use results::FullReport;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    // --list: print server table and exit
    if args.list {
        let all = servers::all();
        let list = if let Some(ref q) = args.filter {
            servers::filter(&all, q)
        } else {
            all
        };
        servers::print_list(&list);
        return Ok(());
    }

    let urls = args.resolved_urls();
    if urls.is_empty() {
        eprintln!("No servers matched. Use --list to see available servers.");
        return Ok(());
    }

    eprintln!();
    eprintln!(
        "  {} {}",
        ">>>".bold().cyan(),
        "nperf-rs".bold().white()
    );
    eprintln!(
        "  {}",
        "────────────────────────────────────────".bright_black()
    );
    for url in &urls {
        let host = url
            .strip_prefix("wss://")
            .unwrap_or(url)
            .split('/')
            .next()
            .unwrap_or(url);
        eprintln!("  {} {}", "●".bright_blue(), host.bright_blue());
    }
    eprintln!();

    let mut report = FullReport::new(&urls);

    // Latency test - all servers in parallel
    if !args.no_latency {
        eprintln!(
            "  {} {}",
            "⏱".yellow(),
            "Latency test...".bold().yellow()
        );
        match tests::latency::run_all(&urls, args.insecure, args.latency_samples, args.debug).await
        {
            Ok(result) => {
                for s in &result.servers {
                    let host = s
                        .url
                        .strip_prefix("wss://")
                        .unwrap_or(&s.url)
                        .split('.')
                        .next()
                        .unwrap_or(&s.url);
                    eprintln!(
                        "    {} {:>22}  {:.1} ms",
                        "·".bright_black(),
                        host.bright_blue(),
                        s.avg_ms,
                    );
                }
                report.set_latency(&result);
            }
            Err(e) => {
                eprintln!("    {} {}", "✗".red(), e);
            }
        }
        eprintln!();
    }

    // Download test
    if !args.no_download {
        eprintln!(
            "  {} {}",
            "↓".green(),
            "Download test...".bold().green()
        );
        match tests::download::run_multi(
            &urls,
            args.insecure,
            args.threads,
            args.download_duration,
            args.slow_start,
            args.debug,
        )
        .await
        {
            Ok(result) => {
                report.set_download(&result);
            }
            Err(e) => {
                eprintln!("    {} {}", "✗".red(), e);
            }
        }
        eprintln!();
    }

    // Upload test
    if !args.no_upload {
        eprintln!(
            "  {} {}",
            "↑".magenta(),
            "Upload test...".bold().magenta()
        );
        match tests::upload::run_multi(
            &urls,
            args.insecure,
            args.threads,
            args.upload_duration,
            args.slow_start,
            args.debug,
        )
        .await
        {
            Ok(result) => {
                report.set_upload(&result);
            }
            Err(e) => {
                eprintln!("    {} {}", "✗".red(), e);
            }
        }
        eprintln!();
    }

    // Final report
    if args.json {
        report.print_json();
    } else {
        report.print_text();
    }

    Ok(())
}
