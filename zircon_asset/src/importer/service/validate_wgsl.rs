use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use crate::{AssetImportError, AssetUri};

pub(super) fn validate_wgsl(uri: &AssetUri, source: &str) -> Result<(), AssetImportError> {
    let module = wgsl::parse_str(source).map_err(|error| {
        AssetImportError::ShaderValidation(format!("{uri}: {}", error.emit_to_string(source)))
    })?;
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(&module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{uri}: {error}")))?;
    Ok(())
}
