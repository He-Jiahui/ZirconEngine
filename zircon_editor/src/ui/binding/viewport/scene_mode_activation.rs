use crate::core::editor_message::SceneModeId;
use crate::scene::modes::{SceneModeActivation, SELECT_SCENE_MODE_ID, TRANSFORM_SCENE_MODE_ID};
use crate::scene::viewport::TransformHandleKind;

pub(crate) fn symbol(mode: &SceneModeActivation) -> String {
    match mode {
        SceneModeActivation::Select => "Select".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Move) => "Transform.Move".to_string(),
        SceneModeActivation::Transform(TransformHandleKind::Rotate) => {
            "Transform.Rotate".to_string()
        }
        SceneModeActivation::Transform(TransformHandleKind::Scale) => "Transform.Scale".to_string(),
        SceneModeActivation::Custom(mode_id) => format!("Custom:{}", mode_id.as_str()),
    }
}

pub(crate) fn parse_symbol(symbol: &str) -> Option<SceneModeActivation> {
    match symbol {
        "Select" => Some(SceneModeActivation::Select),
        "Transform.Move" => Some(SceneModeActivation::Transform(TransformHandleKind::Move)),
        "Transform.Rotate" => Some(SceneModeActivation::Transform(TransformHandleKind::Rotate)),
        "Transform.Scale" => Some(SceneModeActivation::Transform(TransformHandleKind::Scale)),
        custom if custom.starts_with("Custom:") && custom.len() > "Custom:".len() => {
            let mode_id = &custom["Custom:".len()..];
            (mode_id != SELECT_SCENE_MODE_ID && mode_id != TRANSFORM_SCENE_MODE_ID)
                .then(|| SceneModeActivation::Custom(SceneModeId::new(mode_id)))
        }
        _ => None,
    }
}
