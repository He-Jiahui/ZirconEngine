use super::{
    module_mapping_policy, plan_module_release_for, ModuleMappingPolicy, ProcessModuleReceipt,
    RuntimeImageLedger,
};

fn editor_exit_after_one_play(
    policy: ModuleMappingPolicy,
    pin_succeeded: bool,
) -> RuntimeImageLedger {
    let mut ledger = RuntimeImageLedger::new();
    ledger.load_and_seal(policy, pin_succeeded);
    ledger.load_and_seal(policy, pin_succeeded);
    ledger.release_session_handle();
    ledger.release_session_handle();
    ledger
}

#[test]
fn module_mapping_policy_matches_the_target_os() {
    if cfg!(windows) {
        assert_eq!(
            module_mapping_policy(),
            ModuleMappingPolicy::PinUntilProcessExit
        );
    } else {
        assert_eq!(
            module_mapping_policy(),
            ModuleMappingPolicy::ReleaseOnLastReference
        );
    }
}

#[test]
fn release_plan_pins_only_after_a_successful_windows_pin() {
    assert_eq!(
        plan_module_release_for(ModuleMappingPolicy::ReleaseOnLastReference, true),
        super::ModuleReleasePlan {
            retain_library_handle: true,
            process_receipt: ProcessModuleReceipt::ReleaseOnLastSession,
        }
    );
    assert_eq!(
        plan_module_release_for(ModuleMappingPolicy::PinUntilProcessExit, true).process_receipt,
        ProcessModuleReceipt::PinnedUntilProcessExit
    );
    assert!(
        !plan_module_release_for(ModuleMappingPolicy::PinUntilProcessExit, false)
            .retain_library_handle
    );
}

#[test]
fn play_stop_keeps_the_image_while_the_editor_session_holds_a_reference() {
    let mut ledger = RuntimeImageLedger::new();
    ledger.load_and_seal(ModuleMappingPolicy::ReleaseOnLastReference, false);
    ledger.load_and_seal(ModuleMappingPolicy::ReleaseOnLastReference, false);
    ledger.release_session_handle();
    assert!(ledger.image_mapped());
    ledger.release_session_handle();
    assert!(!ledger.image_mapped());
}

#[test]
fn windows_pin_keeps_the_image_mapped_after_play_and_editor_release() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::PinUntilProcessExit, true);
    assert!(ledger.image_mapped());
}

#[test]
fn failed_windows_pin_leaks_the_session_reference_instead_of_unmapping() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::PinUntilProcessExit, false);
    assert!(ledger.image_mapped());
    assert_eq!(ledger.leaked_handles, 2);
}

#[test]
fn non_windows_policy_unmaps_on_the_last_session_release() {
    let ledger = editor_exit_after_one_play(ModuleMappingPolicy::ReleaseOnLastReference, true);
    assert!(!ledger.image_mapped());
}
