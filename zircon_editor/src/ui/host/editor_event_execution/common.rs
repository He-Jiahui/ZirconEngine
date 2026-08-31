use serde_json::{json, Value};
use zircon_runtime_interface::resource::ResourceKind;

use crate::core::editing::intent::EditorIntent;
use crate::core::editor_event::EditorEventEffect;
use crate::ui::host::EditorError;
use crate::ui::workbench::shell_state::WorkbenchShellStateData;
use crate::ui::workbench::state::EditorStateOperationError;
use crate::ui::workbench::view::ViewDescriptorId;

use super::error::AssetKindFilterError;
use super::execution_outcome::ExecutionOutcome;
pub(super) fn scene_intent_event(
    shell: &mut WorkbenchShellStateData,
    intent: EditorIntent,
) -> Result<ExecutionOutcome, EditorStateOperationError> {
    let changed = shell.state.apply_intent(intent)?;
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

pub(super) fn effects_when<const N: usize>(
    changed: bool,
    effects: [EditorEventEffect; N],
) -> Vec<EditorEventEffect> {
    if changed {
        Vec::from(effects)
    } else {
        Vec::new()
    }
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

pub(super) fn asset_mutation_effects(
    changed: bool,
    refresh_details: bool,
    refresh_visible_previews: bool,
) -> ExecutionOutcome {
    if changed {
        asset_effects(true, refresh_details, refresh_visible_previews)
    } else {
        ExecutionOutcome {
            changed: false,
            effects: Vec::new(),
        }
    }
}

pub(super) fn open_view(
    shell: &mut WorkbenchShellStateData,
    descriptor_id: &str,
    status_line: &str,
) -> Result<ExecutionOutcome, EditorError> {
    let instance_id = shell
        .manager
        .open_view(ViewDescriptorId::new(descriptor_id), None)?;
    let focused = shell.manager.focus_view(&instance_id)?;
    shell.state.set_status_line(status_line);
    Ok(ExecutionOutcome {
        changed: focused || !instance_id.0.is_empty(),
        effects: vec![
            EditorEventEffect::LayoutChanged,
            EditorEventEffect::PresentationChanged,
            EditorEventEffect::ReflectionChanged,
        ],
    })
}

pub(super) fn parse_asset_kind_filter(
    kind: Option<&str>,
) -> Result<Option<ResourceKind>, AssetKindFilterError> {
    match kind.unwrap_or_default() {
        "" | "All" => Ok(None),
        "Texture" => Ok(Some(ResourceKind::Texture)),
        "Shader" => Ok(Some(ResourceKind::Shader)),
        "Material" => Ok(Some(ResourceKind::Material)),
        "PhysicsMaterial" => Ok(Some(ResourceKind::PhysicsMaterial)),
        "Scene" => Ok(Some(ResourceKind::Scene)),
        "Model" => Ok(Some(ResourceKind::Model)),
        "Mesh" => Ok(Some(ResourceKind::Mesh)),
        "AnimationSkeleton" => Ok(Some(ResourceKind::AnimationSkeleton)),
        "AnimationClip" => Ok(Some(ResourceKind::AnimationClip)),
        "AnimationSequence" => Ok(Some(ResourceKind::AnimationSequence)),
        "AnimationGraph" => Ok(Some(ResourceKind::AnimationGraph)),
        "AnimationStateMachine" => Ok(Some(ResourceKind::AnimationStateMachine)),
        "UiLayout" => Ok(Some(ResourceKind::UiLayout)),
        "UiWidget" => Ok(Some(ResourceKind::UiWidget)),
        "UiStyle" => Ok(Some(ResourceKind::UiStyle)),
        other => Err(AssetKindFilterError::Unknown {
            value: other.to_string(),
        }),
    }
}

pub(crate) fn event_result_value(revision: u64, changed: bool) -> Value {
    json!({
        "revision": revision,
        "changed": changed,
    })
}
