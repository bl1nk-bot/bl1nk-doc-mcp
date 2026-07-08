use std::path::PathBuf;
use std::process::Command;

use bl1nk_doc_mcp::tools::status::repo_status_impl;
use tempfile::TempDir;

/// CI checkouts are detached HEAD (`git branch --show-current` is empty),
/// so tests that assert on branch state need a repo of their own.
fn init_temp_repo() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let git = |args: &[&str]| {
        let status = Command::new("git")
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap();
        assert!(status.success(), "git {args:?} failed");
    };
    git(&["init", "-b", "main"]);
    git(&["config", "user.email", "test@example.com"]);
    git(&["config", "user.name", "Test"]);
    std::fs::write(dir.path().join("README.md"), "test").unwrap();
    git(&["add", "."]);
    git(&["commit", "--no-gpg-sign", "-m", "initial commit"]);
    dir
}

#[tokio::test]
async fn test_repo_status_clean_repo() {
    let repo = init_temp_repo();
    let result = repo_status_impl(repo.path().to_path_buf(), Some(5)).await;

    if let Err(e) = &result {
        eprintln!("Error: {:?}", e);
    }

    assert!(result.is_ok(), "repo_status should succeed in a git repo");
    let output = result.unwrap();

    assert_eq!(output.branch, "main", "branch should be the repo's branch");
    assert!(
        !output.head_commit.is_empty(),
        "head_commit should not be empty"
    );
    assert_eq!(
        output.head_commit.len(),
        40,
        "head_commit should be a full SHA"
    );
    assert!(output.recent_commits.len() <= 5, "should respect max_count");
    assert!(!output.evidence.is_empty(), "should have evidence");
}

#[tokio::test]
async fn test_repo_status_default_commits() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let result = repo_status_impl(repo_root, None).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(
        output.recent_commits.len() <= 10,
        "default should be 10 commits"
    );
}

#[tokio::test]
async fn test_repo_status_clamp_max() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let result = repo_status_impl(repo_root, Some(100)).await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.recent_commits.len() <= 50, "should clamp to max 50");
}

#[tokio::test]
async fn test_repo_status_nonexistent_repo() {
    let repo_root = PathBuf::from("/nonexistent/path/that/does/not/exist");
    let result = repo_status_impl(repo_root, None).await;

    assert!(result.is_err(), "should fail for non-existent repo");
}
