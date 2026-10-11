use crate::core::math::{Quat, Transform, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 相机控制器输出的光标抓取模式描述。
///
/// # 使用时机与边界
///
/// 与 [`CursorGrabIntent`] 一起交给调用方解释；枚举值不会执行窗口操作，也不设置可见性。
/// `Released` 表示不请求抓取，`Confined` 表示限制指针范围，`Locked` 表示锁定指针。
/// 可见性和失焦策略由意图的另外两个字段描述，平台是否支持请求由宿主决定。
///
/// # 生命周期
///
/// 这是可复制的值，不持有窗口、焦点或抓取资源；复制它不会取得或释放宿主资源。
pub enum CursorGrabMode {
    Released,
    Confined,
    Locked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 控制器提出的光标状态请求，由窗口宿主实际执行；focused_only 允许宿主在失焦时撤销抓取。
///
/// # 字段与使用时机
///
/// `mode`、`visible`、`focused_only` 分别描述抓取方式、可见性和仅在焦点有效时维持请求的策略。
/// 三个字段可以独立构造；只有提供的构造方法预设了它们的组合，枚举模式本身不保证可见性。
///
/// # 返回请求的边界
///
/// 控制器返回这个值只代表提出意图，不证明宿主已经应用它。调用方需要另行执行或转交窗口请求，
/// 并处理平台能力和焦点变化；值的复制或丢弃不会自动更改窗口状态。
///
/// # 参考
///
/// 参见 [`Self::released`]、[`Self::confined`]、[`Self::locked`] 与 [`CameraControllerOutput::with_cursor_grab`]。
pub struct CursorGrabIntent {
    pub mode: CursorGrabMode,
    pub visible: bool,
    pub focused_only: bool,
}

impl CursorGrabIntent {
    /// 构造用于撤销抓取的光标意图。
    ///
    /// # 使用时机
    ///
    /// 控制器禁用或结束抓取时可把它附到输出；调用方仍须执行或转交请求。
    ///
    /// # 返回值
    ///
    /// 返回模式为释放、可见且不依赖焦点的值，不直接释放窗口资源。
    ///
    /// # 参考
    ///
    /// 参见 [`CameraControllerOutput::with_cursor_grab`] 与 [`Self::locked`]。
    pub const fn released() -> Self {
        Self {
            mode: CursorGrabMode::Released,
            visible: true,
            focused_only: false,
        }
    }

    /// 构造限制指针范围并隐藏光标的意图。
    ///
    /// # 参数
    ///
    /// - `focused_only`：是否要求宿主只在焦点有效时维持请求；本方法不读取实际窗口焦点。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回 `Confined`、不可见及给定焦点策略的值；宿主是否应用以及何时撤销由调用方协调。
    /// 本方法不取得窗口资源，也不验证平台支持。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::released`]、[`Self::locked`] 与 [`CursorGrabIntent`]。
    pub const fn confined(focused_only: bool) -> Self {
        Self {
            mode: CursorGrabMode::Confined,
            visible: false,
            focused_only,
        }
    }

    /// 构造锁定指针并隐藏光标的意图。
    ///
    /// # 参数
    ///
    /// - `focused_only`：是否要求宿主只在焦点有效时维持请求；本方法不读取实际窗口焦点。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回 `Locked`、不可见及给定焦点策略的值；构造完成不代表宿主已经锁定光标。
    /// 需要结束请求时，由调用方执行释放意图；丢弃本值不释放窗口资源。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::released`]、[`Self::confined`] 与 [`CursorGrabIntent`]。
    pub const fn locked(focused_only: bool) -> Self {
        Self {
            mode: CursorGrabMode::Locked,
            visible: false,
            focused_only,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// 单次输入后的相机结果。changed 只表示变换变化；调用方即使收到 unchanged 也应处理独立的 cursor_grab 请求。
///
/// # 结果的含义
///
/// `transform` 是更新后的完整变换；三个 `*_delta` 是相对更新前快照的差量，不能再叠加到这个完整结果上。
/// 缩放差量是分量差，不是缩放倍率；旋转差量采用更新后旋转左乘更新前旋转逆元的组合顺序。
/// 构造方法按变换比较结果设置 `changed`，不报告轨道目标等控制器内部状态是否改变。
///
/// # 使用时机与生命周期
///
/// 调用方可以应用完整变换或消费差量；光标请求单独处理，不能只在 `changed` 为真时检查它。
/// 输出是可复制快照，不借用控制器，也不证明场景变换或窗口状态已经提交。
/// 公开字段允许自行组装，本类型不验证快照、差量和标志之间的一致性。
///
/// # 参考
///
/// 参见 [`Self::unchanged`]、[`Self::from_transform`] 与 [`Self::with_cursor_grab`]。
pub struct CameraControllerOutput {
    pub transform: Transform,
    pub translation_delta: Vec3,
    pub rotation_delta: Quat,
    pub scale_delta: Vec3,
    pub changed: bool,
    pub cursor_grab: Option<CursorGrabIntent>,
}

impl CameraControllerOutput {
    /// 构造保留给定变换、没有变换差量的输出。
    ///
    /// # 参数
    ///
    /// - `transform`：要保留的完整快照；本方法不校验或修正其中的数值。
    ///
    /// # 返回值
    ///
    /// 返回 `changed = false`、无光标请求的值；旋转差量为单位旋转，平移和缩放差量为零。
    /// 这不表示宿主已应用变换，也不阻止随后附加独立光标意图。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::with_cursor_grab`] 与 [`Self::from_transform`]。
    pub fn unchanged(transform: Transform) -> Self {
        Self {
            transform,
            translation_delta: Vec3::ZERO,
            rotation_delta: Quat::IDENTITY,
            scale_delta: Vec3::ZERO,
            changed: false,
            cursor_grab: None,
        }
    }

    /// 根据更新前后的变换快照构造输出与差量。
    ///
    /// # 参数
    ///
    /// - `before`：用于比较并计算差量的更新前快照。
    /// - `after`：作为完整输出保存的更新后快照。
    ///
    /// # 返回值与约束
    ///
    /// 采用 `before == after` 的比较结果决定是否变化，没有额外的容差判断；相等时返回无差量输出。
    /// 否则计算分量平移差、分量缩放差和 `after.rotation * before.rotation.inverse()`，光标请求初始为空。
    /// 本方法不验证数值，也不归一化旋转；调用方须满足所用变换和四元数运算的约定。
    /// 返回快照不修改场景或控制器状态。
    ///
    /// # 参考
    ///
    /// 参见 [`Self::unchanged`] 与 [`Self::with_cursor_grab`]。
    pub fn from_transform(before: Transform, after: Transform) -> Self {
        if before == after {
            return Self::unchanged(after);
        }
        Self {
            transform: after,
            translation_delta: after.translation - before.translation,
            rotation_delta: after.rotation * before.rotation.inverse(),
            scale_delta: after.scale - before.scale,
            changed: before != after,
            cursor_grab: None,
        }
    }

    /// 在变换输出中附加或替换光标意图。
    ///
    /// # 参数
    ///
    /// - `cursor_grab`：调用方需要另行执行或转交的请求；已有请求会被这个值替换。
    ///
    /// # 返回值与生命周期
    ///
    /// 返回修改后的输出，完整变换、差量和 `changed` 保持原值；请求不会在此排队或执行。
    /// 因此静止输出也可以携带新的窗口意图。
    ///
    /// # 参考
    ///
    /// 参见 [`CursorGrabIntent`] 与 [`Self::unchanged`]。
    pub fn with_cursor_grab(mut self, cursor_grab: CursorGrabIntent) -> Self {
        self.cursor_grab = Some(cursor_grab);
        self
    }
}
