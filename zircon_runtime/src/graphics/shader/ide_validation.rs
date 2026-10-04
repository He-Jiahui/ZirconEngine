use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderIdeWgslModuleValidation {
    pub entry_points: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderIdeWgslCheckError {
    Parse { module_id: String, message: String },
    Validate { module_id: String, message: String },
}

impl Display for ShaderIdeWgslCheckError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { module_id, message } => {
                write!(f, "parse shader IDE WGSL module {module_id}: {message}")
            }
            Self::Validate { module_id, message } => {
                write!(f, "validate shader IDE WGSL module {module_id}: {message}")
            }
        }
    }
}

impl std::error::Error for ShaderIdeWgslCheckError {}

pub fn parse_shader_ide_wgsl_module(
    module_id: &str,
    wgsl_source: &str,
) -> Result<ShaderIdeWgslModuleValidation, ShaderIdeWgslCheckError> {
    let module = parse_shader_ide_wgsl(module_id, wgsl_source)?;
    Ok(ShaderIdeWgslModuleValidation {
        entry_points: shader_entry_points(module),
    })
}

pub fn validate_shader_ide_wgsl_module(
    module_id: &str,
    wgsl_source: &str,
) -> Result<ShaderIdeWgslModuleValidation, ShaderIdeWgslCheckError> {
    let module = parse_shader_ide_wgsl(module_id, wgsl_source)?;
    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );
    validator
        .validate(&module)
        .map_err(|error| ShaderIdeWgslCheckError::Validate {
            module_id: module_id.to_string(),
            message: error.emit_to_string(wgsl_source),
        })?;
    Ok(ShaderIdeWgslModuleValidation {
        entry_points: shader_entry_points(module),
    })
}

fn parse_shader_ide_wgsl(
    module_id: &str,
    wgsl_source: &str,
) -> Result<naga::Module, ShaderIdeWgslCheckError> {
    naga::front::wgsl::parse_str(wgsl_source).map_err(|error| ShaderIdeWgslCheckError::Parse {
        module_id: module_id.to_string(),
        message: error.emit_to_string(wgsl_source),
    })
}

fn shader_entry_points(module: naga::Module) -> Vec<String> {
    module
        .entry_points
        .into_iter()
        .map(|entry_point| entry_point.name)
        .collect()
}

#[cfg(test)]
#[path = "tests/ide_validation.rs"]
mod tests;
