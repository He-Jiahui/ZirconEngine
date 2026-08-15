#[test]
fn review_f5_gameplay_host_uses_typed_errors_before_script_host_boundary() {
    let gameplay_host = include_str!("../../../../../script/vm/gameplay_host.rs");
    let error_owner = include_str!("../../../../../script/vm/gameplay_host/error.rs");
    let combat = include_str!("../../../../../script/vm/gameplay_host/combat.rs");
    let lifecycle = include_str!("../../../../../script/vm/gameplay_host/lifecycle.rs");
    let navigation = include_str!("../../../../../script/vm/gameplay_host/navigation.rs");
    let transform = include_str!("../../../../../script/vm/gameplay_host/transform.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_doc =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/host/function_ledger.md");

    assert!(
        gameplay_host.contains("mod error;"),
        "gameplay host root should mount the typed error owner"
    );
    for required in [
        "pub(super) type GameplayHostResult<T> = std::result::Result<T, GameplayHostError>;",
        "pub(super) enum GameplayHostError",
        "Scene(#[from] SceneError)",
        "Navigation(#[from] NavigationError)",
        "Json(#[from] serde_json::Error)",
        "MissingEntity",
        "impl From<GameplayHostError> for ScriptHostError",
    ] {
        assert!(
            error_owner.contains(required),
            "gameplay host typed-error owner should contain `{required}`"
        );
    }

    for (label, source) in [
        ("combat", combat),
        ("lifecycle", lifecycle),
        ("navigation", navigation),
        ("transform", transform),
    ] {
        assert!(
            source.contains("GameplayHostResult"),
            "{label} should use GameplayHostResult for internal fallible work"
        );
        for forbidden in [
            "Result<u64, String>",
            "Result<bool, String>",
            "Result<DamageReport, String>",
            ".map_err(|error| error.to_string())",
            "Err(format!(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not keep lossy String-error branch `{forbidden}`"
            );
        }
    }
}

#[test]
fn review_f5_script_scene_system_uses_typed_errors_before_core_boundary() {
    let scene_system = include_str!("../../../../../script/vm/scene_system.rs");
    let error_owner = include_str!("../../../../../script/vm/scene_system/error.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let host_reflection =
        include_str!("../../../../../../../docs/zircon_runtime/script/vm/zr_vm_host_reflection.md");
    let module_doc =
        include_str!("../../../../../../../docs/zircon_runtime/structure/module-convention.md");

    assert!(
        scene_system.contains("mod error;"),
        "script scene system root should mount the typed error owner"
    );
    assert!(
        scene_system.contains(
            "pub(super) use self::error::{ScriptSceneSystemError, ScriptSceneSystemResult};"
        ),
        "script scene system should expose its VM-scoped typed errors through the sibling facade"
    );
    assert!(
        scene_system.contains("ScriptSceneSystemResult"),
        "script scene system should use ScriptSceneSystemResult for internal fallible work"
    );
    assert!(
        scene_system.contains("ScriptSceneSystemError"),
        "script scene system should use ScriptSceneSystemError before the CoreError boundary"
    );
    for required in [
        "pub(in crate::script::vm) type ScriptSceneSystemResult<T> =",
        "pub(in crate::script::vm) enum ScriptSceneSystemError",
        "Core(#[from] CoreError)",
        "InvalidBindingComponent",
        "ExportCall",
        "source: serde_json::Error",
        "source: VmError",
    ] {
        assert!(
            error_owner.contains(required),
            "script scene system typed-error owner should contain `{required}`"
        );
    }
    for required in [
        "CoreError::Initialization",
        "self.id().to_string()",
        "error.to_string()",
    ] {
        assert!(
            scene_system.contains(required),
            "script scene system should stringify only at the CoreError boundary: `{required}`"
        );
    }
    for forbidden in [
        "Result<(), String>",
        "Result<Vec<EntityScriptBindings>, String>",
        ".map_err(|error| error.to_string())",
        "Err(format!(",
        "format!(\"script binding",
        "format!(\"invalid {SCRIPT_BINDINGS_COMPONENT}",
    ] {
        assert!(
            !scene_system.contains(forbidden),
            "script scene system should not keep lossy String-error branch `{forbidden}`"
        );
    }
}
