use super::inventory::{LIB_RS, PRELUDE_RS};

#[test]
fn runtime_crate_root_public_surface_stays_curated() {
    let public_modules: Vec<_> = LIB_RS
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|decl| decl.trim_end_matches(';'))
        .collect();
    let expected_modules = [
        "core",
        "diagnostic_log",
        "dynamic_api",
        "engine_module",
        "prelude",
        "animation",
        "asset",
        "scene",
        "ui",
        "graphics",
        "render_graph",
        "rhi",
        "builtin",
        "foundation",
        "input",
        "navigation",
        "platform",
        "plugin",
        "script",
    ];
    assert_eq!(public_modules.as_slice(), &expected_modules);
    assert!(
        !LIB_RS.contains("rhi_wgpu"),
        "the physical zr_rhi_wgpu crate must not retain a runtime crate-root module"
    );

    let public_use_count = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub use "))
        .count();
    assert_eq!(
        public_use_count, 2,
        "crate root should keep only resource and reflection macro public re-export blocks"
    );

    for required in [
        "pub use crate::core::resource;",
        "pub use zircon_runtime_reflection_macros::{",
    ] {
        assert!(
            LIB_RS.contains(required),
            "crate root is missing expected curated re-export `{required}`"
        );
    }

    for forbidden in [
        "pub use graphics::",
        "pub use render_graph::",
        "pub use rhi::",
        "pub use rhi_wgpu::",
        "pub use ui::",
        "pub use input::",
        "pub use scene::",
        "pub use asset::",
        "pub use builtin::{",
        "pub use crate::builtin::{",
        "pub use plugin::",
        "pub use core::{",
        "pub use crate::core::{",
    ] {
        assert!(
            !LIB_RS.contains(forbidden),
            "crate root must not flatten subsystem namespace `{forbidden}`"
        );
    }
    for non_root_symbol in [
        "RuntimeModuleLoadReport",
        "RuntimeModuleLoadDiagnostic",
        "RuntimePluginId",
        "RuntimeTargetMode",
    ] {
        assert!(
            !LIB_RS.contains(non_root_symbol),
            "Runtime 02 root-surface cutover keeps `{non_root_symbol}` under its domain owner"
        );
        assert!(
            !PRELUDE_RS.contains(non_root_symbol),
            "Runtime 02 root-surface cutover must not re-expose `{non_root_symbol}` through the runtime prelude"
        );
    }

    assert!(
        LIB_RS.lines().count() <= 45,
        "crate root should stay thin after the Runtime 02 M3 graphics alias block cutover"
    );
}
