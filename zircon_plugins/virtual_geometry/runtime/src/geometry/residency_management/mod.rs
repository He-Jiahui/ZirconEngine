//! 集中处理页晋升、淘汰和槽位复用，维持驻留映射与待处理请求一致。
#[path = "../clear_pending_request.rs"]
mod clear_pending_request;
#[path = "../evict_page.rs"]
mod evict_page;
#[path = "../page_in_slot.rs"]
mod page_in_slot;
#[path = "../promote_to_resident.rs"]
mod promote_to_resident;
#[path = "../../promote_to_resident_in_slot.rs"]
mod promote_to_resident_in_slot;
#[path = "../reserve_slot.rs"]
mod reserve_slot;
#[path = "../take_free_slot.rs"]
mod take_free_slot;
#[path = "../take_next_slot.rs"]
mod take_next_slot;
