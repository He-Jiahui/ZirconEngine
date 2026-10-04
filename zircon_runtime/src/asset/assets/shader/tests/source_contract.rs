use super::*;

fn entry(name: &str, stage: &str) -> ShaderEntryPointAsset {
    ShaderEntryPointAsset {
        name: name.to_string(),
        stage: stage.to_string(),
    }
}

#[test]
fn material_function_contract_ignores_commented_anchors() {
    let source = r#"
// fn zr_material_surface() {}
/* fn zr_material_surface() {} */
fn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    return zr_surface_from_base_color(input.color);
}
"#;

    assert_eq!(
        classify_surface_source_contract(source, &[]),
        Ok(ShaderSurfaceSourceContract::MaterialFunction)
    );
}

#[test]
fn duplicate_material_function_is_rejected() {
    assert_eq!(
        classify_surface_source_contract(
            "fn zr_material_surface() {}\nfn zr_material_surface() {}",
            &[],
        ),
        Err(ShaderSurfaceSourceContractError::DuplicateMaterialFunction)
    );
}

#[test]
fn material_function_cannot_also_publish_executable_entry_points() {
    assert_eq!(
        classify_surface_source_contract(
            "fn zr_material_surface() {}\n@fragment fn fs_main() {}",
            &[entry("fs_main", "fragment")],
        ),
        Err(ShaderSurfaceSourceContractError::MaterialFunctionOwnsExecutableEntryPoints)
    );
}

#[test]
fn material_function_requires_the_template_call_signature() {
    assert_eq!(
        classify_surface_source_contract("fn zr_material_surface() {}", &[]),
        Err(ShaderSurfaceSourceContractError::MaterialFunctionAbiMismatch)
    );
    assert_eq!(
        classify_surface_source_contract(
            "fn zr_material_surface(input: ZrVertexOutput) -> vec4<f32> { return vec4<f32>(); }",
            &[],
        ),
        Err(ShaderSurfaceSourceContractError::MaterialFunctionAbiMismatch)
    );
    assert_eq!(
        classify_surface_source_contract(
            "somefn zr_material_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {}",
            &[],
        ),
        Err(ShaderSurfaceSourceContractError::MissingMaterialFunction)
    );
    assert_eq!(
        classify_surface_source_contract(
            "fn zr_material_surface(input: ZrSurfaceInput) -> ZrSurfaceOutput {}",
            &[],
        ),
        Ok(ShaderSurfaceSourceContract::MaterialFunction)
    );
}

#[test]
fn executable_full_pass_is_not_a_surface_material_contract() {
    assert_eq!(
        classify_surface_source_contract(
            "@vertex fn vs_main() {}\n@fragment fn fs_main() {}",
            &[entry("vs_main", "vertex"), entry("fs_main", "fragment")],
        ),
        Err(ShaderSurfaceSourceContractError::MissingMaterialFunction)
    );
}
