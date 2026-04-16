use serde::{Deserialize, Serialize};

use crate::ViewHost;

use super::{SplitAxis, SplitPlacement, WorkspaceTarget};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum DropTarget {
    Host(ViewHost),
    Split {
        workspace: WorkspaceTarget,
        path: Vec<usize>,
        axis: SplitAxis,
        placement: SplitPlacement,
    },
    NewFloatingWindow,
}
