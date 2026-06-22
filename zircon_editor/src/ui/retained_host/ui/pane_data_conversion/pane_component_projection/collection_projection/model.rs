use crate::ui::retained_host as host_contract;

pub(in super::super) struct ProjectedCollection {
    pub(in super::super) items: Vec<String>,
    pub(in super::super) fields: Vec<host_contract::TemplatePaneCollectionFieldData>,
    pub(in super::super) virtualization_enabled: bool,
    pub(in super::super) virtualization_item_extent: f32,
    pub(in super::super) virtualization_overscan: i32,
    pub(in super::super) virtualization_total_count: i32,
    pub(in super::super) virtualization_visible_start: i32,
    pub(in super::super) virtualization_visible_count: i32,
    pub(in super::super) pagination_page_index: i32,
    pub(in super::super) pagination_page_size: i32,
    pub(in super::super) pagination_page_count: i32,
    pub(in super::super) pagination_total_count: i32,
}
