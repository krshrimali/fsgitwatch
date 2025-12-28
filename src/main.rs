mod cli;
mod error;
mod git;
mod matcher;
mod output;
mod progress;
mod scanner;

use clap::Parser;
use cli::Cli;
use colored::Colorize;
use matcher::RepositoryPattern;
use progress::{ProgressMessage, ProgressTracker};
use scanner::Scanner;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse();

    // Determine search path (default to current directory)
    let search_path = cli
        .search_path
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    // Validate that search path exists
    if !search_path.exists() {
        eprintln!("Error: Search path does not exist: {}", search_path.display());
        std::process::exit(1);
    }

    // Parse repository pattern
    let pattern = RepositoryPattern::new(&cli.pattern)?;

    // Determine if we should show progress bar
    let show_progress = !cli.json && !cli.no_progress;

    // Create progress channel if we're showing progress or in verbose mode
    let (progress_tx, progress_rx) = if show_progress || cli.verbose > 0 {
        let (tx, rx) = mpsc::unbounded_channel();
        (Some(tx), Some(rx))
    } else {
        (None, None)
    };

    // Create scanner
    let scanner = Scanner::new(search_path, pattern, cli.max_concurrent, cli.verbose);

    // Clone pattern string for tracker
    let pattern_str = cli.pattern.clone();

    // Spawn progress tracker if we have a receiver
    let tracker_handle = if let Some(rx) = progress_rx {
        Some(tokio::spawn(async move {
            let tracker = ProgressTracker::new(rx, show_progress, cli.verbose, pattern_str);
            tracker.run().await
        }))
    } else {
        None
    };

    // Run async scan
    let scan_results = scanner.scan(progress_tx.clone()).await?;

    // Send done message to progress tracker
    if let Some(tx) = progress_tx {
        let _ = tx.send(ProgressMessage::Done);
    }

    // Get results from progress tracker or use scan results
    let results = if let Some(handle) = tracker_handle {
        handle.await?
    } else {
        scan_results
    };

    // Output results (only if not in streaming mode)
    if cli.json {
        output::print_json(&results, &cli.pattern)?;
    } else if !show_progress {
        // If we didn't show progress, print results now
        output::print_results(&results, &cli.pattern);
    } else {
        // Progress bar already printed results, just show summary
        if results.is_empty() {
            println!(
                "\n{}",
                format!("No repositories found matching '{}'", cli.pattern)
                    .yellow()
                    .bold()
            );
        } else {
            println!(
                "\n{} {} matching '{}'",
                "Found".green().bold(),
                if results.len() == 1 {
                    format!("{} repository", results.len())
                } else {
                    format!("{} repositories", results.len())
                },
                cli.pattern.cyan()
            );
        }
    }

    // Exit with code 0 if found, 1 if not found
    std::process::exit(if results.is_empty() { 1 } else { 0 });
}
