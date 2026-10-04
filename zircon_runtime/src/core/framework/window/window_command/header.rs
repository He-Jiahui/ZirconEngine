use std::time::Instant;

use super::WindowCommandId;
use crate::core::framework::window::WindowId;

/// Required authority and deadline fields for every platform-thread window
/// side effect. A desired payload cannot be routed without this header.
/// 中文：HostCommandBroker 在目标快照校验并预留请求容量后生成此头；目标代次防止旧窗口命令串线，
/// request id 贯穿执行与回执，deadline 使过期请求在平台线程执行前转为取消终态。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowCommandHeader {
    target: WindowId,
    request_id: WindowCommandId,
    deadline: Instant,
}

impl WindowCommandHeader {
    pub(crate) const fn new(
        target: WindowId,
        request_id: WindowCommandId,
        deadline: Instant,
    ) -> Self {
        Self {
            target,
            request_id,
            deadline,
        }
    }

    pub const fn target(self) -> WindowId {
        self.target
    }

    pub const fn request_id(self) -> WindowCommandId {
        self.request_id
    }

    pub const fn deadline(self) -> Instant {
        self.deadline
    }
}

/// A generation-qualified desired window change. The concrete desired-state
/// schema stays separate from this transport contract so retired descriptor
/// fields cannot silently re-enter the runtime command path.
/// 中文：broker 仅在 admission 成功后把期望状态与同一头部封装进 FIFO 执行项；平台线程用头部校验目标/期限，
/// 完成时再将同一请求身份放入终态回执。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WindowCommand<Desired> {
    header: WindowCommandHeader,
    desired: Desired,
}

impl<Desired> WindowCommand<Desired> {
    pub(crate) const fn new(header: WindowCommandHeader, desired: Desired) -> Self {
        Self { header, desired }
    }

    pub const fn header(&self) -> WindowCommandHeader {
        self.header
    }

    pub const fn target(&self) -> WindowId {
        self.header.target()
    }

    pub const fn request_id(&self) -> WindowCommandId {
        self.header.request_id()
    }

    pub const fn deadline(&self) -> Instant {
        self.header.deadline()
    }

    pub const fn desired(&self) -> &Desired {
        &self.desired
    }

    pub fn into_desired(self) -> Desired {
        self.desired
    }
}
