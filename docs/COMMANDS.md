# Gust Commands Reference

Complete reference for all Gust commands with detailed examples and use cases.

## Table of Contents

- [init](#init)
- [add](#add)
- [rm](#rm)
- [commit](#commit)
- [status](#status)
- [log](#log)
- [branch](#branch)
- [checkout](#checkout)
- [Common Workflows](#common-workflows)

---

## init

Initialize a new Gust repository in the current directory.

### Syntax

```bash
Gust init
```

### Description

Creates a `.gust` directory with the following structure:
- `blobs/` - Content-addressable file storage
- `commits/` - Commit metadata storage
- `branches/` - Branch references (creates default `main` branch)
- `HEAD.json` - Points to current branch
- `staging_area.json` - Staging area state

### Examples

```bash
# Create a new project
mkdir my-project
cd my-project
Gust init

# Verify initialization
ls -la .gust
```

### Notes

- Cannot initialize if `.gust` already exists in current or parent directories
- Creates an empty repository with no commits
- Default branch is named `main`

### Exit Codes

- `0` - Success
- `1` - Already initialized or I/O error

---

## add

Add file(s) to the staging area for the next commit.

### Syntax

```bash
Gust add <path> [<path>...]
```

### Arguments

- `<path>` - One or more file paths to stage (relative or absolute)

### Description

Stages files for inclusion in the next commit. Files are analyzed to determine their change type:
- **Added**: New file not in the last commit
- **Modified**: Existing file with changes
- The actual file content is copied to `.gust/blobs/<hash>` when committed

### Examples

```bash
# Add single file
Gust add README.md

# Add multiple files
Gust add src/main.rs src/lib.rs

# Add with wildcards (shell expansion)
Gust add src/*.rs
Gust add *.txt

# Add file with spaces in name
Gust add "My File.txt"

# Add from different directory (relative path)
cd subdir
Gust add ../file.txt
```

### Notes

- Paths must be within the project root (above or at `.gust` level)
- Staging the same file twice updates its entry in the staging area
- Change type detection happens at commit time based on last commit
- Files are content-addressed by SHA-256 hash
- Duplicate content is stored only once (deduplication)

### Errors

- **File not found**: Path doesn't exist
- **Outside project**: Path is not within the project root
- **No project**: Run `Gust init` first

### Exit Codes

- `0` - Success
- `1` - Error (file not found, outside project, etc.)

---

## rm

Remove file(s) from the staging area and mark for deletion.

### Syntax

```bash
Gust rm <path> [<path>...]
```

### Arguments

- `<path>` - One or more file paths to remove (relative or absolute)

### Description

Marks files for removal in the next commit. The file is:
1. Removed from the staging area if present
2. Marked with `ChangeType::Removed` so the next commit records the deletion

This does NOT delete the file from your working directory (unlike `git rm`).

### Examples

```bash
# Remove single file
Gust rm old_file.txt

# Remove multiple files
Gust rm file1.txt file2.txt

# Remove with wildcards
Gust rm *.log
```

### Notes

- Does not delete files from working directory (only marks for deletion in VCS)
- To actually delete from filesystem, use `rm` command separately
- Removing a file that wasn't staged does nothing (no error)

### Workflow

```bash
# Mark file for removal in VCS
Gust rm config.txt

# Commit the removal
Gust commit -m "Remove old config"

# Optionally delete from filesystem
rm config.txt
```

### Exit Codes

- `0` - Success (even if file wasn't staged)
- `1` - Error (outside project, no project)

---

## commit

Create a new commit with staged changes.

### Syntax

```bash
Gust commit -m <message>
Gust commit --message <message>
```

### Options

- `-m, --message <message>` - Commit message (required)

### Description

Creates a new commit containing all staged changes. The commit:
1. Copies the tree from the last commit (if exists)
2. Applies staged changes (Added/Modified/Removed)
3. Computes commit hash (SHA-256 of tree + metadata)
4. Saves commit metadata to `.gust/commits/<hash>.json`
5. Updates current branch to point to new commit
6. Clears the staging area

### Examples

```bash
# Simple commit
Gust commit -m "Add login feature"

# Commit with longer message
Gust commit -m "Fix authentication bug in user service"

# Empty message (allowed but not recommended)
Gust commit -m ""

# Multi-word message
Gust commit -m "This is a longer commit message with spaces"
```

### Commit ID

The commit ID is the SHA-256 hash of the commit's serialized content:
- Includes file tree (all tracked files with their hashes)
- Includes commit metadata (message)
- Deterministic: same content = same hash

### Notes

- Must have changes staged (use `Gust add` first)
- Commit messages can be empty (default: `""`)
- In detached HEAD state, shows warning about untracked changes
- Original file metadata (size, timestamps) is preserved in TrackedFile

### Errors

- **Nothing staged**: No files in staging area
- **No project**: Run `Gust init` first

### Exit Codes

- `0` - Success
- `1` - Error (nothing staged, no project)

---

## status

Show the working tree status.

### Syntax

```bash
Gust status
```

### Description

Displays:
1. **Current branch or commit** (HEAD pointer)
2. **Staged changes**: Files added to the staging area
   - `+` - Added (new file)
   - `M` - Modified (changed existing file)
   - `-` - Removed (deleted file)
3. **Unstaged changes**: Modified files not staged
4. **Untracked files**: New files not in staging area or last commit

### Output Format

```
On branch: <branch-name>
or
On commit: <commit-hash> (detached HEAD)

Staged changes:
  [+/-/M] path/to/file

Unstaged changes:
  M path/to/modified/file
  - path/to/deleted/file

Untracked files:
  path/to/new/file
```

### Examples

```bash
# Check status after adding files
Gust add README.md
Gust status
# Output:
# On branch: main
# Staged changes:
#   + README.md

# Check status with unstaged changes
echo "new content" >> file.txt
Gust status
# Output:
# On branch: main
# Unstaged changes:
#   M file.txt

# Check status in detached HEAD
Gust checkout abc123 --mode commit
Gust status
# Output:
# On commit: abc123 (detached HEAD)
```

### Change Detection

- **Staged**: Files in the staging area
- **Unstaged Modified**: Files changed since last commit but not staged
- **Deleted**: Files in last commit but missing from working directory
- **Untracked**: Files in working directory but not tracked or staged

### Notes

- Works in both attached (branch) and detached (commit) HEAD states
- Respects `.gustignore` patterns for untracked files
- Comparing files uses metadata first (fast), then hash (slow)

### Exit Codes

- `0` - Always succeeds if project exists
- `1` - No project found

---

## log

Display commit history for the current branch.

### Syntax

```bash
Gust log
```

### Description

Shows the commit history of the current branch or detached HEAD:
- Lists commits in reverse chronological order (newest first)
- Each commit shows: message and full commit hash
- In detached HEAD, shows history of that specific commit

### Output Format

```
Commit history of <branch-name> branch:
<message>: <commit-hash>
<message>: <commit-hash>
...

or

Commit history of detached HEAD(commit <hash>):
<message>: <commit-hash>
...
```

### Examples

```bash
# View commit history
Gust log
# Output:
# Commit history of main branch:
# Add feature: 5f3a9c8b2d1e6f4a7c8b9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0
# Initial commit: 1a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b1c2d3e4f5a6b7c8d9e0f1

# View log after checkout to specific commit
Gust checkout abc123 --mode commit
Gust log
# Output:
# Commit history of detached HEAD(commit abc123):
# First commit: abc123...
```

### Notes

- Shows complete commit hashes (64 characters for SHA-256)
- Newest commits appear first
- Empty history if no commits exist
- In detached HEAD, shows the commit chain leading to current commit

### Exit Codes

- `0` - Success
- `1` - No project found

---

## branch

List branches or create a new branch.

### Syntax

```bash
# List all branches
Gust branch

# Create new branch
Gust branch <branch-name>
```

### Arguments

- `<branch-name>` - (Optional) Name of the branch to create

### Description

**Without arguments**: Lists all existing branches
- Shows branch names from `.gust/branches/`
- Current branch indicated by HEAD

**With argument**: Creates a new branch
- Branch points to the same commit as current HEAD
- Does NOT switch to the new branch (use `checkout` to switch)
- Branch is saved to `.gust/branches/<name>.json`

### Examples

```bash
# List branches
Gust branch
# Output: main, feature, bugfix

# Create new branch
Gust branch feature-login

# Create branch and switch to it
Gust branch new-feature
Gust checkout new-feature

# Verify branch was created
Gust branch
# Output: main, new-feature, feature-login
```

### Branch Naming

- Can contain letters, numbers, hyphens, underscores
- Avoid names that look like commit hashes (e.g., `abc123`)
- Case-sensitive on Unix systems

### Notes

- New branch shares history with source branch/commit
- Branch is a pointer to a commit, not a copy of files
- Creating branch from detached HEAD is allowed

### Errors

- **Branch exists**: Name already used
- **No project**: Run `Gust init` first
- **No commits**: Must have at least one commit

### Exit Codes

- `0` - Success
- `1` - Error (branch exists, no project, no commits)

---

## checkout

Switch branches or checkout a specific commit.

### Syntax

```bash
# Switch to a branch
Gust checkout <branch-name>

# Checkout a commit (detached HEAD)
Gust checkout <commit-hash> --mode commit

# Explicit branch checkout
Gust checkout <name> --mode branch

# Auto-detect (if unambiguous)
Gust checkout <name>
```

### Arguments

- `<name>` - Branch name or commit hash

### Options

- `--mode <commit|branch>` - Explicitly specify what to checkout
  - `commit` - Checkout commit by hash (detached HEAD)
  - `branch` - Checkout branch by name
  - If omitted, auto-detects based on name

### Description

Switches the working directory to a different branch or commit:

1. **Verifies no uncommitted changes** (would be lost on checkout)
2. **Loads target branch/commit**
3. **Updates working directory files** from target commit's tree
4. **Updates HEAD** to point to new branch/commit

### Detached HEAD State

When checking out a commit directly:
- HEAD points to commit, not a branch
- New commits are not saved to any branch
- Warning shown when committing in detached state
- Can create a new branch to save work

### Examples

```bash
# Switch to existing branch
Gust checkout main

# Create and switch to new branch
Gust branch feature-x
Gust checkout feature-x

# Checkout specific commit (creates detached HEAD)
Gust log  # Get commit hash
Gust checkout 5f3a9c8 --mode commit

# Return from detached HEAD to branch
Gust checkout main

# Resolve ambiguity (if branch and commit have same name)
Gust checkout abc123 --mode branch  # Checkout branch named "abc123"
Gust checkout abc123 --mode commit  # Checkout commit abc123...
```

### Commit Hash Prefix

- Can use shortened hash (first 7+ characters)
- Must be unambiguous (unique prefix)

### Naming Collision

If a branch name matches a commit hash prefix:
```bash
# Error: ambiguous name
Gust checkout abc123
# Error: both branch 'abc123' and commit 'abc123...' exist
# Specify with --mode

# Explicit checkout
Gust checkout abc123 --mode branch
```

### File Restoration

Checkout restores files from the target commit:
1. Copies each file from `.gust/blobs/<hash>` to working directory
2. Deletes files not in target commit
3. Preserves metadata (timestamps, size)

### Safety

- **Prevents data loss**: Blocks checkout if uncommitted changes exist
- **Clean working directory required**: All changes must be staged and committed

### Errors

- **Uncommitted changes**: Working directory has modifications
  - Solution: Commit or stash changes first
- **Branch/commit not found**: Name doesn't match any branch or commit
- **Ambiguous name**: Both branch and commit exist with same name
  - Solution: Use `--mode` flag
- **No project**: Run `Gust init` first

### Exit Codes

- `0` - Success
- `1` - Error (uncommitted changes, not found, ambiguous, no project)

---

## Common Workflows

### Starting a New Project

```bash
# Initialize repository
mkdir my-project
cd my-project
Gust init

# Create initial files
echo "# My Project" > README.md
echo "fn main() {}" > src/main.rs

# Add and commit
Gust add README.md src/main.rs
Gust commit -m "Initial commit"
```

### Feature Branch Workflow

```bash
# Start from main
Gust checkout main

# Create and switch to feature branch
Gust branch feature-auth
Gust checkout feature-auth

# Make changes
echo "auth code" > auth.rs
Gust add auth.rs
Gust commit -m "Add authentication"

# Continue working
echo "more auth" >> auth.rs
Gust add auth.rs
Gust commit -m "Improve auth security"

# Return to main (feature complete)
Gust checkout main
```

### Inspecting Changes

```bash
# Check what's changed
Gust status

# Review commit history
Gust log

# Check what's staged
Gust status

# See available branches
Gust branch
```

### Undoing Changes (via Checkout)

```bash
# View commit history
Gust log

# Checkout old commit to inspect
Gust checkout abc123 --mode commit

# Look around (read-only)
cat old-file.txt
Gust log

# Return to branch
Gust checkout main
```

### Recovering from Detached HEAD

```bash
# Made commits in detached HEAD
Gust checkout abc123 --mode commit
# ... make changes ...
Gust add file.txt
Gust commit -m "Experimental change"

# Save work by creating branch
Gust branch experiment

# Switch to new branch
Gust checkout experiment

# Now commits are saved on 'experiment' branch
```

### Handling Uncommitted Changes

```bash
# Scenario: Want to checkout but have changes

# Check status
Gust status
# Output: Unstaged changes: M file.txt

# Option 1: Commit changes
Gust add file.txt
Gust commit -m "WIP: save progress"
Gust checkout other-branch

# Option 2: Discard changes (careful!)
# Manually revert files or use git commands
# Then checkout
Gust checkout other-branch
```

### Multiple Files Workflow

```bash
# Stage multiple files at once
Gust add src/main.rs src/lib.rs tests/test.rs

# Remove multiple files
Gust rm old1.txt old2.txt

# Commit all changes
Gust commit -m "Refactor codebase"

# Check what was committed
Gust log
```

### Branch Management

```bash
# List all branches
Gust branch

# Create multiple branches for different features
Gust branch feature-ui
Gust branch feature-backend
Gust branch bugfix-auth

# Switch between branches
Gust checkout feature-ui
# Work on UI...
Gust checkout feature-backend
# Work on backend...

# Return to main
Gust checkout main
```

### Working with Removed Files

```bash
# Remove file from version control
Gust rm deprecated.txt
Gust commit -m "Remove deprecated file"

# File still exists in old commits
Gust log  # Get old commit hash
Gust checkout abc123 --mode commit
cat deprecated.txt  # File restored from history
Gust checkout main  # File gone again
```

---

## Tips and Best Practices

### Commit Messages

- **Be descriptive**: "Add user authentication" not "Update files"
- **Use imperative mood**: "Fix bug" not "Fixed bug"
- **Keep it concise**: One line summary is often enough

### Branching Strategy

- **main**: Stable, tested code
- **feature-***: New features in development
- **bugfix-***: Bug fixes
- **experiment-***: Experimental changes

### Before Checkout

Always check status to avoid losing work:
```bash
Gust status
# If changes exist, commit or stash them
Gust add .
Gust commit -m "Save progress"
# Now safe to checkout
Gust checkout other-branch
```

### Regular Commits

- Commit early and often
- Each commit should be a logical unit of work
- Makes history easier to navigate with `Gust log`

### Detached HEAD

- Use for read-only inspection of old commits
- If making changes, immediately create a branch
- Don't commit multiple times in detached HEAD without a branch

---

## Comparison with Git

| Feature | Gust | Git |
|---------|------|-----|
| Initialize | `Gust init` | `git init` |
| Stage files | `Gust add <files>` | `git add <files>` |
| Commit | `Gust commit -m "msg"` | `git commit -m "msg"` |
| Status | `Gust status` | `git status` |
| History | `Gust log` | `git log` |
| Branches | `Gust branch` | `git branch` |
| Switch | `Gust checkout <name>` | `git checkout <name>` or `git switch <name>` |
| Remove | `Gust rm <files>` | `git rm --cached <files>` (Gust doesn't delete from disk) |
| Merge | Not supported | `git merge` |
| Remotes | Not supported | `git push/pull/fetch` |

### Key Differences

1. **No merge**: Gust doesn't support merging branches
2. **No remotes**: All operations are local
3. **Simpler rm**: Doesn't delete from working directory
4. **JSON storage**: Metadata in human-readable JSON
5. **SHA-256**: Uses SHA-256 instead of SHA-1

---

## Troubleshooting

### "No project found"

**Problem**: Command fails with "No project found"

**Solution**:
```bash
# Ensure you're in or below the project directory
pwd
ls -la .gust  # Should exist

# If not, initialize
Gust init
```

### "Uncommitted changes"

**Problem**: Checkout blocked due to uncommitted changes

**Solution**:
```bash
# Check what's changed
Gust status

# Commit changes
Gust add changed-file.txt
Gust commit -m "Save work"

# Now checkout
Gust checkout other-branch
```

### "Ambiguous name" on Checkout

**Problem**: Branch and commit share the same name

**Solution**:
```bash
# Explicitly specify
Gust checkout name --mode branch
# or
Gust checkout name --mode commit
```

### "Nothing staged" on Commit

**Problem**: Tried to commit without staging files

**Solution**:
```bash
# Stage files first
Gust add file.txt

# Then commit
Gust commit -m "message"
```

### Files not restored on Checkout

**Problem**: Files missing after checkout

**Possible causes**:
- Files were never committed on that branch
- Files were removed in a commit
- File ignored by `.gustignore`

**Solution**:
```bash
# Check commit history
Gust log

# Check if file exists in commit tree
# (requires inspecting `.gust/commits/<hash>.json`)
```

---

## Advanced Usage

### Inspecting Internal State

```bash
# View current HEAD
cat .gust/HEAD.json

# View branch commit history
cat .gust/branches/main.json

# View staging area
cat .gust/staging_area.json

# View commit details
cat .gust/commits/<hash>.json

# List all commits
ls .gust/commits/

# List all branches
ls .gust/branches/

# List all blobs
ls .gust/blobs/
```

### Manual File Recovery

```bash
# Get file hash from commit
cat .gust/commits/<commit-hash>.json
# Find file entry with blob_id

# Copy blob to working directory
cp .gust/blobs/<blob-hash> recovered-file.txt
```

### Ignoring Files

Create `.gustignore` in project root:
```
# Patterns to ignore
target/
*.log
.env
node_modules/
```

Note: `.gustignore` support may be limited depending on implementation.

---

For architecture details and internals, see [ARCHITECTURE.md](ARCHITECTURE.md).

For project overview and installation, see [README.md](../README.md).
