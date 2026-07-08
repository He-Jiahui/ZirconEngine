#[path = "inventory/behavior_anchors.rs"]
mod behavior_anchors;
#[path = "inventory/cursor_host_requests.rs"]
mod cursor_host_requests;
#[path = "inventory/guard_anchors.rs"]
mod guard_anchors;
#[path = "inventory/mirror_docs.rs"]
mod mirror_docs;
#[path = "inventory/module_sets.rs"]
mod module_sets;
#[path = "inventory/public_surface.rs"]
mod public_surface;
#[path = "inventory/split_layout.rs"]
mod split_layout;

const INVENTORY_SLICE: &str = "Runtime 15 M3 input-stack inventory guard folder-backed split";
const INVENTORY_STATUS: &str =
    "runtime_15_input_stack_inventory_guard_folder_backed_static_passed_cargo_deferred";
const INVENTORY_FRAMEWORKS_STATUS: &str =
    "frameworks_02_m3_input_stack_inventory_guard_folder_backed_static_passed_cargo_deferred";
const INVENTORY_GUARD: &str = "runtime_15_input_stack_inventory_guard_is_folder_backed";
const INVENTORY_PARENT_PATH: &str = "input_stack/inventory.rs";
const INVENTORY_CHILD_PATHS: &[&str] = &[
    "input_stack/inventory/module_sets.rs",
    "input_stack/inventory/public_surface.rs",
    "input_stack/inventory/guard_anchors.rs",
    "input_stack/inventory/behavior_anchors.rs",
    "input_stack/inventory/cursor_host_requests.rs",
    "input_stack/inventory/mirror_docs.rs",
    "input_stack/inventory/split_layout.rs",
];
