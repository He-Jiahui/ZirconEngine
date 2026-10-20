//! 为类型化错误审查集中声明路径、子模块与锚点清单；消费者把这些值用于源码检查，清单中的名称不证明对应行为已执行。
pub(super) const TYPED_ERROR_PARENT_PATHS: &[(&str, &str)] = &[
    (
        "typed-error convergence parent",
        "tests/contracts/review/errors/mod.rs",
    ),
    (
        "asset loader typed-error parent",
        "tests/contracts/review/errors/asset_loaders.rs",
    ),
    (
        "asset records typed-error parent",
        "tests/contracts/review/errors/asset_records.rs",
    ),
    (
        "native typed-error parent",
        "tests/contracts/review/errors/loader.rs",
    ),
    (
        "native ABI typed-error parent",
        "tests/contracts/review/errors/loader/abi_surfaces.rs",
    ),
    (
        "native plugin descriptor typed-error parent",
        "tests/review/errors/loader/abi_surfaces/plugin_descriptor.rs",
    ),
    (
        "native live-host typed-error parent",
        "tests/contracts/review/errors/loader/live_host.rs",
    ),
    (
        "native live-host lifecycle-paths typed-error parent",
        "tests/review/errors/loader/live_host/lifecycle_paths.rs",
    ),
    (
        "native live-host replay-runtime typed-error parent",
        "tests/review/errors/loader/live_host/replay_and_runtime.rs",
    ),
    (
        "native manifest typed-error parent",
        "tests/contracts/review/errors/loader/manifest_sources.rs",
    ),
    (
        "scene world typed-error parent",
        "tests/contracts/review/errors/scene_world.rs",
    ),
    (
        "script host typed-error parent",
        "tests/contracts/review/errors/script_host.rs",
    ),
    (
        "shader prewarm CLI typed-error parent",
        "tests/contracts/review/errors/shader_prewarm_cli.rs",
    ),
    (
        "UI input typed-error parent",
        "tests/contracts/review/errors/ui_input.rs",
    ),
];
