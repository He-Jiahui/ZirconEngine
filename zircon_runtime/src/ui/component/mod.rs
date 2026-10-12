#[path = "catalog/mod.rs"]
mod catalog;
#[path = "data_binding/mod.rs"]
mod data_binding;
#[path = "descriptor/mod.rs"]
mod descriptor;
#[path = "state_reducer.rs"]
mod state_reducer;

pub use catalog::{UiComponentDescriptorRegistry, UiComponentPaletteEntry};
pub use data_binding::inspector_selected_entity_data_source;
pub use descriptor::{validate_component_descriptor, UiComponentDescriptorError};
pub use state_reducer::{
    apply_component_event, CatalogNavigation, CommandCatalog, TableIndex, TableSortResult,
    TreeIndex, TreeRowDelta, UiComponentStateChange, UiComponentStateModel, UiComponentStatePatch,
    UiComponentStateRuntimeExt,
};
