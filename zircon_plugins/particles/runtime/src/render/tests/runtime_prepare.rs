use super::*;

#[test]
fn particle_runtime_prepare_neutral_frame_preserves_readback_payload() {
    let frame = RenderParticleGpuFrameExtract {
        alive_count: 3,
        spawned_total: 5,
        per_emitter_spawned: vec![2, 3],
        indirect_draw_args: [6, 3, 0, 0],
    };

    assert_eq!(
        neutral_readback_outputs_from_frame(&frame),
        RenderParticleGpuReadbackOutputs {
            alive_count: 3,
            spawned_total: 5,
            debug_flags: 0,
            per_emitter_spawned: vec![2, 3],
            indirect_draw_args: [6, 3, 0, 0],
        }
    );
}

#[test]
fn neutral_readback_never_exceeds_the_cpu_projection_bounds() {
    let frame = RenderParticleGpuFrameExtract {
        alive_count: PARTICLE_GPU_MAX_PARTICLES + 1,
        spawned_total: PARTICLE_GPU_MAX_PARTICLES + 2,
        per_emitter_spawned: vec![1; PARTICLE_GPU_NEUTRAL_MAX_EMITTERS as usize + 1],
        indirect_draw_args: [6, PARTICLE_GPU_MAX_PARTICLES + 3, 0, 0],
    };

    let outputs = neutral_readback_outputs_from_frame(&frame);

    assert_eq!(outputs.alive_count, PARTICLE_GPU_MAX_PARTICLES);
    assert_eq!(outputs.spawned_total, PARTICLE_GPU_MAX_PARTICLES);
    assert_eq!(
        outputs.per_emitter_spawned.len(),
        PARTICLE_GPU_NEUTRAL_MAX_EMITTERS as usize
    );
    assert_eq!(outputs.indirect_draw_args[1], PARTICLE_GPU_MAX_PARTICLES);
}

#[test]
fn particle_runtime_prepare_registration_id_is_stable() {
    let registration = particle_runtime_prepare_collector_registration();

    assert_eq!(registration.collector_id(), COLLECTOR_ID);
}

#[test]
fn neutral_runtime_prepare_uses_persistent_owner_and_static_binding_ids() {
    let source = include_str!("../runtime_prepare.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(source.contains("owner.prepare_neutral_frame("));
    assert!(!source.contains("fn create_external_buffer("));
    assert!(!source.contains("format!(\"{logical_name}:runtime-prepare"));
    assert!(source.contains("register_static_external_buffer_binding_with_backing("));
    assert!(source.contains("particles.gpu.indirect-draw-args:neutral-frame"));
}

#[test]
fn neutral_runtime_prepare_splits_frame_from_mutable_context_borrows() {
    let source = include_str!("../runtime_prepare.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let start = source
        .find("fn collect_neutral_particle_gpu_runtime_prepare")
        .expect("neutral collector must remain present");
    let end = source[start..]
        .find("fn collect_real_particle_gpu_runtime_prepare")
        .map(|offset| start + offset)
        .expect("real collector must follow the neutral collector");
    let neutral = &source[start..end];

    assert!(neutral.contains("context.frame_extract()"));
    assert!(neutral.contains("context.gpu_recording_context()"));
    assert!(!neutral.contains("context.frame_extract;"));
    assert!(!neutral.contains("context.device"));
    assert!(!neutral.contains("context.queue"));
    assert!(!neutral.contains("context.encoder"));
    let outputs = neutral
        .find("let particles = neutral_readback_outputs_from_frame(frame);")
        .expect("bounded outputs must be materialized before mutating context bindings");
    let registration = neutral
        .find("register_neutral_particle_external_buffers(context, bindings);")
        .expect("neutral backing bindings must still be registered");
    assert!(outputs < registration);
}

#[test]
fn readback_capacity_degradation_reuses_an_executed_backend_before_compute() {
    let source = include_str!("../runtime_prepare.rs");
    let capacity_check = "let readback_capacity_available = pending_readbacks";
    let admission_gate = [
        "if !context.",
        "gpu_work_admitted() || !readback_capacity_available {",
    ]
    .concat();
    let retained_bindings = ["owner.", "active_bindings()"].concat();
    let compute_execution = ["owner\n        .", "execute_instances("].concat();
    let capacity_check = source
        .find(capacity_check)
        .expect("particle runtime prepare must retain a local in-flight readback bound");
    let admission_gate = source
        .find(&admission_gate)
        .expect("particle runtime prepare must guard work on admission and local capacity");
    let retained_bindings = source[admission_gate..]
        .find(&retained_bindings)
        .map(|offset| admission_gate + offset)
        .expect("capacity degradation must retain an executed backend binding");
    let compute_execution = source
        .find(&compute_execution)
        .expect("particle runtime prepare must retain its GPU compute path");

    assert!(capacity_check < admission_gate);
    assert!(admission_gate < retained_bindings);
    assert!(retained_bindings < compute_execution);
    assert!(source[capacity_check..admission_gate]
        .contains("RuntimePrepareCollectorContext::MAX_IN_FLIGHT_GPU_READBACK_FRAMES"));
}

#[test]
fn real_runtime_prepare_uses_queue_free_uploads_and_registers_rollback_before_readback() {
    let source = include_str!("../runtime_prepare.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let real_start = source
        .find("fn collect_real_particle_gpu_runtime_prepare")
        .expect("real particle collector");
    let real = &source[real_start..];
    let recording = real
        .find("context.gpu_recording_context()")
        .expect("queue-free runtime prepare recording context");
    let transaction = real
        .find("context.register_frame_transaction(")
        .expect("particle prepared state transaction");
    let readback = real
        .find("enqueue_particle_readback(")
        .expect("particle readback registration");

    assert!(!real.contains("context.queue"));
    assert!(!real.contains("context.device"));
    assert!(!real.contains("context.encoder"));
    assert!(recording < transaction);
    assert!(transaction < readback);
    let readback_publish = real[readback..]
        .find("\"particles.gpu.readback\"")
        .map(|offset| readback + offset)
        .expect("particle readback must publish with the accepted frame");
    assert!(readback < readback_publish);
    assert_eq!(
        real.matches("context.register_frame_transaction(").count(),
        2
    );
    let enqueue = &real[readback..readback_publish];
    assert!(!enqueue.contains("push_back"));
    assert!(real[readback_publish..].contains(".push_back(pending_readback);"));
}

#[test]
fn device_epoch_is_activated_before_particle_early_returns_and_old_readbacks() {
    let source = include_str!("../runtime_prepare.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let neutral_start = source
        .find("fn collect_neutral_particle_gpu_runtime_prepare")
        .expect("neutral particle collector");
    let real_start = source
        .find("fn collect_real_particle_gpu_runtime_prepare")
        .expect("real particle collector");
    let neutral = &source[neutral_start..real_start];
    let real = &source[real_start..];

    assert!(
        neutral
            .find("owner.activate_device_epoch(context.device_epoch())")
            .expect("neutral owner epoch activation")
            < neutral
                .find("owner.deactivate()")
                .expect("neutral deactivation")
    );
    let activation = real
        .find(".activate_device_epoch(context.device_epoch())")
        .expect("real owner epoch activation");
    let completed_readback = real
        .find("take_completed_particle_readback(pending_readbacks)")
        .expect("completed readback consumption");
    let admission = real
        .find("if !context.gpu_work_admitted()")
        .expect("GPU admission early return");

    assert!(activation < completed_readback);
    assert!(completed_readback < admission);
    assert!(real[activation..completed_readback].contains("if device_epoch_changed"));
    assert!(real[activation..completed_readback].contains(".clear();"));
}
