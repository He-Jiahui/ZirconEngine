//! 静态清单的静态清单读取、字段校验和唯一性上下文回归；经共享读取和本领域断言检查当前包声明，不执行插件行为。
mod known;
mod manifests;
mod scalar_values;
mod strings;
mod uniqueness;
mod workspace;

pub(super) use known::{assert_known_default_packaging_strategies, assert_known_runtime_targets};
pub(super) use manifests::for_each_static_plugin_manifest;
pub(super) use scalar_values::{bool_value, integer_value};
pub(super) use strings::{
    assert_non_empty_string, assert_non_empty_string_array, non_empty_string_array_values,
    non_empty_string_value,
};
pub(super) use uniqueness::{
    assert_unique_dependency_row, assert_unique_static_identity, assert_unique_string_array_entries,
};
pub(super) use workspace::plugins_workspace_root;
