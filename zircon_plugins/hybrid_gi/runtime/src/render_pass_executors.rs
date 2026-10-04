use zircon_runtime::core::framework::render::PostProcessGraphResourceNames;
use zircon_runtime::graphics::RenderPassExecutionContext;
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

mod resolve_trace_handoff;
mod scene_depth_handoff;
mod scene_hzb_camera_packet;
mod scene_trace_input_packet;
mod trace_schedule_handoff;

use resolve_trace_handoff::record_resolve_trace_handoff;
use scene_depth_handoff::record_scene_depth_handoff;
use scene_trace_input_packet::SCENE_TRACE_INPUT_TOTAL_WORD_COUNT;
use trace_schedule_handoff::record_trace_schedule_handoff;

#[derive(Clone, Copy, Debug)]
struct RenderPassExecutorContract {
    pass_name: &'static str,
    executor_id: &'static str,
    declared_queue: QueueLane,
    flags: PassFlags,
    resources: &'static [ExpectedResource],
}

#[derive(Clone, Copy, Debug)]
struct ExpectedResource {
    name: &'static str,
    kind: ExpectedResourceKind,
    access: RenderGraphResourceAccessKind,
}

#[derive(Clone, Copy, Debug)]
enum ExpectedResourceKind {
    Exact(RenderGraphResourceKind),
    AnyOf(&'static [RenderGraphResourceKind]),
}

impl ExpectedResource {
    const fn new(
        name: &'static str,
        kind: RenderGraphResourceKind,
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self {
            name,
            kind: ExpectedResourceKind::Exact(kind),
            access,
        }
    }

    const fn any_of(
        name: &'static str,
        kinds: &'static [RenderGraphResourceKind],
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self {
            name,
            kind: ExpectedResourceKind::AnyOf(kinds),
            access,
        }
    }

    fn description(self) -> String {
        describe_expected_resource(self.name, self.kind, self.access)
    }

    fn matches(self, resource: &RenderGraphPassResourceAccess) -> bool {
        self.name == resource.name
            && self.access == resource.access
            && self.kind.matches(resource.kind)
    }
}

impl ExpectedResourceKind {
    fn matches(self, kind: RenderGraphResourceKind) -> bool {
        match self {
            Self::Exact(expected) => expected == kind,
            Self::AnyOf(expected) => expected.contains(&kind),
        }
    }
}

const READ_ONLY_TEXTURE_INPUT_KINDS: &[RenderGraphResourceKind] = &[
    RenderGraphResourceKind::External,
    RenderGraphResourceKind::TransientTexture,
];
const SCENE_DEPTH_RESOURCE: &str = PostProcessGraphResourceNames::SCENE_DEPTH;
const SCENE_NORMAL_RESOURCE: &str = PostProcessGraphResourceNames::GBUFFER_NORMAL;
const SCENE_HZB_RESOURCE: &str = PostProcessGraphResourceNames::HZB_FURTHEST;
const SCENE_VELOCITY_RESOURCE: &str = PostProcessGraphResourceNames::SCENE_VELOCITY;
pub(crate) const HYBRID_GI_SCENE_RESOURCE: &str = PostProcessGraphResourceNames::HYBRID_GI_SCENE;
pub(crate) const HYBRID_GI_TRACE_RESOURCE: &str = PostProcessGraphResourceNames::HYBRID_GI_TRACE;
const HYBRID_GI_LIGHTING_RESOURCE: &str = PostProcessGraphResourceNames::HYBRID_GI_LIGHTING;
const HYBRID_GI_TEMPORAL_METADATA_RESOURCE: &str =
    PostProcessGraphResourceNames::HYBRID_GI_TEMPORAL_METADATA;
const HYBRID_GI_HISTORY_RESOURCE: &str = PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI;
const HYBRID_GI_TEMPORAL_METADATA_HISTORY_RESOURCE: &str =
    PostProcessGraphResourceNames::HISTORY_PREVIOUS_HYBRID_GI_TEMPORAL_METADATA;

// Fixed packet headers and 8x8 tile records must fit even when the viewport is one pixel.
pub(crate) const HYBRID_GI_SCENE_BUFFER_MINIMUM_SIZE_BYTES: u64 =
    SCENE_TRACE_INPUT_TOTAL_WORD_COUNT as u64 * 4;
pub(crate) const HYBRID_GI_TRACE_BUFFER_MINIMUM_SIZE_BYTES: u64 = 576 * 4;

const SCENE_PREPARE_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::any_of(
        SCENE_DEPTH_RESOURCE,
        READ_ONLY_TEXTURE_INPUT_KINDS,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::any_of(
        SCENE_NORMAL_RESOURCE,
        READ_ONLY_TEXTURE_INPUT_KINDS,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_SCENE_RESOURCE,
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
];

const TRACE_SCHEDULE_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        SCENE_HZB_RESOURCE,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_SCENE_RESOURCE,
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_TRACE_RESOURCE,
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Write,
    ),
];

const RESOLVE_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        HYBRID_GI_TRACE_RESOURCE,
        RenderGraphResourceKind::TransientBuffer,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::any_of(
        SCENE_VELOCITY_RESOURCE,
        READ_ONLY_TEXTURE_INPUT_KINDS,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_HISTORY_RESOURCE,
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_TEMPORAL_METADATA_HISTORY_RESOURCE,
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        HYBRID_GI_LIGHTING_RESOURCE,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        HYBRID_GI_TEMPORAL_METADATA_RESOURCE,
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
];

const SCENE_PREPARE_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "hybrid-gi-scene-prepare",
    executor_id: "hybrid-gi.scene-prepare",
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: SCENE_PREPARE_RESOURCES,
};

const TRACE_SCHEDULE_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "hybrid-gi-trace-schedule",
    executor_id: "hybrid-gi.trace-schedule",
    declared_queue: QueueLane::AsyncCompute,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: TRACE_SCHEDULE_RESOURCES,
};

const RESOLVE_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "hybrid-gi-resolve",
    executor_id: "hybrid-gi.resolve",
    declared_queue: QueueLane::Graphics,
    flags: PassFlags {
        allow_culling: true,
        has_side_effects: false,
    },
    resources: RESOLVE_RESOURCES,
};

pub(crate) fn hybrid_gi_scene_prepare_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &SCENE_PREPARE_CONTRACT)?;
    if context.gpu().is_none() {
        return Ok(());
    }
    record_scene_depth_handoff(context)
}

pub(crate) fn hybrid_gi_trace_schedule_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &TRACE_SCHEDULE_CONTRACT)?;
    if context.gpu().is_none() {
        return Ok(());
    }
    record_trace_schedule_handoff(context)
}

pub(crate) fn hybrid_gi_resolve_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &RESOLVE_CONTRACT)?;
    if context.gpu().is_none() {
        return Ok(());
    }
    record_resolve_trace_handoff(context)
}

fn validate_context(
    context: &RenderPassExecutionContext<'_>,
    contract: &RenderPassExecutorContract,
) -> Result<(), String> {
    if context.executor_id.as_str() != contract.executor_id {
        return Err(format!(
            "hybrid GI executor contract mismatch: pass `{}` expected executor `{}`, got `{}`",
            context.pass_name, contract.executor_id, context.executor_id
        ));
    }
    if context.pass_name != contract.pass_name {
        return Err(format!(
            "hybrid GI executor `{}` received pass `{}`, expected `{}`",
            contract.executor_id, context.pass_name, contract.pass_name
        ));
    }
    if context.declared_queue != contract.declared_queue {
        return Err(format!(
            "hybrid GI executor `{}` declared queue mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.declared_queue
        ));
    }
    if !queue_is_compatible(context.queue, contract.declared_queue) {
        return Err(format!(
            "hybrid GI executor `{}` ran on incompatible queue for pass `{}`: declared `{:?}`, actual `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.queue
        ));
    }
    if context.flags != contract.flags {
        return Err(format!(
            "hybrid GI executor `{}` pass flag mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.flags, context.flags
        ));
    }

    if !resource_contract_matches(contract.resources, &context.resources) {
        return Err(format!(
            "hybrid GI executor `{}` resource contract mismatch for pass `{}`: expected {:?}, got {:?}",
            contract.executor_id,
            context.pass_name,
            expected_resource_descriptions(contract.resources),
            actual_resource_descriptions(&context.resources)
        ));
    }

    Ok(())
}

// 声明为异步队列的通道可在图调度回退到图形队列时执行；图形通道不能反向落入异步队列。
fn queue_is_compatible(actual: QueueLane, declared: QueueLane) -> bool {
    actual == declared || (declared != QueueLane::Graphics && actual == QueueLane::Graphics)
}

fn expected_resource_descriptions(resources: &[ExpectedResource]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| resource.description())
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

fn actual_resource_descriptions(resources: &[RenderGraphPassResourceAccess]) -> Vec<String> {
    let mut descriptions = resources
        .iter()
        .map(|resource| describe_resource(&resource.name, resource.kind, resource.access))
        .collect::<Vec<_>>();
    descriptions.sort();
    descriptions
}

// 按资源名、类型和访问方式做无序一一匹配，避免重复资源或声明重排掩盖图契约漂移。
fn resource_contract_matches(
    expected: &[ExpectedResource],
    actual: &[RenderGraphPassResourceAccess],
) -> bool {
    if expected.len() != actual.len() {
        return false;
    }

    let mut matched = vec![false; actual.len()];
    for expected_resource in expected {
        let Some(index) = actual
            .iter()
            .enumerate()
            .find(|(index, resource)| !matched[*index] && expected_resource.matches(resource))
            .map(|(index, _)| index)
        else {
            return false;
        };
        matched[index] = true;
    }

    true
}

fn describe_expected_resource(
    name: &str,
    kind: ExpectedResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    match kind {
        ExpectedResourceKind::Exact(kind) => describe_resource(name, kind, access),
        ExpectedResourceKind::AnyOf(kinds) => {
            let kinds = kinds
                .iter()
                .map(|kind| format!("{kind:?}"))
                .collect::<Vec<_>>()
                .join("|");
            format!("{access:?}:{kinds}:{name}")
        }
    }
}

fn describe_resource(
    name: &str,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    format!("{access:?}:{kind:?}:{name}")
}

#[cfg(test)]
#[path = "tests/render_pass_executors.rs"]
mod tests;
