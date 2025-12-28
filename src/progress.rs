use crate::scanner::MatchResult;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;
use tokio::sync::mpsc;

/// Messages sent from scanner to progress tracker
#[derive(Debug, Clone)]
pub enum ProgressMessage {
    /// A directory is being scanned
    ScanningDirectory(PathBuf),
    /// A match was found
    MatchFound(MatchResult),
    /// A warning occurred
    Warning(String),
    /// Scanning is complete
    Done,
}

/// Progress tracker that displays scan progress and matches in real-time
pub struct ProgressTracker {
    rx: mpsc::UnboundedReceiver<ProgressMessage>,
    progress_bar: Option<ProgressBar>,
    show_progress: bool,
    verbose_level: u8,
    pattern: String,
}

impl ProgressTracker {
    pub fn new(
        rx: mpsc::UnboundedReceiver<ProgressMessage>,
        show_progress: bool,
        verbose_level: u8,
        pattern: String,
    ) -> Self {
        let progress_bar = if show_progress {
            let pb = ProgressBar::new_spinner();
            pb.set_style(
                ProgressStyle::default_spinner()
                    .template("{spinner:.green} [{elapsed_precise}] {msg}")
                    .unwrap()
                    .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ "),
            );
            pb.enable_steady_tick(std::time::Duration::from_millis(100));
            Some(pb)
        } else {
            None
        };

        Self {
            rx,
            progress_bar,
            show_progress,
            verbose_level,
            pattern,
        }
    }

    /// Run the progress tracker, consuming messages and updating display
    /// Returns the collected matches
    pub async fn run(mut self) -> Vec<MatchResult> {
        let mut matches = Vec::new();
        let mut dirs_scanned = 0;

        while let Some(msg) = self.rx.recv().await {
            match msg {
                ProgressMessage::ScanningDirectory(path) => {
                    dirs_scanned += 1;

                    // Verbose level 2 (-vv): Show each directory being scanned
                    if self.verbose_level >= 2 {
                        if let Some(pb) = &self.progress_bar {
                            pb.println(format!("Scanning: {}", path.display()));
                        } else {
                            eprintln!("Scanning: {}", path.display());
                        }
                    }

                    // Update progress bar
                    if let Some(pb) = &self.progress_bar {
                        pb.set_message(format!(
                            "Scanned {} directories, found {} matches",
                            dirs_scanned,
                            matches.len()
                        ));
                    }
                }
                ProgressMessage::MatchFound(result) => {
                    // Print match immediately (streaming output)
                    // Always show matches when progress bar is enabled
                    if let Some(pb) = &self.progress_bar {
                        pb.println(self.format_match(&result, matches.len() + 1));
                    }

                    matches.push(result);

                    // Update progress bar
                    if let Some(pb) = &self.progress_bar {
                        pb.set_message(format!(
                            "Scanned {} directories, found {} matches",
                            dirs_scanned,
                            matches.len()
                        ));
                    }
                }
                ProgressMessage::Warning(msg) => {
                    // Display warnings through progress bar to avoid interference
                    if self.verbose_level >= 1 {
                        if let Some(pb) = &self.progress_bar {
                            pb.println(msg);
                        } else {
                            eprintln!("{}", msg);
                        }
                    }
                }
                ProgressMessage::Done => {
                    break;
                }
            }
        }

        // Finish progress bar
        if let Some(pb) = &self.progress_bar {
            pb.finish_with_message(format!(
                "Scan complete: {} directories scanned, {} matches found",
                dirs_scanned,
                matches.len()
            ));
        }

        matches
    }

    /// Format a match result for display
    fn format_match(&self, result: &MatchResult, index: usize) -> String {
        use colored::Colorize;

        let mut output = format!(
            "\n{}. {}",
            index.to_string().yellow(),
            result.path.display().to_string().bold()
        );

        for (remote_name, url) in &result.remotes {
            output.push_str(&format!("\n   {}: {}", remote_name.blue(), url));
        }

        output
    }
}
