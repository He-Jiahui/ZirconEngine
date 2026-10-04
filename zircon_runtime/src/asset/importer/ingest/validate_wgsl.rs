use naga::front::wgsl;
use naga::valid::{Capabilities, ValidationFlags, Validator};

use crate::asset::{AssetImportError, AssetUri};

// 独立 WGSL 来源先解析并执行 Naga 全量验证；返回模块供 shader 反射和 package 导入复用，
// URI 附在诊断中以区分同一 package 内的不同源码。
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

// shader 导入对已有 Naga 模块再次验证，确保后续反射看到的模块符合相同能力约束。
pub(super) fn validate_naga_module(
    uri: &AssetUri,
    module: &naga::Module,
) -> Result<naga::valid::ModuleInfo, AssetImportError> {
    let mut validator = Validator::new(ValidationFlags::all(), Capabilities::all());
    validator
        .validate(module)
        .map_err(|error| AssetImportError::ShaderValidation(format!("{uri}: {error}")))
}
