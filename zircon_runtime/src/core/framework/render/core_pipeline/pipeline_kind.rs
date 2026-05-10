use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CorePipelineKind {
    Core2d,
    #[default]
    Core3d,
}
