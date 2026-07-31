use serde::{Deserialize, Serialize};

use crate::core::editor_message::SceneModeId;
use crate::scene::viewport::TransformHandleKind;

pub(crate) const SELECT_SCENE_MODE_ID: &str = "scene.select";
pub(crate) const TRANSFORM_SCENE_MODE_ID: &str = "scene.transform";

/// A request to make a base scene mode active.
///
/// Selection and transform are editor-owned built-ins. Extensions use `Custom`
/// and still resolve through the same scene-mode registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum SceneModeActivation {
    Select,
    Transform(TransformHandleKind),
    Custom(SceneModeId),
}

impl SceneModeActivation {
    pub(crate) fn mode_id(&self) -> SceneModeId {
        match self {
            Self::Select => SceneModeId::new(SELECT_SCENE_MODE_ID),
            Self::Transform(_) => SceneModeId::new(TRANSFORM_SCENE_MODE_ID),
            Self::Custom(mode_id) => mode_id.clone(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), String> {
        let Self::Custom(mode_id) = self else {
            return Ok(());
        };
        if mode_id.as_str() == SELECT_SCENE_MODE_ID || mode_id.as_str() == TRANSFORM_SCENE_MODE_ID {
            return Err(format!(
                "custom scene mode activation cannot use reserved built-in id \"{}\"",
                mode_id.as_str()
            ));
        }
        Ok(())
    }

    pub(crate) fn transform_handle(&self) -> Option<TransformHandleKind> {
        match self {
            Self::Transform(kind) => Some(*kind),
            Self::Select | Self::Custom(_) => None,
        }
    }
}
