use crate::core::framework::render::strip_wgsl_include_directives;

use super::*;

#[test]
fn shader_template_assemblies_move_the_include_manifest() {
    for source in [
        include_str!("../assemble.rs"),
        include_str!("../deferred_gbuffer.rs"),
        include_str!("../taa_reactive_mask.rs"),
    ] {
        assert!(source.contains("registry.into_manifest()"));
        assert!(!source.contains(concat!("registry.include_", "tokens()")));
        assert!(!source.contains(concat!("registry.content_", "hashes()")));
    }
}

#[test]
fn builtin_lightmap_resolves_irradiance_volume_dependency_first() {
    let registry = ShaderModuleRegistry::with_builtin_modules();
    let resolved = registry
        .resolve_roots([LIGHTMAP_INCLUDE_TOKEN.to_string()])
        .expect("builtin lightmap dependency graph should resolve");

    assert_eq!(
        resolved
            .ordered_sources
            .iter()
            .map(|module| module.token.as_str())
            .collect::<Vec<_>>(),
        vec![IRRADIANCE_VOLUME_INCLUDE_TOKEN, LIGHTMAP_INCLUDE_TOKEN]
    );
}

#[test]
fn builtin_pbr_extras_is_independent_from_volumetric_uv_helpers() {
    let registry = ShaderModuleRegistry::with_builtin_modules();
    let resolved = registry
        .resolve_roots([PBR_EXTRAS_INCLUDE_TOKEN.to_string()])
        .expect("builtin PBR extras dependency graph should resolve");

    assert_eq!(
        resolved
            .ordered_sources
            .iter()
            .map(|module| module.token.as_str())
            .collect::<Vec<_>>(),
        vec![PBR_COMMON_INCLUDE_TOKEN, PBR_EXTRAS_INCLUDE_TOKEN]
    );
}

#[test]
fn builtin_normal_include_resolves_bc5_reconstruction_helpers() {
    let registry = ShaderModuleRegistry::with_builtin_modules();
    let resolved = registry
        .resolve_roots([NORMAL_INCLUDE_TOKEN.to_string()])
        .expect("builtin normal include should resolve");

    assert_eq!(resolved.ordered_sources.len(), 1);
    assert_eq!(resolved.ordered_sources[0].token, NORMAL_INCLUDE_TOKEN);
    assert!(
        resolved.ordered_sources[0]
            .source
            .contains("zr_reconstruct_bc5_normal")
    );
}

#[test]
fn root_scoped_registry_constructs_only_the_requested_dependency_closure() {
    let project_include = ShaderTemplateInclude::new(
        "project::surface",
        "#include <zr_pbr_extras.wgsl>\nfn project_surface() {}",
    );
    let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
        ["project::surface".to_string()],
        [project_include],
    );

    assert!(registry.modules.contains_key("project::surface"));
    assert!(registry.modules.contains_key(PBR_COMMON_INCLUDE_TOKEN));
    assert!(registry.modules.contains_key(PBR_EXTRAS_INCLUDE_TOKEN));
    assert!(!registry.modules.contains_key(SHADOW_INCLUDE_TOKEN));
    assert!(!registry.modules.contains_key(VOLUMETRIC_INCLUDE_TOKEN));
}

#[test]
fn root_scoped_registry_prefers_supplied_source_over_builtin() {
    let disabled_volumetric = ShaderTemplateInclude::new(
        VOLUMETRIC_INCLUDE_TOKEN,
        "fn zr_apply_volumetric_fog(color: vec3<f32>) -> vec3<f32> { return color; }",
    );
    let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
        [VOLUMETRIC_INCLUDE_TOKEN.to_string()],
        [disabled_volumetric.clone()],
    );

    let resolved = registry
        .resolve_roots([VOLUMETRIC_INCLUDE_TOKEN.to_string()])
        .expect("supplied source should replace the builtin module for the same token");

    assert_eq!(resolved.ordered_sources, vec![disabled_volumetric]);
}

#[test]
fn root_scoped_registry_preserves_unknown_dependency_errors() {
    let project_include = ShaderTemplateInclude::new(
        "project::surface",
        "#include <project::missing>\nfn project_surface() {}",
    );
    let registry = ShaderModuleRegistry::with_builtin_modules_for_roots(
        ["project::surface".to_string()],
        [project_include],
    );

    let error = registry
        .resolve_roots(["project::surface".to_string()])
        .expect_err("an unknown transitive dependency must remain an assembly error");

    assert_eq!(
        error,
        ShaderModuleResolutionError::UnknownModule {
            token: "project::missing".to_string(),
        }
    );
}

#[test]
fn builtin_standard_pbr_resolves_advanced_lighting_dependencies_first() {
    let registry = ShaderModuleRegistry::with_builtin_modules();
    let resolved = registry
        .resolve_roots([STANDARD_PBR_SHADING_INCLUDE_TOKEN.to_string()])
        .expect("builtin Standard PBR dependency graph should resolve");

    assert_eq!(
        resolved
            .ordered_sources
            .iter()
            .map(|module| module.token.as_str())
            .collect::<Vec<_>>(),
        vec![
            PBR_COMMON_INCLUDE_TOKEN,
            PBR_EXTRAS_INCLUDE_TOKEN,
            LIGHT_COOKIE_INCLUDE_TOKEN,
            STANDARD_PBR_SHADING_INCLUDE_TOKEN,
        ]
    );
}

#[test]
fn shader_module_registry_resolves_transitive_modules_once() {
    let mut registry = ShaderModuleRegistry::with_builtin_modules();
    registry.register(ShaderTemplateInclude::new(
        "project::a",
        "#include <project::b>\nfn a_value() -> f32 { return b_value(); }",
    ));
    registry.register(ShaderTemplateInclude::new(
        "project::b",
        "fn b_value() -> f32 { return 1.0; }",
    ));

    let resolved = registry
        .resolve_for_source("#include <project::a>\n#include <project::b>")
        .expect("modules should resolve");

    assert_eq!(
        resolved
            .ordered_sources
            .iter()
            .map(|module| module.token.as_str())
            .collect::<Vec<_>>(),
        vec!["project::b", "project::a"]
    );
    assert!(!resolved.content_hash.is_empty());
}

#[test]
fn shader_module_registry_reports_cycles() {
    let mut registry = ShaderModuleRegistry::with_builtin_modules();
    registry.register(ShaderTemplateInclude::new(
        "project::a",
        "#include <project::b>",
    ));
    registry.register(ShaderTemplateInclude::new(
        "project::b",
        "#include <project::a>",
    ));

    let error = registry
        .resolve_for_source("#include <project::a>")
        .expect_err("cycle should fail");

    assert_eq!(
        error,
        ShaderModuleResolutionError::CircularDependency {
            cycle: vec![
                "project::a".to_string(),
                "project::b".to_string(),
                "project::a".to_string(),
            ],
        }
    );
}

#[test]
fn shader_module_registry_strips_include_directives() {
    let source = "// #include <ignored>\n#include <self::material>\nfn surface() {}";

    assert_eq!(
        wgsl_include_paths(source),
        vec!["self::material".to_string()]
    );
    assert_eq!(
        strip_wgsl_include_directives(source),
        "// #include <ignored>\nfn surface() {}"
    );
}
