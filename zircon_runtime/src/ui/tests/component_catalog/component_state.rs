//! 这里按语义事件拆分状态归约契约，子模块复用同一注册表与错误类型观察公开调用结果。

use std::collections::BTreeMap;

use crate::ui::component::UiComponentStateRuntimeExt;
use zircon_runtime_interface::ui::component::UiComponentEventError;

use super::*;

mod button;
mod collection_mutation;
mod command_palette;
mod interaction_numeric;
mod keyboard;
mod keyboard_menu;
mod notification_center;
mod overlay;
mod reference_sources;
mod retained_events;
mod selection;
mod slider;
mod table;
mod text_input_validation;
mod toast;
mod tree_view;
mod value_validation;
mod windowing;
