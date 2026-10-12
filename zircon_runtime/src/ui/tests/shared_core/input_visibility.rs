//! 这组测试从树可见性进入布局、焦点、命中和事件路由，确认各消费者使用同一有效可见性契约。

use super::*;

#[path = "input_visibility/collapsed_layout.rs"]
mod collapsed_layout;
#[path = "input_visibility/focus_candidates.rs"]
mod focus_candidates;
#[path = "input_visibility/hit_visibility.rs"]
mod hit_visibility;
#[path = "input_visibility/pointer_routes.rs"]
mod pointer_routes;
