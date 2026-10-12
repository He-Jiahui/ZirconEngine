use super::{assert_contains_all, read_repo, read_runtime_src};

#[test]
fn runtime_15_render_asset_residency_manager_delegates_recovery_and_ticket_issuance() {
    let manager = read_runtime_src("graphics/scene/resources/render_asset_residency/manager.rs");
    let recovery = read_runtime_src(
        "graphics/scene/resources/render_asset_residency/manager/device_recovery.rs",
    );
    let ticket_issuance = read_runtime_src(
        "graphics/scene/resources/render_asset_residency/manager/ticket_issuance.rs",
    );
    let design = read_repo(
        "docs/plans/zircon_runtime/render/13/2026-08-27-render-asset-device-generation-recovery-design.md",
    );

    assert_contains_all(
        "render asset residency root delegates cold-path recovery and ticket ownership",
        &manager,
        &["pub(super) mod device_recovery;", "mod ticket_issuance;"],
    );
    assert_contains_all(
        "device recovery child owns generation replacement",
        &recovery,
        &[
            "pub(crate) fn recover_device_epoch(",
            "resources.sort_by_key",
            "reserve_ticket_ids(seeds.len())",
            "abandon_for_device_recovery(replacement)",
            "RenderAssetDeviceRecoveryReport",
        ],
    );
    assert_contains_all(
        "ticket issuance child owns batch reservation and publication",
        &ticket_issuance,
        &[
            "fn issue_reference_change_tickets(",
            "fn issue_reconciliation_tickets(",
            "fn reserve_ticket_ids(",
        ],
    );
    assert!(
        !manager.contains("fn recover_device_epoch(")
            && !manager.contains("fn reserve_ticket_ids("),
        "manager.rs must remain the state/orchestration root rather than absorbing cold-path owners"
    );
    assert_contains_all(
        "device recovery design records cold-path complexity and dynamic evidence limits",
        &design,
        &["`O(N log N)`", "source-only", "docs/tests/runtime/render"],
    );

    for (path, source) in [
        ("render_asset_residency/manager.rs", manager.as_str()),
        (
            "render_asset_residency/manager/device_recovery.rs",
            recovery.as_str(),
        ),
        (
            "render_asset_residency/manager/ticket_issuance.rs",
            ticket_issuance.as_str(),
        ),
    ] {
        let line_count = source.lines().count();
        assert!(
            line_count < 800,
            "{path} should stay below the Runtime 15 production-file soft budget; got {line_count} lines"
        );
    }
}
