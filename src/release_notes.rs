use std::collections::HashSet;

#[derive(Clone, Debug)]
pub struct PullRequestInfo {
    pub number: u64,
    pub title: String,
    pub merged_at: Option<String>,
}

pub fn release_marker(branch: &str) -> String {
    format!("<!-- breezy:branch={branch} -->")
}

fn sort_by_merge_date(pull_requests: &[PullRequestInfo]) -> Vec<PullRequestInfo> {
    let mut ordered = pull_requests.to_vec();
    ordered.sort_by(|left, right| left.merged_at.cmp(&right.merged_at));
    ordered
}

pub fn build_release_notes(marker: &str, pull_requests: &[PullRequestInfo]) -> String {
    let mut lines = vec![marker.to_string()];
    let mut seen = HashSet::new();

    for pull_request in sort_by_merge_date(pull_requests) {
        if seen.contains(&pull_request.number) {
            continue;
        }
        seen.insert(pull_request.number);
        lines.push(pull_request.title);
    }

    if lines.len() == 1 {
        return lines.remove(0);
    }

    let mut body = Vec::with_capacity(lines.len() + 1);
    body.push(lines.remove(0));
    body.push(String::new());
    body.extend(lines);
    body.join("\n")
}
