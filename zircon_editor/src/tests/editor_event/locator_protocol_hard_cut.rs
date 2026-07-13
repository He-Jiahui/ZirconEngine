#[test]
fn editor_asset_identity_protocol_uses_locator_fields_only() {
    let protocol_sources = [
        include_str!("../../core/editor_event/types.rs"),
        include_str!("../../ui/binding/asset/command.rs"),
        include_str!("../../ui/binding/asset/codec.rs"),
        include_str!("../../ui/binding/animation/command.rs"),
        include_str!("../../ui/binding/animation/codec.rs"),
        include_str!("../../ui/binding_dispatch/asset/asset_host_event.rs"),
        include_str!("../../ui/binding_dispatch/asset/dispatch.rs"),
        include_str!("../../ui/binding_dispatch/animation/animation_host_event.rs"),
        include_str!("../../ui/binding_dispatch/animation/dispatch.rs"),
        include_str!("../../ui/binding_dispatch/editor_event_normalization.rs"),
        include_str!("../../ui/host/editor_event_execution/asset_event.rs"),
        include_str!("../../ui/host/editor_event_execution/animation_event.rs"),
        include_str!("../../ui/host/animation_editor_sessions/editing.rs"),
    ];
    let legacy_identity_fields = [
        concat!("asset_", "path"),
        concat!("graph_", "path"),
        concat!("state_machine_", "path"),
    ];

    for source in protocol_sources {
        for legacy_field in legacy_identity_fields {
            assert!(
                !source.contains(legacy_field),
                "editor asset identity protocol still contains legacy field `{legacy_field}`"
            );
        }
    }
}
