//! 这里按语义事件拆分状态归约契约，子模块复用同一注册表与错误类型观察公开调用结果。

use std::collections::BTreeMap;

use crate::ui::component::UiComponentStateRuntimeExt;
use zircon_runtime_interface::ui::component::UiComponentEventError;

use super::*;

#[path = "component_state/button.rs"]
mod button;
#[path = "../component_state/collection_mutation.rs"]
mod collection_mutation;
#[path = "component_state/command_palette.rs"]
mod command_palette;
#[path = "../component_state/interaction_numeric.rs"]
mod interaction_numeric;
#[path = "component_state/keyboard.rs"]
mod keyboard;
#[path = "component_state/keyboard_menu.rs"]
mod keyboard_menu;
#[path = "../component_state/notification_center.rs"]
mod notification_center;
#[path = "component_state/overlay.rs"]
mod overlay;
#[path = "../component_state/reference_sources.rs"]
mod reference_sources;
#[path = "component_state/retained_events.rs"]
mod retained_events;
#[path = "component_state/selection.rs"]
mod selection;
#[path = "component_state/slider.rs"]
mod slider;
#[path = "component_state/table.rs"]
mod table;
#[path = "../component_state/text_input_validation.rs"]
mod text_input_validation;
#[path = "component_state/toast.rs"]
mod toast;
#[path = "component_state/tree_view.rs"]
mod tree_view;
#[path = "../component_state/value_validation.rs"]
mod value_validation;
#[path = "component_state/windowing.rs"]
mod windowing;
