#[test]
fn builtin_root_stays_structural_after_runtime_module_split() {
    let source = include_str!("../../builtin/mod.rs");

    for required in [
        "mod runtime_modules;",
        "pub use runtime_modules::builtin_runtime_modules;",
    ] {
        assert!(
            source.contains(required),
            "expected builtin/mod.rs to keep structural wiring `{required}`"
        );
    }

    for forbidden in [
        "use std::sync::Arc;",
        "use crate::engine_module::EngineModule;",
        "pub fn builtin_runtime_modules()",
        "fn runtime_extension_modules()",
        "Arc::new(",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected builtin/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}
