use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fsgitwatch")]
#[command(about = "Find git repositories matching owner/repo pattern")]
#[command(version)]
#[command(long_about = "Asynchronously search for git repositories by owner/repo pattern.
Supports both SSH (git@github.com:owner/repo.git) and HTTPS (https://github.com/owner/repo.git) URLs.
Checks all git remotes (origin, upstream, etc.) and intelligently prunes directory traversal.")]
pub struct Cli {
    /// Repository pattern in owner/repo format (e.g., 'anthropics/claude-code')
    #[arg(value_name = "PATTERN")]
    pub pattern: String,

    /// Directory to search (defaults to current directory)
    #[arg(value_name = "PATH")]
    pub search_path: Option<PathBuf>,

    /// Maximum number of concurrent scan tasks
    #[arg(short = 'j', long, default_value = "100")]
    pub max_concurrent: usize,

    /// Output results as JSON
    #[arg(long)]
    pub json: bool,

    /// Verbose output (show warnings and debugging information)
    #[arg(short, long)]
    pub verbose: bool,
}
