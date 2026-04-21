use serde::{Deserialize, Serialize};

use super::MainPageId;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkspaceTarget {
    MainPage(MainPageId),
    FloatingWindow(MainPageId),
}
