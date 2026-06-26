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
    let module = naga::front::wgsl::parse_str(wgsl_source).map_err(|error| {
        ShaderTemplateValidationError::Parse {
            message: error.emit_to_string(wgsl_source),
        }
    })?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .map_err(|error| ShaderTemplateValidationError::Validate {
            message: format!("{error:?}"),
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
