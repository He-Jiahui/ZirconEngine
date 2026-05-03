use zircon_runtime::graphics::{RenderPassExecutionContext, RenderPassExecutorRegistration};
use zircon_runtime::render_graph::{
    PassFlags, QueueLane, RenderGraphPassResourceAccess, RenderGraphResourceAccessKind,
    RenderGraphResourceKind,
};

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
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
}

impl ExpectedResource {
    const fn new(
        name: &'static str,
        kind: RenderGraphResourceKind,
        access: RenderGraphResourceAccessKind,
    ) -> Self {
        Self { name, kind, access }
    }

    fn description(self) -> String {
        describe_resource(self.name, self.kind, self.access)
    }
}

const SIDE_EFFECT_COMPUTE_FLAGS: PassFlags = PassFlags {
    allow_culling: true,
    has_side_effects: true,
};

const DEFAULT_GRAPHICS_FLAGS: PassFlags = PassFlags {
    allow_culling: true,
    has_side_effects: false,
};

const SPAWN_UPDATE_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "particles.gpu.particles-a",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.emitter-params",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.particles-b",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "particles.gpu.counters",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
];

const COMPACT_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "particles.gpu.particles-b",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.alive-indices",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "particles.gpu.counters",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
];

const INDIRECT_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "particles.gpu.counters",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.indirect-draw-args",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
    ExpectedResource::new(
        "particles.gpu.debug-readback",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Write,
    ),
];

const TRANSPARENT_RESOURCES: &[ExpectedResource] = &[
    ExpectedResource::new(
        "particles.gpu.particles-b",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.alive-indices",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "particles.gpu.indirect-draw-args",
        RenderGraphResourceKind::External,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-depth",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-color",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Read,
    ),
    ExpectedResource::new(
        "scene-color",
        RenderGraphResourceKind::TransientTexture,
        RenderGraphResourceAccessKind::Write,
    ),
];

const SPAWN_UPDATE_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "particle-gpu-spawn-update",
    executor_id: "particle.gpu.spawn-update",
    declared_queue: QueueLane::AsyncCompute,
    flags: SIDE_EFFECT_COMPUTE_FLAGS,
    resources: SPAWN_UPDATE_RESOURCES,
};

const COMPACT_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "particle-gpu-compact-alive",
    executor_id: "particle.gpu.compact-alive",
    declared_queue: QueueLane::AsyncCompute,
    flags: SIDE_EFFECT_COMPUTE_FLAGS,
    resources: COMPACT_RESOURCES,
};

const INDIRECT_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "particle-gpu-build-indirect-args",
    executor_id: "particle.gpu.indirect-args",
    declared_queue: QueueLane::AsyncCompute,
    flags: SIDE_EFFECT_COMPUTE_FLAGS,
    resources: INDIRECT_RESOURCES,
};

const TRANSPARENT_CONTRACT: RenderPassExecutorContract = RenderPassExecutorContract {
    pass_name: "particle-render",
    executor_id: "particle.transparent",
    declared_queue: QueueLane::Graphics,
    flags: DEFAULT_GRAPHICS_FLAGS,
    resources: TRANSPARENT_RESOURCES,
};

pub fn particle_render_pass_executor_registrations() -> Vec<RenderPassExecutorRegistration> {
    vec![
        RenderPassExecutorRegistration::new(
            "particle.gpu.spawn-update",
            particle_gpu_spawn_update_executor,
        ),
        RenderPassExecutorRegistration::new(
            "particle.gpu.compact-alive",
            particle_gpu_compact_alive_executor,
        ),
        RenderPassExecutorRegistration::new(
            "particle.gpu.indirect-args",
            particle_gpu_indirect_args_executor,
        ),
        RenderPassExecutorRegistration::new("particle.transparent", particle_transparent_executor),
    ]
}

fn particle_gpu_spawn_update_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &SPAWN_UPDATE_CONTRACT)
}

fn particle_gpu_compact_alive_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &COMPACT_CONTRACT)
}

fn particle_gpu_indirect_args_executor(
    context: &mut RenderPassExecutionContext<'_>,
) -> Result<(), String> {
    validate_context(context, &INDIRECT_CONTRACT)
}

fn particle_transparent_executor(context: &mut RenderPassExecutionContext<'_>) -> Result<(), String> {
    validate_context(context, &TRANSPARENT_CONTRACT)
}

fn validate_context(
    context: &RenderPassExecutionContext<'_>,
    contract: &RenderPassExecutorContract,
) -> Result<(), String> {
    if context.executor_id.as_str() != contract.executor_id {
        return Err(format!(
            "particle executor contract mismatch: pass `{}` expected executor `{}`, got `{}`",
            context.pass_name, contract.executor_id, context.executor_id
        ));
    }
    if context.pass_name != contract.pass_name {
        return Err(format!(
            "particle executor `{}` received pass `{}`, expected `{}`",
            contract.executor_id, context.pass_name, contract.pass_name
        ));
    }
    if context.declared_queue != contract.declared_queue {
        return Err(format!(
            "particle executor `{}` declared queue mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.declared_queue
        ));
    }
    if !queue_is_compatible(context.queue, contract.declared_queue) {
        return Err(format!(
            "particle executor `{}` ran on incompatible queue for pass `{}`: declared `{:?}`, actual `{:?}`",
            contract.executor_id, context.pass_name, contract.declared_queue, context.queue
        ));
    }
    if context.flags != contract.flags {
        return Err(format!(
            "particle executor `{}` pass flag mismatch for pass `{}`: expected `{:?}`, got `{:?}`",
            contract.executor_id, context.pass_name, contract.flags, context.flags
        ));
    }

    let expected = expected_resource_descriptions(contract.resources);
    let actual = actual_resource_descriptions(&context.resources);
    if expected != actual {
        return Err(format!(
            "particle executor `{}` resource contract mismatch for pass `{}`: expected {:?}, got {:?}",
            contract.executor_id, context.pass_name, expected, actual
        ));
    }

    Ok(())
}

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

fn describe_resource(
    name: &str,
    kind: RenderGraphResourceKind,
    access: RenderGraphResourceAccessKind,
) -> String {
    format!("{access:?}:{kind:?}:{name}")
}
