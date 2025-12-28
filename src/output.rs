use crate::error::Result;
use crate::scanner::MatchResult;
use colored::Colorize;
use serde::Serialize;

#[derive(Serialize)]
struct JsonRepo {
    path: String,
    remotes: Vec<JsonRemote>,
}

#[derive(Serialize)]
struct JsonRemote {
    name: String,
    url: String,
}

#[derive(Serialize)]
struct JsonOutput {
    pattern: String,
    count: usize,
    repositories: Vec<JsonRepo>,
}

/// Print results in human-readable format with colors
pub fn print_results(results: &[MatchResult], pattern: &str) {
    if results.is_empty() {
        println!(
            "{}",
            format!("No repositories found matching '{}'", pattern)
                .yellow()
                .bold()
        );
        return;
    }

    println!(
        "Found {} matching {} for '{}':\n",
        results.len().to_string().green().bold(),
        if results.len() == 1 {
            "repository"
        } else {
            "repositories"
        },
        pattern.cyan()
    );

    for (idx, result) in results.iter().enumerate() {
        println!(
            "{}. {}",
            (idx + 1).to_string().yellow(),
            result.path.display().to_string().bold()
        );

        for (remote_name, url) in &result.remotes {
            println!("   {}: {}", remote_name.blue(), url);
        }

        println!();
    }
}

/// Print results in JSON format
pub fn print_json(results: &[MatchResult], pattern: &str) -> Result<()> {
    let json_output = JsonOutput {
        pattern: pattern.to_string(),
        count: results.len(),
        repositories: results
            .iter()
            .map(|result| JsonRepo {
                path: result.path.display().to_string(),
                remotes: result
                    .remotes
                    .iter()
                    .map(|(name, url)| JsonRemote {
                        name: name.clone(),
                        url: url.clone(),
                    })
                    .collect(),
            })
            .collect(),
    };

    let json_str = serde_json::to_string_pretty(&json_output)?;
    println!("{}", json_str);

    Ok(())
}
