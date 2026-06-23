use crate::plugin::PluginModuleKind;

use super::super::abi_declarations::ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3;
use super::diagnostics::{invalid_diagnostic, module_kind_label, ValidationDiagnostic};

pub const ZIRCON_NATIVE_COMMAND_MANIFEST_SCHEMA_V3: &str = "zircon.native.command-manifest/3";
pub const ZIRCON_NATIVE_EVENT_MANIFEST_SCHEMA_V3: &str = "zircon.native.event-manifest/3";
pub const ZIRCON_NATIVE_REGISTRATION_MANIFEST_SCHEMA_V3: &str =
    "zircon.native.registration-manifest/3";

pub(super) fn validate_v3_schema(
    diagnostics: &mut Vec<ValidationDiagnostic>,
    abi_version: u32,
    plugin_id: &str,
    module_kind: PluginModuleKind,
    field_name: &str,
    schema: Option<&str>,
    manifest: Option<&str>,
    expected_schema: &str,
) {
    if abi_version != ZIRCON_NATIVE_PLUGIN_ABI_VERSION_V3 {
        return;
    }
    let Some(schema) = schema.map(str::trim).filter(|schema| !schema.is_empty()) else {
        return;
    };
    if schema != expected_schema {
        diagnostics.push(invalid_diagnostic(format!(
            "native plugin {plugin_id} {} {field_name} is unsupported: {schema}; expected {expected_schema}",
            module_kind_label(module_kind)
        )));
        return;
    }
    if !has_manifest_text(manifest) {
        diagnostics.push(invalid_diagnostic(format!(
            "native plugin {plugin_id} {} declares {field_name} {schema} but provides no manifest text",
            module_kind_label(module_kind)
        )));
    }
}

pub(super) fn has_manifest_text(manifest: Option<&str>) -> bool {
    manifest
        .into_iter()
        .flat_map(str::lines)
        .map(str::trim)
        .any(|line| !line.is_empty())
}
