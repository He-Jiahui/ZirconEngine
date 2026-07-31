use super::support::collect_rust_files;

#[test]
fn editor_event_owners_are_split_without_the_legacy_aggregate() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let core_root = crate_root.join("core");
    let core_event_root = core_root.join("editor_event");
    let context_root = core_root.join("context");
    let ui_host_root = crate_root.join("ui").join("host");
    let workbench_root = crate_root.join("ui").join("workbench");
    let viewport_root = crate_root.join("scene").join("viewport");

    for deleted in [
        core_event_root.join("runtime.rs"),
        core_event_root.join("runtime"),
        core_root.join("editing").join("operation_state.rs"),
        ui_host_root.join("editor_event_runtime_bootstrap.rs"),
        ui_host_root.join("editor_event_listener_control.rs"),
    ] {
        assert!(
            !deleted.exists(),
            "legacy editor event owner remains: {deleted:?}"
        );
    }

    for required in [
        context_root.join("mod.rs"),
        context_root.join("editor_context.rs"),
        context_root.join("builder.rs"),
        core_event_root
            .join("service")
            .join("editor_event_service.rs"),
        core_event_root.join("service").join("listener_control.rs"),
        core_root.join("editing").join("mod.rs"),
        core_root.join("editing").join("context.rs"),
        core_root.join("editing").join("engine").join("mod.rs"),
        core_root.join("editing").join("engine").join("history.rs"),
        core_root
            .join("editing")
            .join("engine")
            .join("transaction.rs"),
        core_root.join("play").join("bridge.rs"),
        ui_host_root.join("editor_host_event_controller.rs"),
        workbench_root.join("shell_state.rs"),
    ] {
        assert!(
            required.exists(),
            "new editor event owner missing: {required:?}"
        );
    }

    for owner in [
        core_event_root.clone(),
        context_root.clone(),
        core_root.join("play"),
    ] {
        for file in collect_rust_files(&owner) {
            let source = std::fs::read_to_string(&file).expect("core source");
            assert!(
                !source.contains("crate::ui"),
                "headless editor owner must not depend on UI: {file:?}"
            );
        }
    }

    for owner in [
        core_root.clone(),
        crate_root.join("ui"),
        crate_root.join("scene"),
    ] {
        for file in collect_rust_files(&owner) {
            let source = std::fs::read_to_string(&file).expect("editor source");
            for forbidden in [
                "EditorEventRuntime",
                "EditorEventRuntimeState",
                "lock_inner(",
                "editor_event_runtime_state",
            ] {
                assert!(
                    !source.contains(forbidden),
                    "legacy editor event symbol `{forbidden}` remains in {file:?}"
                );
            }
        }
    }

    let core_event_mod =
        std::fs::read_to_string(core_event_root.join("mod.rs")).expect("core event mod");
    assert!(core_event_mod.contains("mod service;"));
    assert!(core_event_mod.contains("pub use service::EditorEventService;"));
    assert!(!core_event_mod.contains("mod runtime;"));

    let ui_host_mod = std::fs::read_to_string(ui_host_root.join("mod.rs")).expect("ui host mod");
    assert!(ui_host_mod.contains("mod editor_host_event_controller;"));
    assert!(
        ui_host_mod.contains("pub use editor_host_event_controller::EditorHostEventController;")
    );
    assert!(!ui_host_mod.contains("mod editor_event_runtime_bootstrap;"));
    assert!(!ui_host_mod.contains("mod editor_event_listener_control;"));
}

#[test]
fn core_remains_independent_from_the_ui_layer() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let core_root = crate_root.join("core");
    let ui_import_root = concat!("use crate", "::ui");
    let ui_module_path = concat!("crate", "::ui", "::");

    for file in collect_rust_files(&core_root) {
        let source = std::fs::read_to_string(&file).expect("core source");
        assert!(
            !source.contains(ui_import_root) && !source.contains(ui_module_path),
            "core source must not depend on the UI layer: {file:?}"
        );
    }
}

#[test]
fn editor_transaction_context_hard_cuts_the_operation_stack_shape() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let core_root = crate_root.join("core");
    let ui_root = crate_root.join("ui");

    for owner in [core_root.clone(), ui_root] {
        for file in collect_rust_files(&owner) {
            let source = std::fs::read_to_string(&file).expect("editor source");
            for forbidden in [
                concat!("EditorOperation", "Stack"),
                concat!("EditorOperation", "StackEntry"),
                concat!("QueryOperation", "Stack"),
                concat!("operation_", "stack("),
            ] {
                assert!(
                    !source.contains(forbidden),
                    "legacy operation history API `{forbidden}` remains in {file:?}"
                );
            }
        }
    }

    assert!(
        !core_root
            .join("editing")
            .join("operation_state.rs")
            .exists(),
        "retired aggregate operation state owner must remain deleted"
    );

    let editing_root =
        std::fs::read_to_string(core_root.join("editing").join("mod.rs")).expect("editing root");
    assert!(editing_root.contains("pub mod engine;"));
    assert!(editing_root.contains("pub(crate) mod context;"));

    let transaction_engine_root =
        std::fs::read_to_string(core_root.join("editing").join("engine").join("mod.rs"))
            .expect("transaction engine root");
    for required in [
        "mod history;",
        "mod transaction;",
        "EditorTransactionEngine",
        "HistoryStore",
    ] {
        assert!(
            transaction_engine_root.contains(required),
            "transaction engine root should expose current owner `{required}`"
        );
    }

    let edit_context = core_root.join("editing").join("context.rs");
    assert!(
        edit_context.exists(),
        "headless edit context owner is missing"
    );
    let edit_context_source = std::fs::read_to_string(edit_context).expect("edit context source");
    assert!(!edit_context_source.contains("crate::ui"));

    let editor_context =
        std::fs::read_to_string(core_root.join("context").join("editor_context.rs"))
            .expect("editor context source");
    assert!(editor_context.contains("transactions: EditorTransactionEngine"));
    assert!(editor_context.contains("pub fn transactions(&self)"));
}

#[test]
fn editor_host_input_translation_uses_shared_dispatch_contracts_without_private_effect_types() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let retained_host_root = crate_root.join("ui").join("retained_host");
    let host_contract_root = retained_host_root.join("host_contract");
    let host_contract_mod_source =
        std::fs::read_to_string(host_contract_root.join("mod.rs")).expect("host contract mod");
    let event_loop_root = host_contract_root.join("window").join("event_loop");
    let platform_input_source = std::fs::read_to_string(event_loop_root.join("platform_input.rs"))
        .expect("editor platform input adapter source");
    let event_loop_source =
        std::fs::read_to_string(event_loop_root.join("events.rs")).expect("event loop source");
    let workspace_root = crate_root
        .parent()
        .and_then(std::path::Path::parent)
        .expect("workspace root");
    let runtime_platform_input_source = std::fs::read_to_string(
        workspace_root
            .join("zircon_runtime")
            .join("src")
            .join("ui")
            .join("platform_input")
            .join("winit_translation.rs"),
    )
    .expect("runtime platform input source");
    let platform_input_tests = crate_root
        .join("tests")
        .join("host")
        .join("retained_window")
        .join("platform_input_translation.rs");

    for required in [
        "UiWindowPlatformInputEvent::keyboard",
        "UiWindowPlatformInputEvent::text",
        "UiWindowPlatformInputEvent::ime_with_cursor_range",
        "UiWindowPlatformInputEvent::pointer",
        "UiPreciseScrollDelta::pixels",
        "UiPreciseScrollDelta::lines",
    ] {
        assert!(
            runtime_platform_input_source.contains(required),
            "expected runtime platform_input to translate host input through shared `{required}`"
        );
    }
    assert!(
        !host_contract_root
            .join("native_input_translation.rs")
            .exists(),
        "expected editor-local native_input_translation.rs entry to be removed"
    );
    let deleted_native_translation_dir = host_contract_root.join("native_input_translation");
    assert!(
        collect_rust_files(&deleted_native_translation_dir).is_empty(),
        "expected editor-local native input translation directory {:?} to contain no Rust sources",
        deleted_native_translation_dir
    );
    for forbidden_export in [
        "native_keyboard_event_to_shared_input",
        "native_ime_event_to_shared_input",
        "native_mouse_wheel_event_to_shared_input",
        "mod native_input_translation;",
    ] {
        assert!(
            !host_contract_mod_source.contains(forbidden_export),
            "expected host_contract/mod.rs to stop exposing editor-local `{forbidden_export}`"
        );
    }
    assert!(
        platform_input_source.contains("translate_winit_window_event")
            && platform_input_source.contains("translate_winit_modifiers"),
        "expected editor event loop adapter to call runtime platform_input translators"
    );
    assert!(
        event_loop_source.contains("translate_platform_input_event(&event)"),
        "expected editor window event loop to feed winit events through runtime platform_input"
    );
    assert!(
        platform_input_tests.exists(),
        "expected editor host runtime platform input tests at {:?}",
        platform_input_tests
    );

    for path in collect_rust_files(&retained_host_root) {
        let source = std::fs::read_to_string(&path).expect("retained host source");
        for forbidden in [
            "HostDispatchReply",
            "HostDispatchEffect",
            "EditorDispatchReply",
            "EditorDispatchEffect",
            "RetainedDispatchReply",
            "RetainedDispatchEffect",
            "PrivateDispatchReply",
            "PrivateDispatchEffect",
            "HostInputEffect",
            "EditorInputEffect",
            "RetainedInputEffect",
        ] {
            assert!(
                !source.contains(forbidden),
                "expected {:?} to avoid host-private event side effect type `{forbidden}`; use zircon_runtime_interface::ui::dispatch reply/effect DTOs instead",
                path.file_name().expect("file name")
            );
        }
    }
}
