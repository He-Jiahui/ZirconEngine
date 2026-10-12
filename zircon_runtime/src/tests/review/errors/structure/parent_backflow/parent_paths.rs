//! 为类型化错误审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
pub(super) const TYPED_ERROR_PARENT_PATHS: &[(&str, &str)] = &[
    (
        "typed-error convergence parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/mod.rs",
    ),
    (
        "asset loader typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
    ),
    (
        "asset records typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_records.rs",
    ),
    (
        "native typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader.rs",
    ),
    (
        "native ABI typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces.rs",
    ),
    (
        "native plugin descriptor typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor.rs",
    ),
    (
        "native live-host typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host.rs",
    ),
    (
        "native live-host lifecycle-paths typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/lifecycle_paths.rs",
    ),
    (
        "native live-host replay-runtime typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime.rs",
    ),
    (
        "native manifest typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/manifest_sources.rs",
    ),
    (
        "scene world typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/scene_world.rs",
    ),
    (
        "script host typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/script_host.rs",
    ),
    (
        "shader prewarm CLI typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli.rs",
    ),
    (
        "UI input typed-error parent",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/ui_input.rs",
    ),
];
