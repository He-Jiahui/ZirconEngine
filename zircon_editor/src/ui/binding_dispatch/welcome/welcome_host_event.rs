use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WelcomeHostEvent {
    SetProjectName { value: String },
    SetLocation { value: String },
    CreateProject,
    OpenExistingProject,
    OpenRecentProject { path: String },
    RemoveRecentProject { path: String },
}
