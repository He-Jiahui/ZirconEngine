//! 场景切换导出只在当前 Level 写入一个待处理请求；请求 ID 单调递增，新的请求替换尚未消费的旧请求，由动态会话最终执行。
use crate::core::framework::scene::SceneResource;
use crate::core::framework::script::{ScriptHostCallFrame, ScriptHostError, ScriptHostValue};
use crate::script::runtime_context_for_frame;
use zircon_runtime_interface::runtime_api::{
    ZrRuntimeProjectSceneTransitionPolicyV1, ZrRuntimeProjectSceneTransitionRequestErrorV1,
    ZrRuntimeProjectSceneTransitionRequestV1, ZrRuntimeProjectSceneTransitionResultV1,
};

use super::values::with_string;

#[derive(Clone, Copy, Debug)]
struct ProjectSceneTransitionSequence {
    next_request_id: u64,
}

impl SceneResource for ProjectSceneTransitionSequence {}
impl SceneResource for ZrRuntimeProjectSceneTransitionRequestV1 {}
impl SceneResource for ZrRuntimeProjectSceneTransitionResultV1 {}

pub(super) fn request_scene_transition(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    with_string(context, 0, |scene_uri: &str| {
        let request_id = runtime
            .level
            .with_world_mut(
                |world| -> Result<u64, ZrRuntimeProjectSceneTransitionRequestErrorV1> {
                    let generation_floor = world.world_generation().saturating_add(1);
                    let completed_floor = world
                        .get_resource::<ZrRuntimeProjectSceneTransitionResultV1>()
                        .map_or(0, |result| result.request_id.saturating_add(1));
                    let next_request_id = world
                        .get_resource::<ProjectSceneTransitionSequence>()
                        .map(|sequence| sequence.next_request_id)
                        .unwrap_or(generation_floor);
                    let request_id = next_request_id.max(generation_floor).max(completed_floor);
                    let request = ZrRuntimeProjectSceneTransitionRequestV1::try_new(
                        request_id,
                        scene_uri,
                        ZrRuntimeProjectSceneTransitionPolicyV1::ReplaceActive,
                    )?;
                    if let Some(sequence) =
                        world.get_resource_mut::<ProjectSceneTransitionSequence>()
                    {
                        sequence.next_request_id = request_id.saturating_add(1);
                    } else {
                        world.insert_resource(ProjectSceneTransitionSequence {
                            next_request_id: request_id.saturating_add(1),
                        });
                    }
                    world.insert_resource(request);
                    Ok(request_id)
                },
            )
            .map_err(|error| ScriptHostError::new(error.to_string()))?;
        Ok(ScriptHostValue::Int(request_id as i64))
    })
}

/// Latest completed request, or JSON null before a result exists. The ID correlates it
/// with the value returned by request_scene_transition; reading does not consume it.
pub(super) fn scene_transition_result_json(
    context: &ScriptHostCallFrame<'_>,
) -> Result<ScriptHostValue, ScriptHostError> {
    let runtime = runtime_context_for_frame(context)?;
    runtime
        .level
        .with_world(|world| {
            serde_json::to_string(&world.get_resource::<ZrRuntimeProjectSceneTransitionResultV1>())
        })
        .map(ScriptHostValue::String)
        .map_err(|error| ScriptHostError::new(error.to_string()))
}
