# fsgitwatch

A high-performance async Rust CLI tool for finding git repositories in your filesystem by owner/repo pattern.

## Origin

This project was built using [Claude Code](https://claude.com/claude-code) with the following prompt:

> I want to create a CLI that lets me search for a particular "repository" in the given folders across sub-folders. This happens with me a lot of times where I'm searching for a given github repository, but idk where it was cloned in my filesystem. Ideally, I can cd to each folder and try doing git remote -v to see if one of the remote URL contains the information around that repository. Please note that each repository could be cloned via http as well and/or ssh protocol, so the format for both could be different but they are essentially same repositories. I want to make this async, so that no time is spent in iterating through the folders. I also want to ensure that I don't iterate through subfolders within a "Github repository" that didn't match my repository - as that would just be wasting my time. Plan the work accordingly, can we please build this in Rust?

## Features

- **Owner/Repo Pattern Matching**: Search using format like `anthropics/claude-code`
- **Multi-Protocol Support**: Handles both SSH (`git@github.com:owner/repo.git`) and HTTPS (`https://github.com/owner/repo.git`) URLs
- **All Remotes Checked**: Scans origin, upstream, and all configured remotes
- **Async Performance**: Uses Tokio with bounded parallelism (100 concurrent tasks by default)
- **Smart Pruning**: When a `.git` directory is found, subdirectories are NOT scanned (10-100x speedup)
- **Multiple Output Formats**: Human-readable colored output or JSON (`--json`)
- **Verbose Mode**: Optional debugging output with `-v`
- **Flexible Search Path**: Defaults to current directory, accepts custom path

## Installation

```bash
# Clone the repository
git clone https://github.com/krshrimali/fsgitwatch
cd fsgitwatch

# Build in release mode
cargo build --release

# Install system-wide (optional)
cargo install --path .
```

## Usage

```bash
# Search current directory for a repository
fsgitwatch anthropics/claude-code

# Search specific directory
fsgitwatch anthropics/claude-code ~/projects

# Increase parallelism for large directories
fsgitwatch -j 200 user/repo ~/

# JSON output for scripting
fsgitwatch --json user/repo

# Verbose mode to see warnings
fsgitwatch -v user/repo
```

### Command-Line Options

```
Usage: fsgitwatch [OPTIONS] <PATTERN> [PATH]

Arguments:
  <PATTERN>  Repository pattern in owner/repo format (e.g., 'anthropics/claude-code')
  [PATH]     Directory to search (defaults to current directory)

Options:
  -j, --max-concurrent <MAX_CONCURRENT>  Maximum number of concurrent scan tasks [default: 100]
      --json                             Output results as JSON
  -v, --verbose                          Verbose output (show warnings and debugging information)
  -h, --help                            Print help
  -V, --version                         Print version
```

## How It Works

1. **Pattern Parsing**: Parses the `owner/repo` pattern from command line
2. **Async Directory Traversal**: Uses Tokio to asynchronously scan directories with bounded parallelism
3. **Git Detection**: When a `.git` directory is found, reads all remote URLs using git2-rs
4. **URL Normalization**: Normalizes both SSH and HTTPS URLs to extract owner/repo information
5. **Pattern Matching**: Compares extracted owner/repo with the search pattern (case-insensitive)
6. **Smart Pruning**: Once a git repository is found (match or no match), stops scanning subdirectories

## Architecture

### Key Components

- **scanner.rs**: Async directory traversal with semaphore-based concurrency control
- **git.rs**: Git remote extraction using git2-rs with `spawn_blocking`
- **matcher.rs**: URL normalization and pattern matching using git-url-parse
- **cli.rs**: Command-line argument parsing with clap
- **output.rs**: Result formatting (colored terminal or JSON)
- **error.rs**: Custom error types with thiserror

### Performance Optimizations

1. **Bounded Parallelism**: Uses semaphore to limit concurrent tasks (prevents resource exhaustion)
2. **Directory Pruning**: Early return when git repository found (avoids scanning `.git/`, `node_modules/`, etc.)
3. **Async I/O**: Non-blocking filesystem operations with Tokio
4. **Efficient Git Operations**: Uses git2-rs with spawn_blocking for git operations

### Expected Performance

- Small projects (~10 directories): <100ms
- Large codebases (~1000 directories): 1-3 seconds
- Entire home directory (~10,000 directories): 10-30 seconds

## Examples

### Finding a repository you cloned somewhere

```bash
$ fsgitwatch torvalds/linux ~/projects
Found 1 matching repository for 'torvalds/linux':

1. /Users/you/projects/kernels/linux
   origin: git@github.com:torvalds/linux.git
```

### Finding all clones of a repository

```bash
$ fsgitwatch anthropics/claude-code ~
Found 3 matching repositories for 'anthropics/claude-code':

1. /Users/you/work/claude-code
   origin: https://github.com/anthropics/claude-code.git

2. /Users/you/projects/ai/claude-code
   origin: git@github.com:anthropics/claude-code.git
   upstream: https://github.com/anthropics/claude-code.git

3. /Users/you/tmp/claude-code-test
   origin: https://github.com/anthropics/claude-code.git
```

### JSON output for scripting

```bash
$ fsgitwatch --json user/repo ~/projects
{
  "pattern": "user/repo",
  "count": 1,
  "repositories": [
    {
      "path": "/Users/you/projects/repo",
      "remotes": [
        {
          "name": "origin",
          "url": "https://github.com/user/repo.git"
        }
      ]
    }
  ]
}
```

## Dependencies

- **tokio**: Async runtime
- **clap**: CLI argument parsing
- **git2**: Git operations (libgit2 bindings)
- **git-url-parse**: URL parsing and normalization
- **anyhow/thiserror**: Error handling
- **colored**: Terminal colors
- **serde/serde_json**: JSON serialization

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
