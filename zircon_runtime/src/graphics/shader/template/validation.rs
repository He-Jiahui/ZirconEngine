use super::assemble::{
    MaterialShaderTemplateAssembly, ShaderAssemblySegment, shader_assembly_source_location_for_line,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialShaderTemplateValidation {
    pub(crate) entry_points: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum ShaderTemplateValidationError {
    Parse { message: String },
    Validate { message: String },
}

pub(crate) fn validate_material_shader_template_wgsl(
    wgsl_source: &str,
) -> Result<MaterialShaderTemplateValidation, ShaderTemplateValidationError> {
    validate_material_shader_template_wgsl_with_segments(wgsl_source, &[])
}

pub(crate) fn validate_material_shader_template_assembly(
    assembly: &MaterialShaderTemplateAssembly,
) -> Result<MaterialShaderTemplateValidation, ShaderTemplateValidationError> {
    validate_material_shader_template_wgsl_with_segments(&assembly.wgsl_source, &assembly.segments)
}

pub(crate) fn validate_material_shader_template_wgsl_with_segments(
    wgsl_source: &str,
    segments: &[ShaderAssemblySegment],
) -> Result<MaterialShaderTemplateValidation, ShaderTemplateValidationError> {
    let module = naga::front::wgsl::parse_str(wgsl_source).map_err(|error| {
        let message = remap_shader_diagnostic_message(
            error.emit_to_string(wgsl_source),
            error
                .location(wgsl_source)
                .map(|location| (location.line_number, location.line_position)),
            segments,
        );
        ShaderTemplateValidationError::Parse { message }
    })?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .map_err(|error| ShaderTemplateValidationError::Validate {
            message: remap_shader_diagnostic_message(
                error.emit_to_string(wgsl_source),
                error
                    .location(wgsl_source)
                    .map(|location| (location.line_number, location.line_position)),
                segments,
            ),
        })?;

    Ok(MaterialShaderTemplateValidation {
        entry_points: module
            .entry_points
            .iter()
            .map(|entry_point| entry_point.name.clone())
            .collect(),
    })
}

pub(crate) fn validate_shader_variant_prewarm_wgsl(
    wgsl_source: &str,
) -> Result<MaterialShaderTemplateValidation, ShaderTemplateValidationError> {
    validate_material_shader_template_wgsl(wgsl_source)
}

fn remap_shader_diagnostic_message(
    mut message: String,
    location: Option<(u32, u32)>,
    segments: &[ShaderAssemblySegment],
) -> String {
    let Some((line, column)) = location else {
        return message;
    };
    let Some(source_location) = shader_assembly_source_location_for_line(segments, line) else {
        return message;
    };
    message.push_str(&format!(
        "\nZircon shader source: {}:{}:{} (assembled line {})",
        source_location.module_id,
        source_location.local_line,
        column,
        source_location.assembled_line
    ));
    message
}

#[cfg(test)]
mod tests {
    use crate::core::framework::render::{
        GEOMETRY_SOURCE_ID_STATIC_MESH, ShaderPassType, builtin_geometry_source_descriptor,
    };

    use super::{ShaderTemplateValidationError, validate_material_shader_template_assembly};
    use crate::graphics::shader::template::assemble::{
        MaterialShaderTemplateRequest, assemble_material_shader_template,
    };

    const INVALID_USER_SURFACE: &str = r#"
fn user_surface(input: ZrVertexOutput) -> ZrSurfaceOutput {
    let bad = vec4<f32>(1.0;
    return zr_surface_from_base_color(input.color + bad);
}
"#;

    #[test]
    fn shader_template_validation_remaps_parse_errors_to_source_segment() {
        let geometry_source = builtin_geometry_source_descriptor(GEOMETRY_SOURCE_ID_STATIC_MESH)
            .expect("static geometry source");
        let assembly = assemble_material_shader_template(
            MaterialShaderTemplateRequest::new(
                geometry_source,
                ShaderPassType::Forward,
                INVALID_USER_SURFACE,
                "user_surface",
            )
            .with_material_surface_module_id("project::materials::invalid"),
        )
        .expect("template assembly");

        let error = validate_material_shader_template_assembly(&assembly)
            .expect_err("invalid user WGSL should fail");
        let ShaderTemplateValidationError::Parse { message } = error else {
            panic!("expected parse error");
        };
        assert!(
            message.contains("Zircon shader source: project::materials::invalid:"),
            "{message}"
        );
    }
}
