use crate::core::framework::scene::ComponentTypeDescriptor;

use super::*;

#[test]
fn native_system_access_authority_resolves_known_owned_ids() {
    let plan = NativeSystemAccessPlan::from_manifest(
        NativePluginRegistrationThreadAffinity::WorkerSafe,
        &[
            "read:component:physics.Body".to_string(),
            "write:resource:physics.solver".to_string(),
        ],
        &[NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
    )
    .unwrap();
    let authority = NativeSystemAccessAuthority::new(
        "physics",
        ["physics.Body".to_string()],
        ["physics.solver".to_string()],
        [NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
    );
    authority.authorize(&plan).unwrap();
    let mut world = World::empty();
    world
        .register_component_type(ComponentTypeDescriptor::new(
            "physics.Body",
            "physics",
            "Physics Body",
        ))
        .unwrap();

    let access = plan.compile(&mut world).unwrap();

    assert!(!access.has_conservative_world_access());
    assert!(world
        .registered_external_resource_id("physics.solver")
        .is_some());
}

#[test]
fn native_system_access_authority_rejects_foreign_or_ungranted_worker_access() {
    let plan = NativeSystemAccessPlan::from_manifest(
        NativePluginRegistrationThreadAffinity::WorkerSafe,
        &["read:component:render.Visible".to_string()],
        &[NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
    )
    .unwrap();
    let no_grants =
        NativeSystemAccessAuthority::new("physics", ["render.Visible".to_string()], [], []);
    assert!(matches!(
        no_grants.authorize(&plan),
        Err(NativeSystemAccessAuthorityError::WorkerSafeCapabilityNotGranted)
    ));
    let worker_only = NativeSystemAccessAuthority::new(
        "physics",
        ["render.Visible".to_string()],
        [],
        [NATIVE_SYSTEM_WORKER_SAFE_CAPABILITY.to_string()],
    );
    assert!(matches!(
        worker_only.authorize(&plan),
        Err(NativeSystemAccessAuthorityError::CapabilityNotGranted {
            required_capability,
            ..
        }) if required_capability == "runtime.native.ecs.component.read.render.Visible"
    ));
}
