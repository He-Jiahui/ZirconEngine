#[path = "local_git.rs"]
mod local_git;

pub use local_git::{discover_team_overview, TeamMemberEntry, TeamOverview};
