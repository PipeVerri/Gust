# Gust

A lightweight, Git-inspired version control system written in Rust.

## Description

Gust is a version control system that implements core Git-like functionality including commits, branches, staging, and checkout operations. Built with Rust for performance and reliability, Gust uses SHA-256 hashing for content addressing and supports essential version control workflows.

## Installation

### Prerequisites

- Rust 2024 edition or later
- Cargo (comes with Rust)

### Build from Source

```bash
# Clone the repository
git clone <repository-url>
cd Gust

# Build in release mode
cargo build --release

# The binary will be available at ./target/release/Gust
```

### Add to PATH (Optional)

```bash
# Linux/macOS
export PATH="$PATH:/path/to/Gust/target/release"

# Or copy to a directory in your PATH
sudo cp target/release/Gust /usr/local/bin/
```

## Usage

### Initialize a Repository

```bash
# Create a new Gust repository in the current directory
Gust init
```

This creates a `.gust` directory with the following structure:
- `blobs/` - Stores file contents
- `commits/` - Stores commit metadata
- `branches/` - Stores branch information

### Basic Workflow

```bash
# Create and add files
echo "Hello, Gust!" > file.txt
Gust add file.txt

# Commit changes
Gust commit -m "Initial commit"

# Check repository status
Gust status

# View commit history
Gust log
```

### Working with Branches

```bash
# Create a new branch
Gust branch feature-branch

# Switch to a branch
Gust checkout feature-branch

# Make changes and commit
echo "New feature" > feature.txt
Gust add feature.txt
Gust commit -m "Add new feature"

# Switch back to main
Gust checkout main
```

### Checkout by Commit Hash

```bash
# View commit history to get hash
Gust log

# Checkout a specific commit (detached HEAD state)
Gust checkout <commit-hash> --mode commit
```

## Available Commands

### `init`
Initialize a new Gust repository in the current directory.

```bash
Gust init
```

### `add <paths...>`
Add files to the staging area.

```bash
Gust add file1.txt file2.txt
Gust add *.rs
```

### `rm <paths...>`
Remove files from the staging area and mark them for deletion.

```bash
Gust rm file.txt
```

### `commit -m <message>`
Create a new commit with staged changes.

```bash
Gust commit -m "Your commit message"
Gust commit --message "Your commit message"
```

### `status`
Show the working tree status, including:
- Staged files (added/modified/removed)
- Unstaged changes
- Untracked files

```bash
Gust status
```

### `log`
Display commit history for the current branch.

```bash
Gust log
```

### `branch [name]`
List branches or create a new branch.

```bash
# List all branches
Gust branch

# Create a new branch
Gust branch feature-name
```

### `checkout <name> [--mode <commit|branch>]`
Switch branches or checkout a specific commit.

```bash
# Switch to a branch
Gust checkout main

# Checkout a commit (requires --mode flag if ambiguous)
Gust checkout abc123 --mode commit

# Checkout a branch explicitly
Gust checkout feature --mode branch
```

**Note:** Checkout will fail if you have uncommitted changes in your working directory.

## Build and Testing

### Run Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_init_creates_gust_directory
```

### Build

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Check without building
cargo check
```

### Run in Development

```bash
# Run directly with cargo
cargo run -- init
cargo run -- add file.txt
cargo run -- commit -m "Test commit"
```

## Project Structure

```
Gust/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # CLI argument parsing with clap
│   ├── project.rs           # Module exports
│   └── project/
│       ├── root.rs          # Project root management and main operations
│       ├── commit.rs        # Commit data structure and operations
│       ├── branch.rs        # Branch management
│       ├── head.rs          # HEAD pointer management
│       ├── staging_area.rs  # Staging area (index) management
│       ├── tracked_file.rs  # File tracking and hashing
│       ├── paths.rs         # Path utilities
│       ├── storable.rs      # Serialization/deserialization trait
│       └── error.rs         # Error types and handling
├── tests/
│   └── integration_tests.rs # End-to-end integration tests
├── Cargo.toml               # Project dependencies and metadata
└── .github/
    └── workflows/
        └── rust.yml         # CI/CD pipeline
```

### Key Modules

- **Root**: Manages the project root, staging area, and HEAD pointer
- **Commit**: Represents a commit with message, timestamp, parent, and file tree
- **Branch**: Manages branch creation and references
- **StagingArea**: Handles the staging area (index) for tracking changes
- **Head**: Manages the HEAD pointer (current branch or commit)
- **TrackedFile**: Represents files tracked by the VCS with content hashing

## Continuous Integration

This project uses GitHub Actions for automated testing and building. On every push or pull request to the `master` branch:

1. Builds the project in release mode
2. Runs the full test suite

See `.github/workflows/rust.yml` for the CI configuration.

## Dependencies

- **clap** (4.5.57) - Command-line argument parsing
- **serde** (1.0.228) - Serialization framework
- **serde_json** (1.0.149) - JSON serialization
- **sha256** (1.6.0) - SHA-256 hashing for content addressing
- **thiserror** (2.0.18) - Error handling

## Storage Format

Gust stores all version control data in the `.gust` directory:

- **blobs/**: Content-addressed file storage using SHA-256 hashes
- **commits/**: JSON files containing commit metadata
- **branches/**: JSON files for each branch with commit references
- **staging_area.json**: Current staging area state
- **head.json**: Current HEAD pointer (branch or commit)

## Limitations and Future Work

Current limitations:
- No merge functionality
- No remote repository support (push/pull/fetch)
- No conflict resolution
- Detached HEAD state is supported but limited

For detailed architecture information, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

For comprehensive command documentation, see [docs/COMMANDS.md](docs/COMMANDS.md).
