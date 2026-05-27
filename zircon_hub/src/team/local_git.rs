use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::HubError;
use crate::projects::project_filesystem_path_key;

const GIT_COMMAND: &str = "git";
const RECENT_AUTHOR_LIMIT: usize = 8;
const RECENT_COMMIT_SCAN_LIMIT: &str = "200";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamOverview {
    pub repository_path: PathBuf,
    pub identity_name: String,
    pub identity_email: String,
    pub members: Vec<TeamMemberEntry>,
}

impl TeamOverview {
    pub fn empty() -> Self {
        Self {
            repository_path: PathBuf::new(),
            identity_name: String::new(),
            identity_email: String::new(),
            members: Vec::new(),
        }
    }
}

impl Default for TeamOverview {
    fn default() -> Self {
        Self::empty()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamMemberEntry {
    pub name: String,
    pub email: String,
    pub commits: u32,
}

pub fn discover_team_overview<I>(repo_roots: I) -> Result<TeamOverview, HubError>
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut visited = HashSet::new();
    for repo_root in repo_roots {
        if repo_root.as_os_str().is_empty() {
            continue;
        }
        let key = project_filesystem_path_key(&repo_root);
        if !visited.insert(key) {
            continue;
        }
        let Some(repository_path) = git_repository_root(&repo_root)? else {
            continue;
        };
        return Ok(read_team_overview(repository_path));
    }
    Ok(TeamOverview::empty())
}

fn read_team_overview(repository_path: PathBuf) -> TeamOverview {
    let identity_name = git_output(&repository_path, &["config", "--get", "user.name"])
        .ok()
        .flatten()
        .unwrap_or_default()
        .trim()
        .to_string();
    let identity_email = git_output(&repository_path, &["config", "--get", "user.email"])
        .ok()
        .flatten()
        .unwrap_or_default()
        .trim()
        .to_string();
    let members = git_output(
        &repository_path,
        &[
            "log",
            "--all",
            "--format=%an%x1f%ae",
            "-n",
            RECENT_COMMIT_SCAN_LIMIT,
        ],
    )
    .ok()
    .flatten()
    .map(|output| parse_git_log_authors(&output))
    .unwrap_or_default();

    TeamOverview {
        repository_path,
        identity_name,
        identity_email,
        members,
    }
}

fn git_repository_root(path: &Path) -> Result<Option<PathBuf>, HubError> {
    Ok(git_output(path, &["rev-parse", "--show-toplevel"])?
        .map(|output| PathBuf::from(output.trim()))
        .filter(|path| !path.as_os_str().is_empty()))
}

fn git_output(path: &Path, args: &[&str]) -> Result<Option<String>, HubError> {
    let output = match Command::new(GIT_COMMAND)
        .arg("-C")
        .arg(path)
        .args(args)
        .output()
    {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if !output.status.success() {
        return Ok(None);
    }
    Ok(Some(String::from_utf8_lossy(&output.stdout).into_owned()))
}

fn parse_git_log_authors(output: &str) -> Vec<TeamMemberEntry> {
    let mut counts = BTreeMap::<(String, String), u32>::new();
    for line in output.lines() {
        let Some((name, email)) = line.split_once('\x1f') else {
            continue;
        };
        let name = name.trim();
        let email = email.trim();
        if name.is_empty() && email.is_empty() {
            continue;
        }
        *counts
            .entry((name.to_string(), email.to_string()))
            .or_insert(0) += 1;
    }

    let mut members: Vec<_> = counts
        .into_iter()
        .map(|((name, email), commits)| TeamMemberEntry {
            name,
            email,
            commits,
        })
        .collect();
    members.sort_by(|left, right| {
        right
            .commits
            .cmp(&left.commits)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.email.cmp(&right.email))
    });
    members.truncate(RECENT_AUTHOR_LIMIT);
    members
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_git_log_authors_counts_and_sorts_recent_authors() {
        let members = parse_git_log_authors(
            "Ada\x1fada@example.com\nLin\x1flin@example.com\nAda\x1fada@example.com\n",
        );

        assert_eq!(members.len(), 2);
        assert_eq!(members[0].name, "Ada");
        assert_eq!(members[0].email, "ada@example.com");
        assert_eq!(members[0].commits, 2);
        assert_eq!(members[1].name, "Lin");
        assert_eq!(members[1].commits, 1);
    }

    #[test]
    fn parse_git_log_authors_skips_invalid_and_empty_lines() {
        let members = parse_git_log_authors("\nmissing separator\n\x1f\nName\x1f\n");

        assert_eq!(members.len(), 1);
        assert_eq!(members[0].name, "Name");
        assert_eq!(members[0].email, "");
        assert_eq!(members[0].commits, 1);
    }
}
