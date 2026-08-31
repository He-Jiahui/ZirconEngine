use crate::tests::runtime_absorption::structure_convention::support::read_runtime_src;

#[test]
fn runtime_06_surface_source_contract_is_classified_before_pipeline_resolution() {
    let shader_asset = read_runtime_src("asset/assets/shader/shader_asset.rs");
    let readiness = read_runtime_src("asset/assets/shader/readiness.rs");
    let shader_runtime = read_runtime_src("graphics/scene/resources/runtime/shader_runtime.rs");
    let ensure_shader = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_ensure_shader_source.rs",
    );
    let accessors = read_runtime_src(
        "graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs",
    );

    let source_contract = read_runtime_src("asset/assets/shader/source_contract.rs");

    assert!(shader_asset.contains("surface_source_contract"));
    assert!(source_contract.contains("authored_declaration_matches"));
    assert!(source_contract.contains("MaterialFunctionAbiMismatch"));
    assert!(source_contract.contains("MissingMaterialFunction"));
    assert!(source_contract.contains("material_function_abi_is_valid"));
    assert!(source_contract.contains("executable_full_pass_is_not_a_surface_material_contract"));
    assert!(!source_contract.contains("LegacyFullPass"));
    assert!(!source_contract.contains("MissingMaterialFunctionOrFullPass"));
    assert!(!source_contract.contains("String::from_utf8"));
    assert!(readiness.contains(".surface_source_contract()"));
    assert!(shader_runtime.contains("surface_source_contract:"));
    assert!(shader_runtime.contains("Option<ShaderSurfaceSourceContract>"));
    assert!(
        ensure_shader.contains("let surface_source_contract = shader.surface_source_contract()")
    );
    assert!(ensure_shader.contains("surface_source_contract,"));
    assert!(accessors.contains("ShaderSurfaceSourceContract::MaterialFunction"));
    assert!(!accessors.contains("source.contains(\"fn zr_material_surface\")"));
}
