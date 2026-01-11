mod github;
mod release_notes;
mod version;

use anyhow::{anyhow, bail, Context, Result};
use github::ReleaseInfo;
use release_notes::{build_release_notes, release_marker};
use std::env;
use version::{parse_languages, resolve_version};

const MAX_PER_PAGE: u32 = 100;

struct DraftSelection {
    primary: Option<u64>,
    extras: Vec<u64>,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let branch = required_input("branch")?;
    let language_input = required_input("language")?;
    let tag_prefix = read_input("tag-prefix").unwrap_or_else(|| "v".to_string());
    let token = read_input("github-token")
        .or_else(|| env::var("GITHUB_TOKEN").ok())
        .unwrap_or_default();

    if token.trim().is_empty() {
        bail!("Missing GitHub token. Set the github-token input or GITHUB_TOKEN env.");
    }

    let languages = parse_languages(&language_input);
    if languages.is_empty() {
        bail!("No language archetypes provided.");
    }

    let cwd = env::current_dir().context("Unable to resolve current working directory.")?;
    let version_info = resolve_version(&cwd, &languages)?;

    let tag_name = format!("{}{}", tag_prefix.trim(), version_info.version);
    let release_name = format!("{tag_name} ({branch})");
    let marker = release_marker(&branch);

    let (owner, repo) = parse_repository()?;
    let client = github::GitHubClient::new(&token, &owner, &repo)?;

    let releases = client.list_all_releases(MAX_PER_PAGE)?;
    let selection = select_draft_releases(&releases, &marker);

    for release_id in selection.extras {
        client.delete_release(release_id)?;
        println!("Deleted extra draft release {release_id} for {branch}");
    }

    let since = select_latest_published_release(&releases, &branch)
        .map(|release| release.published_at.as_deref().unwrap_or(&release.created_at))
        .map(|value| value.to_string());

    let pull_requests =
        client.fetch_merged_pull_requests(&branch, since.as_deref(), MAX_PER_PAGE)?;
    let release_notes = build_release_notes(&marker, &pull_requests);

    if let Some(release_id) = selection.primary {
        client.update_release(
            release_id,
            &tag_name,
            &release_name,
            &release_notes,
            &branch,
        )?;
        println!("Updated draft release {release_id} for {branch}");
    } else {
        client.create_release(&tag_name, &release_name, &release_notes, &branch)?;
        println!("Created draft release for {branch}");
    }

    Ok(())
}

fn input_key(name: &str) -> String {
    format!("INPUT_{}", name.replace(' ', "_").to_uppercase())
}

fn read_input(name: &str) -> Option<String> {
    let key = input_key(name);
    if let Ok(value) = env::var(&key) {
        return Some(value);
    }

    let alternate = key.replace('-', "_");
    if alternate != key {
        if let Ok(value) = env::var(&alternate) {
            return Some(value);
        }
    }

    None
}

fn required_input(name: &str) -> Result<String> {
    let value = read_input(name).unwrap_or_default();
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("Missing required input: {name}");
    }
    Ok(trimmed.to_string())
}

fn parse_repository() -> Result<(String, String)> {
    let repository = env::var("GITHUB_REPOSITORY")
        .context("Missing GITHUB_REPOSITORY environment variable.")?;
    let mut parts = repository.splitn(2, '/');
    let owner = parts.next().unwrap_or_default();
    let repo = parts.next().unwrap_or_default();
    if owner.is_empty() || repo.is_empty() {
        return Err(anyhow!(
            "Invalid GITHUB_REPOSITORY value; expected owner/repo."
        ));
    }

    Ok((owner.to_string(), repo.to_string()))
}

fn select_draft_releases(releases: &[ReleaseInfo], marker: &str) -> DraftSelection {
    let mut drafts: Vec<&ReleaseInfo> = releases
        .iter()
        .filter(|release| {
            release.draft && release.body.as_deref().unwrap_or("").contains(marker)
        })
        .collect();

    drafts.sort_by(|left, right| right.created_at.cmp(&left.created_at));

    let primary = drafts.first().map(|release| release.id);
    let extras = drafts.iter().skip(1).map(|release| release.id).collect();

    DraftSelection { primary, extras }
}

fn select_latest_published_release<'a>(
    releases: &'a [ReleaseInfo],
    branch: &str,
) -> Option<&'a ReleaseInfo> {
    let mut published: Vec<&ReleaseInfo> = releases
        .iter()
        .filter(|release| !release.draft && release.target_commitish == branch)
        .collect();

    if published.is_empty() {
        return None;
    }

    published.sort_by(|left, right| {
        let left_key = left.published_at.as_deref().unwrap_or(&left.created_at);
        let right_key = right
            .published_at
            .as_deref()
            .unwrap_or(&right.created_at);
        right_key.cmp(left_key)
    });

    published.first().copied()
}
