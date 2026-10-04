//! 这组测试从树可见性进入布局、焦点、命中和事件路由，确认各消费者使用同一有效可见性契约。

use super::*;

mod collapsed_layout;
mod focus_candidates;
mod hit_visibility;
mod pointer_routes;
