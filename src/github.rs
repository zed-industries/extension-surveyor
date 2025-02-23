use anyhow::{anyhow, Result};
use url::Url;

/// Returns a URL for creating a new GitHub Issue with the given title and body.
pub fn create_github_issue_url(repository_url: &str, title: &str, body: &str) -> Result<Url> {
    let mut github_issue_url = Url::parse(repository_url)?;
    github_issue_url
        .path_segments_mut()
        .map_err(|_| anyhow!("invalid repository URL"))?
        .extend(["issues", "new"]);
    github_issue_url
        .query_pairs_mut()
        .append_pair("title", title)
        .append_pair("body", body)
        .finish();

    Ok(github_issue_url)
}
