use serde::{Deserialize, Serialize};

/// Groups Runtime UI components by editor-host rendering and interaction role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiComponentCategory {
    /// Static or low-interaction visual primitives such as text, images, and separators.
    Visual,
    /// Direct input controls such as buttons, toggles, text fields, and context menus.
    Input,
    /// Numeric and vector controls that edit scalar, color, or vector values.
    Numeric,
    /// Option-driven controls such as dropdowns, enum fields, flags fields, and search select.
    Selection,
    /// Reference controls that accept asset, scene-instance, or object drag payloads.
    Reference,
    /// Data-structure controls that expose array, map, list, or tree rows.
    Collection,
    /// Layout or inspector grouping controls that own child content slots.
    Container,
    /// Status and feedback controls such as progress, spinner, badge, and help rows.
    Feedback,
}
