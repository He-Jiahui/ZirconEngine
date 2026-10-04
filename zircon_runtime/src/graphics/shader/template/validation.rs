use super::assemble::{
    shader_assembly_source_location_for_line, MaterialShaderTemplateAssembly, ShaderAssemblySegment,
};
use super::reflection::{reflect_validated_shader_module, ShaderTemplateReflection};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct MaterialShaderTemplateValidation {
    pub(crate) entry_points: Vec<String>,
    pub(crate) reflection: ShaderTemplateReflection,
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
    let module_info =
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

    let reflection = reflect_validated_shader_module(&module, &module_info);

    Ok(MaterialShaderTemplateValidation {
        entry_points: reflection
            .entry_points
            .iter()
            .map(|entry_point| entry_point.name.clone())
            .collect(),
        reflection,
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
#[path = "tests/validation.rs"]
mod tests;
