# Gust Architecture

This document describes the internal architecture and design of Gust, a Git-inspired version control system written in Rust.

## Table of Contents

- [Overview](#overview)
- [Directory Structure](#directory-structure)
- [Core Modules](#core-modules)
- [Data Flow](#data-flow)
- [Storage System](#storage-system)
- [Design Patterns](#design-patterns)
- [Error Handling](#error-handling)

## Overview

Gust is built around a content-addressable storage system using SHA-256 hashing. The architecture follows a modular design with clear separation of concerns:

- **CLI Layer**: Command-line interface and argument parsing
- **Project Root**: Central coordinator for all operations
- **Storage Layer**: Serialization, deserialization, and file management
- **Version Control**: Commits, branches, and HEAD management
- **File Tracking**: Staging area and file hashing

## Directory Structure

```
.gust/
├── blobs/              # Content-addressed file storage
│   └── <hash>          # Files stored by their SHA-256 hash
├── commits/            # Commit metadata
│   └── <hash>.json     # Commit objects with tree and metadata
├── branches/           # Branch references
│   ├── main.json       # Default branch
│   ├── <name>.json     # Named branches
│   └── DETACHED_HEAD.json  # Detached HEAD state
├── staging_area.json   # Current staging area (index)
└── HEAD.json          # Current HEAD pointer
```

## Core Modules

### 1. Root (`root.rs`)

The **Root** struct is the central coordinator for all version control operations. It manages:

- **Project root path discovery**: Traverses up the directory tree to find `.gust/`
- **HEAD pointer**: Tracks the current branch or detached commit
- **Staging area**: Manages files staged for commit
- **Ignored files**: Handles `.gustignore` patterns

**Key responsibilities:**
- Project initialization (`create_project`)
- Finding the project root (`find_project_root`)
- Providing access to staging area, HEAD, and last commit
- Coordinating add, remove, commit, checkout, branch, status, and log operations

**Type safety:**
- `RootPath` newtype wrapper ensures only valid project roots are used
- Provides `join()` for safe path construction
- `unsafe_join()` validates that paths exist before returning

### 2. Commit (`commit.rs`)

Commits represent snapshots of the project state.

**Data structures:**
- `Commit`: The in-memory commit object with store path and data
- `CommitRef`: A lightweight reference containing commit ID and metadata
- `CommitMetadata`: Contains commit message/name
- `StorableCommit`: The serialized form with file tree and metadata

**Key features:**
- Content-addressable: Commit ID is SHA-256 hash of its content
- Immutable: Once created, commits cannot be modified
- Tree structure: Maps `RootRelativePath` → `TrackedFile`
- File status detection: Determines if files are Added/Modified/Unchanged

**Operations:**
- `new_commit()`: Creates a new commit from current staging area
- `has_file_changed()`: Compares file with version in commit
- `tree_iterator()`: Iterates over files in commit
- `from_commit_ref()`: Loads commit from reference

### 3. Branch (`branch.rs`)

Branches are named sequences of commits.

**Types:**
- `Branch`: Normal branch with a name
- `DetachedBranch`: Represents a detached HEAD state (checkout by commit hash)

**BranchTrait:**
Common interface for both branch types:
- `commits()`: Access commit history
- `get_last_commit_ref()`: Get the tip of the branch
- `insert()`: Add a new commit to the branch
- `display()`: Format commit history for display
- `handle_checkout()`: Clean up when switching away

**Storage:**
- Branches stored as JSON arrays of `CommitRef`
- DetachedBranch stores tuple of `(Vec<CommitRef>, String)` where String is the passed hash
- Branch files in `.gust/branches/<name>.json`

### 4. Head (`head.rs`)

The HEAD pointer tracks the current location in the commit graph.

**States:**
- `Attached(Branch)`: HEAD points to a branch (normal state)
- `Detached(DetachedBranch)`: HEAD points directly to a commit

**StoredHead:**
Serialized representation:
- `Attached(String)`: Stores branch name
- `Detached`: Indicates detached state (actual data in DETACHED_HEAD.json)

**Operations:**
- `get_tree()`: Get current commit reference
- `insert_commit()`: Add commit to current branch/detached state
- `display()`: Format current branch/commit history
- `handle_checkout()`: Cleanup when switching branches

### 5. StagingArea (`staging_area.rs`)

The staging area (index) tracks files to be included in the next commit.

**Data structure:**
- `HashMap<RootRelativePath, ChangeType>`
- Maps file paths to their change type

**ChangeType enum:**
- `Added`: New file
- `Modified`: Existing file with changes
- `Removed`: File marked for deletion

**Operations:**
- `insert()`: Add/stage a file with change type
- `remove()`: Unstage a file
- `is_empty()`: Check if anything is staged
- `contains()`: Check if specific file is staged
- `get_files()`: Get all staged changes
- `clear()`: Remove all staged changes (after commit)

**Smart cleanup:**
When loaded, automatically removes entries for files that no longer exist (except `Removed` entries).

### 6. TrackedFile (`tracked_file.rs`)

Represents a file tracked by the version control system.

**Components:**
- `blob_id`: SHA-256 hash of file content
- `metadata`: File metadata for quick change detection

**Metadata:**
- `len`: File size
- `modify_time`: Last modification time
- `access_time`: Last access time

**Change detection strategy:**
1. First compare metadata (fast)
2. If metadata changed, compare SHA-256 hashes (slower but accurate)
3. Avoids unnecessary hashing for unchanged files

**Blob storage:**
- Files copied to `.gust/blobs/<hash>`
- Deduplication: identical content stored only once
- Enables efficient storage and fast checkouts

### 7. Storable (`storable.rs`)

Provides serialization/deserialization infrastructure.

**ProjectStorable trait:**
Core trait for all persistable objects:
- `Stored`: The serialized type
- `CreationArgs`: Arguments needed to construct/locate the object
- `build_absolute_path()`: Compute storage path
- `from_stored()`: Deserialize into object
- `into_stored()`: Serialize from object
- `new()`: Create or load object
- `create()`: Create new object and save
- `load()`: Load existing object

**ContainsStorePath trait:**
Extension for objects that store their own path:
- `get_absolute_path()`: Access stored path
- `save()`: Save object to its stored path

**Benefits:**
- Consistent serialization across all types
- Type-safe path construction
- Automatic JSON handling
- Default value support

### 8. Paths (`paths.rs`)

Type-safe path handling with three types:

- `AbsolutePath`: Any absolute path on the filesystem
- `RootRelativePath`: Path relative to project root
- `RootPath`: The project root directory

**Safety features:**
- Newtype pattern prevents path confusion
- `RootPath` can only be created by finding `.gust/`
- Automatic conversions where safe
- Display trait for error messages

### 9. Error (`error.rs`)

Unified error handling using `thiserror`.

**GustError enum:**
- `Io(std::io::Error)`: File system errors
- `ProjectParsing(String)`: Malformed project data
- `User(String)`: User-facing errors (missing file, uncommitted changes, etc.)
- `Json(serde_json::Error)`: JSON serialization errors

**Result type:**
`type Result<T> = std::result::Result<T, GustError>;`

Used throughout the codebase for consistent error propagation.

## Data Flow

### Adding and Committing Files

```
User: gust add file.txt
  ↓
CLI: Parse command
  ↓
Root: add([file.txt])
  ↓
Root: Convert to absolute paths
  ↓
Root: Detect change type (Added/Modified)
  ↓
StagingArea: insert(path, ChangeType)
  ↓
Save staging_area.json
```

```
User: gust commit -m "message"
  ↓
CLI: Parse command
  ↓
Root: commit("message")
  ↓
CommitRef: new_commit(root, metadata)
  ↓
Copy tree from last commit (if exists)
  ↓
For each staged file:
  - Create TrackedFile (copy to blobs/, compute hash)
  - Update tree
  ↓
Compute commit hash (SHA-256 of tree + metadata)
  ↓
Save commit JSON
  ↓
Add CommitRef to current branch
  ↓
Clear staging area
  ↓
Save HEAD and branch
```

### Checkout Flow

```
User: gust checkout branch-name
  ↓
CLI: Parse command
  ↓
Root: checkout(mode, name)
  ↓
Determine if name is branch or commit hash
  ↓
Load target branch/commit
  ↓
Verify no uncommitted changes
  ↓
Get target commit's file tree
  ↓
For each file in tree:
  - Copy from blobs/<hash> to working directory
  ↓
Remove files not in target tree
  ↓
Update HEAD to point to new branch/commit
  ↓
Save HEAD.json
  ↓
Cleanup old DetachedBranch if needed
```

### Status Flow

```
User: gust status
  ↓
CLI: Parse command
  ↓
Root: status()
  ↓
Display current branch name
  ↓
For each staged file:
  - Show path with change type marker (+/M/-)
  ↓
Get last commit (if exists)
  ↓
For each file in working directory:
  - If not staged and changed: show as unstaged
  ↓
For each file in last commit:
  - If not in working directory: show as deleted
  ↓
For each file in working directory:
  - If not tracked and not ignored: show as untracked
```

## Storage System

### Content-Addressable Storage

Gust uses SHA-256 hashing for content addressing:

1. **Blobs**: File contents stored by hash
   - Deduplication: same content = same hash = one copy
   - Immutable: changing content = new hash = new blob

2. **Commits**: Commit metadata stored by hash
   - Commit ID is hash of serialized commit data
   - Includes tree (all files) and metadata
   - Parent commit referenced by ID

3. **Branches**: Mutable references to commits
   - Stored as arrays of CommitRef
   - Branch name maps to file in `.gust/branches/`

### JSON Serialization

All metadata stored as JSON:
- Human-readable for debugging
- Easy to inspect with standard tools
- Uses serde for type-safe serialization

### File System Layout

```
project/
├── .gust/               # VCS metadata (like .git/)
│   ├── blobs/          # Content storage
│   ├── commits/        # Commit objects
│   ├── branches/       # Branch references
│   ├── staging_area.json
│   └── HEAD.json
├── .gustignore         # Ignored patterns (optional)
└── [working files]     # User's actual files
```

## Design Patterns

### 1. Newtype Pattern

Used for type-safe paths:
- `RootPath`, `AbsolutePath`, `RootRelativePath`
- Prevents mixing incompatible path types
- Zero runtime cost

### 2. Trait-Based Polymorphism

- `ProjectStorable` for serialization
- `ContainsStorePath` for objects with paths
- `BranchTrait` for Branch/DetachedBranch

### 3. Content-Addressable Storage

- Objects identified by SHA-256 of content
- Immutable once stored
- Efficient deduplication

### 4. Separation of Concerns

- CLI layer: User interface
- Root: Coordination
- Modules: Specific responsibilities
- Storage: Persistence

### 5. Cow (Clone-on-Write)

Used in `ProjectStorable::into_stored()`:
- Borrowed when possible (no copy)
- Owned when transformation needed
- Optimizes performance

## Error Handling

### Design Principles

1. **User-facing errors**: Clear, actionable messages
2. **Developer errors**: Include context and paths
3. **Error propagation**: Use `?` operator with Result
4. **No panics**: All errors returned as Results

### Error Flow

```
Low-level error (IO, JSON)
  ↓
Wrapped in GustError
  ↓
Propagated with ? operator
  ↓
Caught in CLI command handler
  ↓
Displayed to user (eprintln!)
```

### Examples

```rust
// User error
GustError::User("No uncommitted changes".into())

// Project parsing error
GustError::ProjectParsing(format!("Nonexistent commit at {:?}", path))

// IO error (auto-converted)
fs::read_file(path)?  // Returns GustError::Io on failure
```

## Future Enhancements

Areas for potential improvement:

1. **Merge functionality**: Three-way merge, conflict detection
2. **Remote repositories**: Push, pull, fetch operations
3. **Packed storage**: Compress blob storage
4. **Ref logs**: Track HEAD movement history
5. **Tags**: Lightweight and annotated tags
6. **Garbage collection**: Remove unreachable objects
7. **Performance**: Parallel file operations, index caching
8. **Delta compression**: Store diffs instead of full files

## Implementation Notes

### Why SHA-256?

- Cryptographically secure
- Collision resistance
- Standard in modern VCS
- Fast enough for typical repositories

### Why JSON?

- Human-readable for debugging
- Standard tooling support
- Easy to extend
- Acceptable performance for metadata

### Why Rust?

- Memory safety without garbage collection
- Strong type system prevents bugs
- Zero-cost abstractions
- Excellent error handling with Result
- Fast enough for CLI tools

## Testing

Gust includes comprehensive integration tests (`tests/integration_tests.rs`):

- Tests run in isolated temporary directories
- Full workflow testing (init, add, commit, branch, checkout)
- Edge cases: uncommitted changes, detached HEAD, collisions
- Uses actual binary for realistic testing

See the test file for examples of expected behavior and usage patterns.
