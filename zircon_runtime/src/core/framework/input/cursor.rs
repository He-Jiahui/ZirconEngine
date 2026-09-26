use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum CursorGrabMode {
    None,
    Confined,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct CursorPosition {
    pub x: f32,
    pub y: f32,
}

impl CursorPosition {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 运行时发往窗口宿主的反向命令；提交为输入事件后由管理器保留，宿主排空后执行。
/// 帧快照仅展示本帧新增命令，不能代替实际排空。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum CursorHostRequest {
    SetVisible(bool),
    SetGrabMode(CursorGrabMode),
    SetHitTest(bool),
    SetPosition(CursorPosition),
}

impl CursorHostRequest {
    pub const fn set_visible(visible: bool) -> Self {
        Self::SetVisible(visible)
    }

    pub const fn set_grab_mode(grab_mode: CursorGrabMode) -> Self {
        Self::SetGrabMode(grab_mode)
    }

    pub const fn set_hit_test(hit_test: bool) -> Self {
        Self::SetHitTest(hit_test)
    }

    pub const fn set_position(x: f32, y: f32) -> Self {
        Self::SetPosition(CursorPosition::new(x, y))
    }
}
