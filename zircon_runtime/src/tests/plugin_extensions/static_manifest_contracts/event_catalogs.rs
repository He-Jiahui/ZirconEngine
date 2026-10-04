//! 静态清单的事件目录、载荷模式与版本身份回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
use super::{for_each_static_plugin_manifest, integer_value, non_empty_string_value};

mod assertions;
mod catalog_ids;
mod payload_schemas;
mod rows;
mod traversal;

pub(super) use assertions::{
    assert_dot_namespaced_event_id, assert_event_rows, assert_versioned_payload_schema,
};
pub(super) use traversal::{event_catalog_array, visit_event_catalogs, visit_event_rows};
