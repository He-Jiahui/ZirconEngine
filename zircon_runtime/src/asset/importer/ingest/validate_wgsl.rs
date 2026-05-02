use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use crate::asset::{AssetImportError, AssetUri};

pub(super) fn validate_wgsl(
    uri: &AssetUri,
    source: &str,
) -> Result<(naga::Module, naga::valid::ModuleInfo), AssetImportError> {
    let module = wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!("{uri}: {}", error.emit_to_string(source)))
    })?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    let info = validator
        .validate(&module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{uri}: {error}")))?;
    Ok((module, info))
}

pub(super) fn validate_naga_module(
    uri: &AssetUri,
    module: &naga::Module,
) -> Result<naga::valid::ModuleInfo, AssetImportError> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{uri}: {error}")))
}
