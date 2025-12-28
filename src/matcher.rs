use crate::error::{FsgitError, Result};
use git_url_parse::GitUrl;

#[derive(Debug, Clone)]
pub struct RepositoryPattern {
    owner: String,
    repo: String,
}

impl RepositoryPattern {
    /// Create a new repository pattern from "owner/repo" format
    pub fn new(pattern: &str) -> Result<Self> {
        let parts: Vec<&str> = pattern.split('/').collect();
        if parts.len() != 2 {
            return Err(FsgitError::InvalidPattern(pattern.to_string()));
        }

        let owner = parts[0].trim();
        let repo = parts[1].trim();

        if owner.is_empty() || repo.is_empty() {
            return Err(FsgitError::InvalidPattern(pattern.to_string()));
        }

        Ok(Self {
            owner: owner.to_string(),
            repo: repo.to_string(),
        })
    }

    /// Check if a remote URL matches this pattern
    pub fn matches(&self, remote_url: &str) -> bool {
        // Try using git-url-parse first
        match GitUrl::parse(remote_url) {
            Ok(parsed) => {
                // Extract owner and repo from parsed URL
                let url_owner = parsed.owner.as_deref().unwrap_or("");
                let url_repo = self.normalize_repo_name(&parsed.name);

                self.owner.eq_ignore_ascii_case(url_owner) && self.repo.eq_ignore_ascii_case(&url_repo)
            }
            Err(_) => {
                // Fallback to manual parsing if git-url-parse fails
                self.manual_parse(remote_url)
            }
        }
    }

    /// Normalize repository name by stripping .git suffix
    fn normalize_repo_name(&self, repo_name: &str) -> String {
        repo_name.trim_end_matches(".git").to_string()
    }

    /// Manual parsing fallback for edge cases
    fn manual_parse(&self, url: &str) -> bool {
        // Try to extract owner/repo from various URL formats
        // Pattern: .*[:/]owner/repo(.git)?

        // Remove common prefixes
        let url = url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("ssh://")
            .trim_start_matches("git@");

        // Look for owner/repo pattern
        // Examples:
        // github.com:owner/repo.git
        // github.com/owner/repo.git
        // github.com/owner/repo

        if let Some(colon_pos) = url.find(':') {
            // SSH format: github.com:owner/repo
            let after_colon = &url[colon_pos + 1..];
            return self.match_owner_repo_path(after_colon);
        }

        if let Some(slash_pos) = url.find('/') {
            // HTTPS format: github.com/owner/repo
            let after_first_slash = &url[slash_pos + 1..];
            return self.match_owner_repo_path(after_first_slash);
        }

        false
    }

    /// Check if a path segment matches owner/repo pattern
    fn match_owner_repo_path(&self, path: &str) -> bool {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() < 2 {
            return false;
        }

        let url_owner = parts[0];
        let url_repo = self.normalize_repo_name(parts[1]);

        self.owner.eq_ignore_ascii_case(url_owner) && self.repo.eq_ignore_ascii_case(&url_repo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pattern_creation() {
        let pattern = RepositoryPattern::new("anthropics/claude-code").unwrap();
        assert_eq!(pattern.owner, "anthropics");
        assert_eq!(pattern.repo, "claude-code");
    }

    #[test]
    fn test_invalid_pattern() {
        assert!(RepositoryPattern::new("invalid").is_err());
        assert!(RepositoryPattern::new("invalid/foo/bar").is_err());
        assert!(RepositoryPattern::new("/").is_err());
        assert!(RepositoryPattern::new("owner/").is_err());
        assert!(RepositoryPattern::new("/repo").is_err());
    }

    #[test]
    fn test_ssh_url_matching() {
        let pattern = RepositoryPattern::new("anthropics/claude-code").unwrap();

        assert!(pattern.matches("git@github.com:anthropics/claude-code.git"));
        assert!(pattern.matches("git@github.com:anthropics/claude-code"));
        assert!(pattern.matches("ssh://git@github.com/anthropics/claude-code.git"));
    }

    #[test]
    fn test_https_url_matching() {
        let pattern = RepositoryPattern::new("anthropics/claude-code").unwrap();

        assert!(pattern.matches("https://github.com/anthropics/claude-code.git"));
        assert!(pattern.matches("https://github.com/anthropics/claude-code"));
        assert!(pattern.matches("http://github.com/anthropics/claude-code.git"));
    }

    #[test]
    fn test_case_insensitive_matching() {
        let pattern = RepositoryPattern::new("Anthropics/Claude-Code").unwrap();

        assert!(pattern.matches("git@github.com:anthropics/claude-code.git"));
        assert!(pattern.matches("https://github.com/ANTHROPICS/CLAUDE-CODE.git"));
    }

    #[test]
    fn test_non_matching_urls() {
        let pattern = RepositoryPattern::new("anthropics/claude-code").unwrap();

        assert!(!pattern.matches("git@github.com:different/repo.git"));
        assert!(!pattern.matches("https://github.com/different/repo.git"));
        assert!(!pattern.matches("git@github.com:anthropics/different-repo.git"));
    }
}
