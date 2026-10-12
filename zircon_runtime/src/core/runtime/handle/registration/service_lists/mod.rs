#[path = "multi.rs"]
mod multi;
#[path = "selection.rs"]
mod selection;
#[path = "shutdown.rs"]
mod shutdown;
#[path = "specialized.rs"]
mod specialized;
#[path = "types.rs"]
mod types;

pub(super) use self::selection::module_service_lists;
pub(super) use self::specialized::single_service_module_lists;
pub(super) use self::types::ModuleServiceLists;
