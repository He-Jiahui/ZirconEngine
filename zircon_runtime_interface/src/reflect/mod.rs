//! 反射公共层承载类型、字段和值的可传输契约；运行时目录和对象适配器负责接纳与执行。

#[path = "editor_hint.rs"]
mod editor_hint;
#[path = "error.rs"]
mod error;
#[path = "field_id.rs"]
mod field_id;
#[path = "field_id_parse_error.rs"]
mod field_id_parse_error;
#[path = "field_info.rs"]
mod field_info;
#[path = "numeric_range.rs"]
mod numeric_range;
#[path = "object_address.rs"]
mod object_address;
#[path = "read_write.rs"]
mod read_write;
#[path = "reflected_value.rs"]
mod reflected_value;
#[path = "schema.rs"]
mod schema;
#[path = "schema_catalog/mod.rs"]
mod schema_catalog;
#[path = "script_visibility.rs"]
mod script_visibility;
#[path = "type_info.rs"]
mod type_info;
#[path = "type_kind.rs"]
mod type_kind;
#[path = "type_path.rs"]
mod type_path;
#[path = "type_registration.rs"]
mod type_registration;
#[path = "type_role.rs"]
mod type_role;
#[path = "value_budget.rs"]
mod value_budget;
#[path = "value_validation.rs"]
mod value_validation;
#[path = "zr_reflect.rs"]
mod zr_reflect;
#[path = "zr_reflect_value.rs"]
mod zr_reflect_value;

pub use editor_hint::{ReflectEditorHint, ReflectEnumOption};
pub use error::ReflectError;
pub use field_id::ReflectFieldId;
pub use field_id_parse_error::ReflectFieldIdParseError;
pub use field_info::ReflectFieldInfo;
pub use numeric_range::{ReflectNumericRange, ReflectNumericRangeError};
pub use object_address::ReflectObjectAddress;
pub use read_write::{
    ReflectFieldValue, ReflectFieldsRequest, ReflectFieldsResponse, ReflectReadRequest,
    ReflectReadResponse, ReflectWriteRequest, ReflectWriteResponse,
};
pub use reflected_value::ReflectedValue;
pub use schema::{ReflectSchemaFilter, ReflectSchemaRequest, ReflectSchemaResponse};
pub use schema_catalog::{
    ReflectSchemaCatalog, ReflectSchemaCatalogEntry, ReflectSchemaCatalogSnapshot,
    ReflectSchemaFingerprint, REFLECT_SCHEMA_CATALOG_ALGORITHM_VERSION,
};
pub use script_visibility::ReflectScriptVisibility;
pub use type_info::ReflectTypeInfo;
pub use type_kind::ReflectTypeKind;
pub use type_path::{
    ReflectTypePath, MAX_REFLECT_MODULE_PATH_BYTES, MAX_REFLECT_PLUGIN_ID_BYTES,
    MAX_REFLECT_SHORT_TYPE_PATH_BYTES, MAX_REFLECT_TYPE_PATH_BYTES,
};
pub use type_registration::{ReflectSerializationStrategy, ReflectTypeRegistration};
pub use type_role::ReflectTypeRole;
pub use value_budget::{
    ReflectValueBudget, ReflectValueBudgetDimension, ReflectValueFloatKind,
    ReflectValueValidationError,
};
pub use zr_reflect::ZrReflect;
pub use zr_reflect_value::ZrReflectValue;
