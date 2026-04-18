use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum WelcomeCommand {
    SetProjectName { value: String },
    SetLocation { value: String },
    CreateProject,
    OpenExistingProject,
    OpenRecentProject { path: String },
    RemoveRecentProject { path: String },
}
