mod cli;
mod error;
mod git;
mod matcher;
mod output;
mod scanner;

use clap::Parser;
use cli::Cli;
use matcher::RepositoryPattern;
use scanner::Scanner;

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

    if cli.verbose {
        eprintln!(
            "Searching for '{}' in {} with {} concurrent tasks...",
            cli.pattern,
            search_path.display(),
            cli.max_concurrent
        );
    }

    // Create scanner
    let scanner = Scanner::new(search_path, pattern, cli.max_concurrent, cli.verbose);

    // Run async scan
    let results = scanner.scan().await?;

    // Output results
    if cli.json {
        output::print_json(&results, &cli.pattern)?;
    } else {
        output::print_results(&results, &cli.pattern);
    }

    // Exit with code 0 if found, 1 if not found
    std::process::exit(if results.is_empty() { 1 } else { 0 });
}
