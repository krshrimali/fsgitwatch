use crate::error::{FsgitError, Result};
use crate::git;
use crate::matcher::RepositoryPattern;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::{Mutex, Semaphore};

#[derive(Debug, Clone)]
pub struct MatchResult {
    pub path: PathBuf,
    pub remotes: Vec<(String, String)>,
}

pub struct Scanner {
    search_path: PathBuf,
    pattern: RepositoryPattern,
    max_concurrent: usize,
    verbose: bool,
}

impl Scanner {
    pub fn new(
        search_path: PathBuf,
        pattern: RepositoryPattern,
        max_concurrent: usize,
        verbose: bool,
    ) -> Self {
        Self {
            search_path,
            pattern,
            max_concurrent,
            verbose,
        }
    }

    /// Perform the async scan for matching repositories
    pub async fn scan(&self) -> Result<Vec<MatchResult>> {
        let results = Arc::new(Mutex::new(Vec::new()));
        let semaphore = Arc::new(Semaphore::new(self.max_concurrent));
        let pattern = Arc::new(self.pattern.clone());

        // Start scanning from the root path
        self.scan_directory(
            self.search_path.clone(),
            results.clone(),
            semaphore.clone(),
            pattern.clone(),
        )
        .await??;

        // Extract results from Arc<Mutex<>>
        let final_results = Arc::try_unwrap(results)
            .map_err(|_| FsgitError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to unwrap results",
            )))?
            .into_inner();

        Ok(final_results)
    }

    /// Recursively scan a directory for git repositories
    fn scan_directory(
        &self,
        path: PathBuf,
        results: Arc<Mutex<Vec<MatchResult>>>,
        semaphore: Arc<Semaphore>,
        pattern: Arc<RepositoryPattern>,
    ) -> tokio::task::JoinHandle<Result<()>> {
        let verbose = self.verbose;
        let scanner = self.clone();

        tokio::spawn(async move {
            // Acquire semaphore permit for bounded concurrency
            let _permit = semaphore.acquire().await.map_err(|_| {
                FsgitError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to acquire semaphore permit",
                ))
            })?;

            // Try to read directory
            let mut entries = match fs::read_dir(&path).await {
                Ok(entries) => entries,
                Err(e) => {
                    // Soft failure - permission denied or other IO errors
                    if verbose {
                        eprintln!("Warning: Cannot read directory {}: {}", path.display(), e);
                    }
                    return Ok(());
                }
            };

            let mut subdirs = Vec::new();

            // First pass: collect entries and check for .git directory
            while let Some(entry) = entries.next_entry().await? {
                let entry_path = entry.path();
                let file_name = entry.file_name();

                // Check if this is a .git directory
                if file_name == ".git" {
                    // This is a git repository - check if it matches our pattern
                    if let Ok(remotes) = git::get_remote_urls(&path).await {
                        // Check if any remote matches the pattern
                        let matching_remotes: Vec<(String, String)> = remotes
                            .iter()
                            .filter(|(_, url)| pattern.matches(url))
                            .cloned()
                            .collect();

                        if !matching_remotes.is_empty() {
                            // This repo matches! Add it to results
                            let mut results_guard = results.lock().await;
                            results_guard.push(MatchResult {
                                path: path.clone(),
                                remotes: matching_remotes,
                            });
                        }
                    } else if verbose {
                        eprintln!(
                            "Warning: Failed to read remotes from git repo at {}",
                            path.display()
                        );
                    }

                    // CRITICAL: Return early - don't scan subdirectories of git repos
                    return Ok(());
                }

                // Collect subdirectories for later scanning
                if let Ok(file_type) = entry.file_type().await {
                    if file_type.is_dir() {
                        subdirs.push(entry_path);
                    }
                }
            }

            // Only scan subdirectories if NO .git was found (we return early when .git is found)
            let mut tasks = Vec::new();

            for subdir in subdirs {
                let task = scanner.scan_directory(
                    subdir,
                    results.clone(),
                    semaphore.clone(),
                    pattern.clone(),
                );
                tasks.push(task);
            }

            // Wait for all subtasks to complete
            for task in tasks {
                if let Err(e) = task.await? {
                    if verbose {
                        eprintln!("Warning: Scan task failed: {}", e);
                    }
                }
            }

            Ok(())
        })
    }
}

// Implement Clone for Scanner to allow spawning tasks
impl Clone for Scanner {
    fn clone(&self) -> Self {
        Self {
            search_path: self.search_path.clone(),
            pattern: self.pattern.clone(),
            max_concurrent: self.max_concurrent,
            verbose: self.verbose,
        }
    }
}
