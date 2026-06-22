use std::collections::BTreeMap;

use super::super::attribute_values::value_as_i32;

pub(super) struct ProjectedPagination {
    pub(super) page_index: i32,
    pub(super) page_size: i32,
    pub(super) page_count: i32,
    pub(super) total_count: i32,
}

pub(super) fn projected_pagination(
    attributes: &BTreeMap<String, toml::Value>,
) -> ProjectedPagination {
    ProjectedPagination {
        page_index: attributes
            .get("page_index")
            .and_then(value_as_i32)
            .unwrap_or(0),
        page_size: attributes
            .get("page_size")
            .and_then(value_as_i32)
            .unwrap_or(0),
        page_count: attributes
            .get("page_count")
            .and_then(value_as_i32)
            .unwrap_or(0),
        total_count: attributes
            .get("total_count")
            .and_then(value_as_i32)
            .unwrap_or(0),
    }
}
