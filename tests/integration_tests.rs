use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::env;

/// Helper struct to manage a temporary test directory
struct TestRepo {
    path: PathBuf,
}

impl TestRepo {
    fn new(name: &str) -> Self {
        let test_dir = env::temp_dir().join(format!("gust_test_{}", name));
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).unwrap();
        }
        fs::create_dir_all(&test_dir).unwrap();

        TestRepo { path: test_dir }
    }

    fn run_gust(&self, args: &[&str]) -> std::process::Output {
        Command::new(env!("CARGO_BIN_EXE_Gust"))
            .current_dir(&self.path)
            .args(args)
            .output()
            .expect("Failed to execute gust command")
    }

    fn create_file(&self, name: &str, content: &str) {
        fs::write(self.path.join(name), content).unwrap();
    }

    #[allow(dead_code)]
    fn file_exists(&self, name: &str) -> bool {
        self.path.join(name).exists()
    }

    fn read_file(&self, name: &str) -> String {
        fs::read_to_string(self.path.join(name)).unwrap()
    }

    fn gust_dir_exists(&self) -> bool {
        self.path.join(".gust").exists()
    }
}

impl Drop for TestRepo {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_init_creates_gust_directory() {
    let repo = TestRepo::new("init");

    let output = repo.run_gust(&["init"]);
    assert!(output.status.success(), "init command failed");
    assert!(repo.gust_dir_exists(), ".gust directory was not created");
    assert!(repo.path.join(".gust/blobs").exists());
    assert!(repo.path.join(".gust/commits").exists());
    assert!(repo.path.join(".gust/branches").exists());
}

#[test]
fn test_add_and_status() {
    let repo = TestRepo::new("add_status");

    repo.run_gust(&["init"]);
    repo.create_file("test.txt", "hello world");

    let output = repo.run_gust(&["add", "test.txt"]);
    assert!(output.status.success(), "add command failed");

    let status_output = repo.run_gust(&["status"]);
    assert!(status_output.status.success(), "status command failed");
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(status_str.contains("test.txt"), "status should show added file");
}

#[test]
fn test_commit_workflow() {
    let repo = TestRepo::new("commit");

    repo.run_gust(&["init"]);
    repo.create_file("file1.txt", "content1");
    repo.run_gust(&["add", "file1.txt"]);

    let output = repo.run_gust(&["commit", "-m", "Initial commit"]);
    assert!(output.status.success(), "commit command failed");

    let log_output = repo.run_gust(&["log"]);
    assert!(log_output.status.success(), "log command failed");
    let log_str = String::from_utf8_lossy(&log_output.stdout);
    assert!(log_str.contains("Initial commit"), "log should show commit message");
}

#[test]
fn test_remove_file_detection() {
    let repo = TestRepo::new("remove");

    repo.run_gust(&["init"]);
    repo.create_file("file1.txt", "content");
    repo.run_gust(&["add", "file1.txt"]);
    repo.run_gust(&["commit", "-m", "Add file"]);

    // Delete the file from filesystem
    fs::remove_file(repo.path.join("file1.txt")).unwrap();

    let status_output = repo.run_gust(&["status"]);
    let status_str = String::from_utf8_lossy(&status_output.stdout);
    assert!(status_str.contains("file1.txt"), "status should detect removed file");
    assert!(status_str.contains("-"), "status should show file as removed");
}

#[test]
fn test_branch_creation() {
    let repo = TestRepo::new("branch_create");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "content");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Initial"]);

    let output = repo.run_gust(&["branch", "feature"]);
    assert!(output.status.success(), "branch creation failed");

    assert!(repo.path.join(".gust/branches/feature.json").exists(), "branch file was not created");
}

#[test]
fn test_branch_checkout() {
    let repo = TestRepo::new("branch_checkout");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "original");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Initial"]);

    repo.run_gust(&["branch", "feature"]);

    // Modify file on main
    repo.create_file("file.txt", "modified");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Modified"]);

    // Checkout feature branch
    let output = repo.run_gust(&["checkout", "feature"]);
    assert!(output.status.success(), "checkout failed");

    // File should have original content
    let content = repo.read_file("file.txt");
    assert_eq!(content, "original", "file content should be from feature branch");
}

#[test]
fn test_checkout_commit_by_hash() {
    let repo = TestRepo::new("checkout_commit");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "v1");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "First"]);

    // Get first commit hash from log
    let log_output = repo.run_gust(&["log"]);
    let log_str = String::from_utf8_lossy(&log_output.stdout);
    let first_hash: String = log_str
        .lines()
        .find(|line| line.contains("First"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().chars().take(7).collect())
        .expect("Could not extract commit hash");

    repo.create_file("file.txt", "v2");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Second"]);

    // Checkout first commit
    let output = repo.run_gust(&["checkout", &first_hash, "--mode", "commit"]);
    assert!(output.status.success(), "checkout commit failed");

    let content = repo.read_file("file.txt");
    assert_eq!(content, "v1", "file should have v1 content after checkout");
}

#[test]
fn test_checkout_naming_collision_error() {
    let repo = TestRepo::new("checkout_collision");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "content");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Initial"]);

    // Get commit hash
    let log_output = repo.run_gust(&["log"]);
    let log_str = String::from_utf8_lossy(&log_output.stdout);
    let commit_hash: String = log_str
        .lines()
        .find(|line| line.contains("Initial"))
        .and_then(|line| line.split(':').nth(1))
        .map(|s| s.trim().chars().take(7).collect())
        .expect("Could not extract commit hash");

    // Create a branch with same prefix as commit hash
    repo.run_gust(&["branch", &commit_hash]);

    // Try to checkout without specifying mode - should produce error
    let output = repo.run_gust(&["checkout", &commit_hash]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("same name") || stderr.contains("Specify if you want to checkout"),
        "error message should mention naming collision: {}",
        stderr
    );
}

#[test]
fn test_checkout_with_uncommitted_changes_fails() {
    let repo = TestRepo::new("checkout_uncommitted");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "content");
    repo.run_gust(&["add", "file.txt"]);
    repo.run_gust(&["commit", "-m", "Initial"]);

    repo.run_gust(&["branch", "feature"]);

    // Make uncommitted changes
    repo.create_file("file.txt", "modified");

    let output = repo.run_gust(&["checkout", "feature"]);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("uncommitted changes"),
        "error should mention uncommitted changes: {}",
        stderr
    );
}

#[test]
fn test_multiple_files_workflow() {
    let repo = TestRepo::new("multiple_files");

    repo.run_gust(&["init"]);
    repo.create_file("file1.txt", "content1");
    repo.create_file("file2.txt", "content2");
    repo.create_file("file3.txt", "content3");

    repo.run_gust(&["add", "file1.txt", "file2.txt", "file3.txt"]);
    repo.run_gust(&["commit", "-m", "Add three files"]);

    let status_output = repo.run_gust(&["status"]);
    assert!(status_output.status.success());

    // Create new branch and modify files
    repo.run_gust(&["branch", "modify"]);
    repo.run_gust(&["checkout", "modify"]);

    repo.create_file("file2.txt", "modified content");
    repo.run_gust(&["add", "file2.txt"]);
    repo.run_gust(&["commit", "-m", "Modified file2"]);

    // Switch back to main branch
    repo.run_gust(&["checkout", "main"]);

    // Verify file2 has original content
    let content = repo.read_file("file2.txt");
    assert_eq!(content, "content2", "file2 should have original content on main");
}

#[test]
fn test_empty_commit_message() {
    let repo = TestRepo::new("empty_message");

    repo.run_gust(&["init"]);
    repo.create_file("file.txt", "content");
    repo.run_gust(&["add", "file.txt"]);

    let output = repo.run_gust(&["commit", "-m", ""]);
    // This should succeed based on CLI definition (default_value = "")
    assert!(output.status.success(), "commit with empty message should work");
}

#[test]
fn test_log_shows_commit_history() {
    let repo = TestRepo::new("log_history");

    repo.run_gust(&["init"]);

    repo.create_file("file1.txt", "v1");
    repo.run_gust(&["add", "file1.txt"]);
    repo.run_gust(&["commit", "-m", "First commit"]);

    repo.create_file("file2.txt", "v2");
    repo.run_gust(&["add", "file2.txt"]);
    repo.run_gust(&["commit", "-m", "Second commit"]);

    repo.create_file("file3.txt", "v3");
    repo.run_gust(&["add", "file3.txt"]);
    repo.run_gust(&["commit", "-m", "Third commit"]);

    let log_output = repo.run_gust(&["log"]);
    assert!(log_output.status.success());

    let log_str = String::from_utf8_lossy(&log_output.stdout);
    assert!(log_str.contains("First commit"));
    assert!(log_str.contains("Second commit"));
    assert!(log_str.contains("Third commit"));
}
