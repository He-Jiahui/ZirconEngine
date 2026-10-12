//! 将持久页状态投影为本帧可见簇、上传请求和槽位候选。
#[path = "../available_slots.rs"]
mod available_slots;
#[path = "../build_prepare_frame.rs"]
mod build_prepare_frame;
#[path = "../evictable_pages.rs"]
mod evictable_pages;
#[path = "../pending_page_requests.rs"]
mod pending_page_requests;
#[path = "../prepare_visible_clusters.rs"]
mod prepare_visible_clusters;
#[path = "../../prepared_visible_clusters.rs"]
mod prepared_visible_clusters;
#[path = "../resident_pages.rs"]
mod resident_pages;
