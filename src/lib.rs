pub mod cli;
pub mod error;
pub mod git;
pub mod matcher;
pub mod output;
pub mod progress;
pub mod scanner;

// Re-export commonly used types for convenience
pub use cli::Cli;
pub use error::{FsgitError, Result};
pub use matcher::RepositoryPattern;
pub use progress::{ProgressMessage, ProgressTracker};
pub use scanner::{MatchResult, Scanner};
