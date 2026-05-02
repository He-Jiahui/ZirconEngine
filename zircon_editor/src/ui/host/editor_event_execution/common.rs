use serde_json::{json, Value};
use zircon_runtime_interface::resource::ResourceKind;

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorEventEffect;
use crate::ui::workbench::view::ViewDescriptorId;

use super::execution_outcome::ExecutionOutcome;
use crate::core::editor_event::runtime::editor_event_runtime_inner::EditorEventRuntimeInner;

pub(super) fn scene_intent_event(
    inner: &mut EditorEventRuntimeInner,
    intent: EditorIntent,
) -> Result<ExecutionOutcome, String> {
    let changed = inner.state.apply_intent(intent)?;
    Ok(ExecutionOutcome {
        changed,
        effects: scene_effects(),
    })
}

pub(super) fn scene_effects() -> Vec<EditorEventEffect> {
    vec![
        EditorEventEffect::RenderChanged,
        EditorEventEffect::PresentationChanged,
        EditorEventEffect::ReflectionChanged,
    ]
}

pub(super) fn asset_effects(
    changed: bool,
    refresh_details: bool,
    refresh_visible_previews: bool,
) -> ExecutionOutcome {
    let mut effects = vec![
        EditorEventEffect::PresentationChanged,
        EditorEventEffect::ReflectionChanged,
    ];
    if refresh_details {
        effects.push(EditorEventEffect::AssetDetailsRefreshRequested);
    }
    if refresh_visible_previews {
        effects.push(EditorEventEffect::AssetPreviewRefreshRequested);
    }
    ExecutionOutcome { changed, effects }
}

pub(super) fn open_view(
    inner: &mut EditorEventRuntimeInner,
    descriptor_id: &str,
    status_line: &str,
) -> Result<ExecutionOutcome, String> {
    let instance_id = inner
        .manager
        .open_view(ViewDescriptorId::new(descriptor_id), None)
        .map_err(|error| error.to_string())?;
    let focused = inner
        .manager
        .focus_view(&instance_id)
        .map_err(|error| error.to_string())?;
    inner.state.set_status_line(status_line);
    Ok(ExecutionOutcome {
        changed: focused || !instance_id.0.is_empty(),
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

pub(super) fn parse_asset_kind_filter(kind: Option<&str>) -> Result<Option<ResourceKind>, String> {
    match kind.unwrap_or_default() {
        "" | "All" => Ok(None),
        "Texture" => Ok(Some(ResourceKind::Texture)),
        "Shader" => Ok(Some(ResourceKind::Shader)),
        "Material" => Ok(Some(ResourceKind::Material)),
        "PhysicsMaterial" => Ok(Some(ResourceKind::PhysicsMaterial)),
        "Scene" => Ok(Some(ResourceKind::Scene)),
        "Model" => Ok(Some(ResourceKind::Model)),
        "AnimationSkeleton" => Ok(Some(ResourceKind::AnimationSkeleton)),
        "AnimationClip" => Ok(Some(ResourceKind::AnimationClip)),
        "AnimationSequence" => Ok(Some(ResourceKind::AnimationSequence)),
        "AnimationGraph" => Ok(Some(ResourceKind::AnimationGraph)),
        "AnimationStateMachine" => Ok(Some(ResourceKind::AnimationStateMachine)),
        "UiLayout" => Ok(Some(ResourceKind::UiLayout)),
        "UiWidget" => Ok(Some(ResourceKind::UiWidget)),
        "UiStyle" => Ok(Some(ResourceKind::UiStyle)),
        other => Err(format!("unknown asset kind filter {other}")),
    }
}

pub(crate) fn event_result_value(revision: u64, changed: bool) -> Value {
    json!({
        "revision": revision,
        "changed": changed,
    })
}
