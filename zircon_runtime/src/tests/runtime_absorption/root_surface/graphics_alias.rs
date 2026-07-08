use super::inventory::{LIB_RS, ROOT_SURFACE_DOC, RUNTIME_02_PLAN, RUNTIME_INDEX};

#[test]
fn graphics_alias_debt_is_removed_from_runtime_root() {
    for retired_private_alias in [
        "pub(crate) use graphics::pipeline::RendererFeatureReferenceListKind;",
        "pub(crate) use graphics::scene::{",
        "pub(crate) use graphics::{",
    ] {
        assert!(
            !LIB_RS.contains(retired_private_alias),
            "Runtime 02 M3 cutover removed `{retired_private_alias}` from the crate root"
        );
    }

    assert!(
        !LIB_RS.contains("pub use graphics::"),
        "graphics symbols must not be flattened as public root exports"
    );
    assert!(
        !LIB_RS.contains("#[allow(unused_imports)]"),
        "crate root should not need unused-import allowances after removing graphics aliases"
    );

    for required_plan_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "root_surface.md",
        "M3 `lib.rs` graphics alias removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document root-surface guard anchor `{required_plan_anchor}`"
        );
    }
    for required_index_anchor in [
        "Runtime 02 root graphics alias block removal",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "root_surface.md",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document root-surface guard anchor `{required_index_anchor}`"
        );
    }
}

#[test]
fn graphics_type_alias_debt_symbols_are_only_available_through_graphics_namespace() {
    let retired_root_alias_symbols = [
        "RendererFeatureReferenceListKind",
        "GraphicsError",
        "SceneRenderer",
        "WgpuRenderFramework",
        "ViewportFrame",
        "HybridGiRuntimeProvider",
        "VirtualGeometryRuntimeProvider",
        "SolariRuntimeProvider",
    ];
    for retired_root_alias_symbol in retired_root_alias_symbols {
        assert!(
            !LIB_RS.contains(retired_root_alias_symbol),
            "Runtime 02 M3.2 cutover removed root graphics alias symbol `{retired_root_alias_symbol}`"
        );
    }

    let crate_private_graphics_alias_blocks = LIB_RS
        .lines()
        .filter(|line| line.trim_start().starts_with("pub(crate) use graphics"))
        .count();
    assert_eq!(
        crate_private_graphics_alias_blocks, 0,
        "Runtime 02 M3.2 should leave no crate-private graphics alias blocks in lib.rs"
    );
    assert!(
        !LIB_RS
            .lines()
            .any(|line| line.trim_start().starts_with("pub use graphics")),
        "M3.2 type alias debt must not become a public root graphics export"
    );

    for required_doc_anchor in [
        "M3 Alias Cutover",
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "crate-visible graphics alias debt 0/0",
        "Graphics and render callers must use `crate::graphics::...`",
    ] {
        assert!(
            ROOT_SURFACE_DOC.contains(required_doc_anchor),
            "root surface doc must document M3/M3.2 alias cutover anchor `{required_doc_anchor}`"
        );
    }

    for required_plan_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "M3 `lib.rs` graphics alias removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_02_PLAN.contains(required_plan_anchor),
            "Runtime 02 plan must document M3/M3.2 alias cutover anchor `{required_plan_anchor}`"
        );
    }

    for required_index_anchor in [
        "graphics_alias_block_removed_static_passed_cargo_pending",
        "Runtime 02 root graphics alias block removal",
        "crate-visible graphics alias debt 0/0",
    ] {
        assert!(
            RUNTIME_INDEX.contains(required_index_anchor),
            "runtime index must document M3/M3.2 alias cutover anchor `{required_index_anchor}`"
        );
    }
}
