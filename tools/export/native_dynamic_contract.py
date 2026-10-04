"""Shared NativeDynamic export constants."""

from __future__ import annotations


REPORT_FILE_NAME = "report.json"
NATIVE_DYNAMIC_STAGE = "native_dynamic"
NATIVE_DYNAMIC_PACKAGE_REPORT_FILE = "native_dynamic_package.toml"
NATIVE_DYNAMIC_LOADER_MANIFEST = "native_plugins.toml"
NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS = {".dll", ".so", ".dylib"}
NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS = {".pdb", ".dbg", ".dsym"}
NATIVE_DYNAMIC_ARTIFACT_EXTENSIONS = (
    NATIVE_DYNAMIC_LOADABLE_ARTIFACT_EXTENSIONS | NATIVE_DYNAMIC_DEBUG_ARTIFACT_EXTENSIONS
)
NATIVE_DYNAMIC_PLATFORM_ARTIFACT_EXTENSIONS = {
    "windows": {".dll", ".pdb"},
    "linux": {".so", ".dbg"},
    "macos": {".dylib", ".dsym"},
}
NATIVE_DYNAMIC_RESOURCE_DIRS = {"assets", "asset", "resources", "resource"}
NATIVE_DYNAMIC_ABI_STRING_FIELDS = (
    "descriptor_symbol",
    "descriptor_contract",
    "runtime_entry_source",
    "editor_entry_source",
    "host_function_table",
    "entry_report_contract",
    "behavior_contract",
    "state_snapshot_contract",
    "bridge_method_table",
)
NATIVE_DYNAMIC_ABI_V3_EXPECTED_FIELDS = {
    "descriptor_symbol": "zircon_native_plugin_descriptor_v3",
    "descriptor_contract": "NativePluginAbiV3",
    "runtime_entry_source": "NativePluginAbiV3.runtime_entry_name",
    "editor_entry_source": "NativePluginAbiV3.editor_entry_name",
    "host_function_table": "NativePluginHostFunctionTableV3",
    "entry_report_contract": "NativePluginEntryReportV3",
    "behavior_contract": "NativePluginBehaviorV4",
    "state_snapshot_contract": "NativePluginBehaviorV4.save_state/restore_state",
    "bridge_method_table": "NativePluginBridgeMethodTableV3",
}


def native_dynamic_package_directory(package_id: str) -> str:
    sanitized = "".join(
        character
        if character.isascii() and (character.isalnum() or character in "-_")
        else "_"
        for character in package_id
    )
    return sanitized if sanitized else "_"
