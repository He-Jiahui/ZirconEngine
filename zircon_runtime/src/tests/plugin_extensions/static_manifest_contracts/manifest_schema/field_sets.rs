pub(super) const KNOWN_TOP_LEVEL_FIELDS: [&str; 32] = [
    "asset_importers",
    "asset_roots",
    "capabilities",
    "capability_statuses",
    "category",
    "components",
    "content_roots",
    "default_packaging",
    "dependencies",
    "description",
    "display_name",
    "distribution",
    "event_catalogs",
    "feature_extensions",
    "geometry_sources",
    "id",
    "maturity",
    "modules",
    "optional_features",
    "options",
    "package_company",
    "package_kind",
    "package_name",
    "package_prefix",
    "provides_interfaces",
    "sdk_api_version",
    "shader_permutation",
    "shading_models",
    "supported_platforms",
    "supported_targets",
    "ui_components",
    "version",
];

pub(super) const KNOWN_ASSET_IMPORTER_FIELDS: [&str; 9] = [
    "additional_output_kinds",
    "full_suffixes",
    "id",
    "importer_version",
    "output_kind",
    "plugin_id",
    "priority",
    "required_capabilities",
    "source_extensions",
];

pub(super) const KNOWN_CAPABILITY_STATUS_FIELDS: [&str; 5] = [
    "bevy_references",
    "capability",
    "note",
    "status",
    "target_modes",
];

pub(super) const KNOWN_COMPONENT_FIELDS: [&str; 4] =
    ["display_name", "plugin_id", "properties", "type_id"];
pub(super) const KNOWN_COMPONENT_PROPERTY_FIELDS: [&str; 3] = ["editable", "name", "value_type"];
pub(super) const KNOWN_DEPENDENCY_FIELDS: [&str; 4] =
    ["capability", "id", "interfaces", "required"];
pub(super) const KNOWN_EVENT_CATALOG_FIELDS: [&str; 3] = ["events", "namespace", "version"];
pub(super) const KNOWN_EVENT_FIELDS: [&str; 3] = ["display_name", "id", "payload_schema"];
pub(super) const KNOWN_INTERFACE_FIELDS: [&str; 2] = ["id", "methods"];
pub(super) const KNOWN_MODULE_FIELDS: [&str; 10] = [
    "capabilities",
    "crate_name",
    "description",
    "init_level",
    "kind",
    "module_dependencies",
    "name",
    "system_anchors",
    "system_sets",
    "target_modes",
];
pub(super) const KNOWN_OPTION_FIELDS: [&str; 6] = [
    "default_value",
    "display_name",
    "enum_values",
    "key",
    "required_capability",
    "value_type",
];

pub(super) const KNOWN_OPTIONAL_FEATURE_FIELDS: [&str; 10] = [
    "capabilities",
    "default_packaging",
    "dependencies",
    "distribution",
    "display_name",
    "enabled_by_default",
    "id",
    "modules",
    "owner_plugin_id",
    "provider_package_id",
];

pub(super) const KNOWN_OPTIONAL_FEATURE_DEPENDENCY_FIELDS: [&str; 3] =
    ["capability", "plugin_id", "primary"];
pub(super) const KNOWN_GEOMETRY_SOURCE_FIELDS: [&str; 6] = [
    "id",
    "required_bindings",
    "shader_defines",
    "token",
    "vertex_attributes",
    "wgsl_include",
];
pub(super) const KNOWN_SHADING_MODEL_FIELDS: [&str; 6] = [
    "deferred_include",
    "forward_include",
    "gbuffer_encode_include",
    "id",
    "required_channels",
    "token",
];
pub(super) const KNOWN_UI_COMPONENT_FIELDS: [&str; 3] =
    ["component_id", "plugin_id", "ui_document"];
