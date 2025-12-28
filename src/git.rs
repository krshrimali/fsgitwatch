use crate::error::Result;
use git2::Repository;
use std::path::Path;
use tokio::task;

/// Get all remote URLs from a git repository
/// Returns a vector of (remote_name, url) tuples
pub async fn get_remote_urls(repo_path: &Path) -> Result<Vec<(String, String)>> {
    let path = repo_path.to_path_buf();

    // Wrap blocking git2 operations in spawn_blocking
    task::spawn_blocking(move || {
        let repo = Repository::open(&path)?;
        let remotes = repo.remotes()?;

        let mut urls = Vec::new();
        for remote_name in remotes.iter() {
            if let Some(name) = remote_name {
                if let Ok(remote) = repo.find_remote(name) {
                    if let Some(url) = remote.url() {
                        urls.push((name.to_string(), url.to_string()));
                    }
                }
            }
        }

        Ok(urls)
    })
    .await?
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;
    use tempfile::TempDir;

    async fn create_test_repo_with_remote(remote_url: &str) -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Add remote
        Command::new("git")
            .args(&["remote", "add", "origin", remote_url])
            .current_dir(repo_path)
            .output()
            .unwrap();

        temp_dir
    }

    #[tokio::test]
    async fn test_get_remote_urls() {
        let temp_dir = create_test_repo_with_remote("https://github.com/test/repo.git").await;
        let remotes = get_remote_urls(temp_dir.path()).await.unwrap();

        assert_eq!(remotes.len(), 1);
        assert_eq!(remotes[0].0, "origin");
        assert_eq!(remotes[0].1, "https://github.com/test/repo.git");
    }

    #[tokio::test]
    async fn test_multiple_remotes() {
        let temp_dir = TempDir::new().unwrap();
        let repo_path = temp_dir.path();

        // Initialize git repo
        Command::new("git")
            .args(&["init"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Add multiple remotes
        Command::new("git")
            .args(&["remote", "add", "origin", "https://github.com/test/repo.git"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        Command::new("git")
            .args(&["remote", "add", "upstream", "git@github.com:upstream/repo.git"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let remotes = get_remote_urls(repo_path).await.unwrap();

        assert_eq!(remotes.len(), 2);
        assert!(remotes.iter().any(|(name, _)| name == "origin"));
        assert!(remotes.iter().any(|(name, _)| name == "upstream"));
    }
}
