//! Runtime UI module registration absorbed into the runtime layer.

mod module;
mod runtime_ui;

pub use module::{
    module_descriptor, UiConfig, UiModule, UiRuntimeDriver, UI_EVENT_MANAGER_NAME, UI_MODULE_NAME,
    UI_RUNTIME_DRIVER_NAME,
};
pub use runtime_ui::{RuntimeUiFixture, RuntimeUiManager, RuntimeUiManagerError};
pub use zircon_ui::binding;
pub use zircon_ui::dispatch;
pub use zircon_ui::event_ui;
pub use zircon_ui::layout;
pub use zircon_ui::surface;
pub use zircon_ui::template;
pub use zircon_ui::tree;
pub use zircon_ui::{
    Anchor, AxisConstraint, BoxConstraints, DesiredSize, LayoutBoundary, Pivot, Position,
    ResolvedAxisConstraint, StretchMode, UiAssetDocument, UiAssetKind, UiAxis, UiContainerKind,
    UiFocusState, UiFrame, UiInputPolicy, UiLinearBoxConfig, UiNavigationEventKind,
    UiNavigationRoute, UiNavigationState, UiPoint, UiPointerButton, UiPointerEventKind,
    UiPointerRoute, UiScrollState, UiScrollableBoxConfig, UiScrollbarVisibility, UiSize,
    UiSurface, UiVirtualListConfig, UiVirtualListWindow,
};
