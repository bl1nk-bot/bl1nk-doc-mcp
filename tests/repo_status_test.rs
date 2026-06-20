use std::path::PathBuf;

use bl1nk_doc_mcp::tools::status::repo_status_impl;

#[tokio::test]
async fn test_repo_status_clean_repo() {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let result = repo_status_impl(repo_root, Some(5)).await;

    if let Err(e) = &result {
        eprintln!("Error: {:?}", e);
    }

    assert!(result.is_ok(), "repo_status should succeed in a git repo");
    let output = result.unwrap();

    assert!(!output.branch.is_empty(), "branch should not be empty");
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
