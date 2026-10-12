//! 编辑器场景入口连接模式、世界域选择与视口产品；世界存储和编辑事务各有独立权威，UI 不从此取得任意写入能力。

//! Editor-authored scene state and viewport tooling.

#[path = "modes/mod.rs"]
pub mod modes;
#[path = "selection/mod.rs"]
pub mod selection;
#[path = "viewport/mod.rs"]
pub mod viewport;
