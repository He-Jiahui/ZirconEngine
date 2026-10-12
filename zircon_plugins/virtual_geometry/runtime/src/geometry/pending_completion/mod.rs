//! 将可见性反馈或 GPU 完成结果合并到页驻留状态。
#[path = "../../apply_gpu_page_table_entries.rs"]
mod apply_gpu_page_table_entries;
#[path = "../../complete_gpu_uploads_with_slots.rs"]
mod complete_gpu_uploads_with_slots;
#[path = "../complete_pending_pages.rs"]
mod complete_pending_pages;
#[path = "../consume_feedback.rs"]
mod consume_feedback;
#[path = "../evict_preferred_page.rs"]
mod evict_preferred_page;
#[path = "../../ordered_evictable_pages_for.rs"]
mod ordered_evictable_pages_for_target;
#[path = "../slot_is_assignable.rs"]
mod slot_is_assignable;
