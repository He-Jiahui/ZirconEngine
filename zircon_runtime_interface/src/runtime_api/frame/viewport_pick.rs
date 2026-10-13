//! 绑定视口、已展示帧和输入序号的异步像素拾取 ABI。
//!
//! Editor 以请求身份申请票据，再轮询完成值；Runtime 会话持有票据状态，并将请求转换为渲染
//! 后端的同帧拾取。请求与回执重复携带视口尺寸、像素、用途、帧代数、输入序号和策略，供两侧
//! 在消费异步结果前校验其仍对应原始输入。
//!
//! 这里的值类型不持有渲染资源或会话；票据的实际存储和 GPU 读回由 Runtime 按会话管理。ABI
//! 入口见 Runtime 动态导出与 Editor `SessionGateway`，帧身份由 Runtime 视口拾取路径解析。

use crate::{ZrRuntimeViewportHandle, ZrRuntimeViewportSizeV1, ZIRCON_RUNTIME_ABI_VERSION_V1};

/// 在拾取策略中包含半透明绘制对象的标志位。
pub const ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1: u32 = 1 << 0;
/// 在拾取策略中包含背面几何的标志位。
pub const ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1: u32 = 1 << 1;
// 新增策略位必须同时加入验证掩码，并由 Runtime 渲染策略消费者实现。
const ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1: u32 =
    ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_TRANSLUCENT_V1
        | ZR_RUNTIME_VIEWPORT_PICK_POLICY_INCLUDE_BACKFACES_V1;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
/// Runtime 会话内异步视口拾取的不可解释票据；只有非零原始值有效。
///
/// # 使用时机与生命周期
///
/// Runtime 成功受理请求后在对应会话中登记票据，调用方将它用于同一会话的轮询或取消；登记状态
/// 由 `RuntimeViewportPickStore` 持有，本类型不拥有或延长该状态。另一个非零值或已消费票据不会
/// 因 `is_valid` 返回 `true` 而成为可轮询票据。
pub struct ZrRuntimeViewportPickTicket(u64);

impl ZrRuntimeViewportPickTicket {
    /// 从 ABI 原始值构造票据；`0` 仍表示无效票据。
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// 构造零值无效票据，供输出初始化和未成功请求路径使用。
    pub const fn invalid() -> Self {
        Self(0)
    }

    /// 读取供 ABI 传递或诊断使用的原始票据值。
    pub const fn raw(self) -> u64 {
        self.0
    }

    /// 判断票据原始值是否非零。
    ///
    /// 此值级检查不查询会话、登记状态或消费状态；轮询和取消入口仍须查找对应会话中的票据记录。
    pub const fn is_valid(self) -> bool {
        self.0 != 0
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 拾取的输入语义；原始 ABI 值由显式转换方法读写。
pub enum ZrRuntimeViewportPickPurposeV1 {
    /// 指针悬停查询。
    Hover = 1,
    /// 按下输入查询。
    Press = 2,
    /// 显式选择操作查询。
    Selection = 3,
}

impl ZrRuntimeViewportPickPurposeV1 {
    /// 返回用于固定 ABI 编码的整数标签。
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// 将已知 ABI 标签转换为用途；未知值不会构造本地枚举。
    ///
    /// # 参数
    ///
    /// `value` 是跨 ABI 读取的原始用途标签。
    ///
    /// # 返回值
    ///
    /// 标签 `1`、`2`、`3` 分别返回 `Hover`、`Press`、`Selection`；其余值返回 `None`。
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Hover),
            2 => Some(Self::Press),
            3 => Some(Self::Selection),
            _ => None,
        }
    }
}

#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 视口拾取票据当前可观察的进度或终态。
pub enum ZrRuntimeViewportPickDispositionV1 {
    /// 后端操作尚未完成；该票据仍可继续轮询。
    Pending = 1,
    /// 操作完成且指定像素未命中目标。
    NoHit = 2,
    /// 操作完成并包含一个有效命中目标。
    Hit = 3,
    /// 请求所指帧已不可用，不能改用其他帧完成拾取。
    StaleFrame = 4,
    /// 当前运行时或渲染器不提供此拾取能力。
    Unavailable = 5,
    /// 请求或后端完成值未通过验证。
    Rejected = 6,
    /// 拾取操作已被取消。
    Cancelled = 7,
}

impl ZrRuntimeViewportPickDispositionV1 {
    /// 返回用于固定 ABI 编码的整数标签。
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// 将已知 ABI 标签转换为进度/终态值；未知值返回 `None`。
    ///
    /// # 参数
    ///
    /// `value` 是跨 ABI 读取的原始状态标签。
    ///
    /// # 返回值
    ///
    /// 标签 `1` 至 `7` 对应本枚举的七种值；其他数值不会被解释为有效状态。
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Pending),
            2 => Some(Self::NoHit),
            3 => Some(Self::Hit),
            4 => Some(Self::StaleFrame),
            5 => Some(Self::Unavailable),
            6 => Some(Self::Rejected),
            7 => Some(Self::Cancelled),
            _ => None,
        }
    }

    /// 判断是否已完成，不再代表等待中的票据。
    ///
    /// 只有 `Pending` 返回 `false`；所有其他已知状态都为终态。
    pub const fn is_terminal(self) -> bool {
        !matches!(self, Self::Pending)
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 视口像素网格中的整数坐标；请求验证要求坐标落在其宽高范围内。
pub struct ZrRuntimeViewportPixelV1 {
    /// 从视口左边界开始的水平像素索引。
    pub x: u32,
    /// 从视口上边界开始的垂直像素索引。
    pub y: u32,
}

impl ZrRuntimeViewportPixelV1 {
    /// 保存给定的视口像素坐标；范围由携带该坐标的请求验证。
    pub const fn new(x: u32, y: u32) -> Self {
        Self { x, y }
    }
}

/// Allocation-free request for one renderer-owned viewport hit.
///
/// `frame_generation` identifies the already-presented render product. A backend must reject or
/// return `StaleFrame` instead of silently evaluating the pixel against a newer frame.
///
/// # 使用时机与生命周期
///
/// Editor 从当前展示帧创建请求并通过会话 Gateway 申请票据；Runtime 用视口和帧代数解析该帧的
/// 拾取数据，不能将已过期的像素请求改用更新帧执行。
///
/// # 约束
///
/// 像素须在视口宽高内，帧代数与输入序号须非零，策略位必须已知，保留字段必须为零。构造器
/// 填入 ABI 版本、用途标签和零保留位，但不替代完整验证。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeViewportPickRequestV1 {
    /// ABI 版本；构造器填入 V1 常量。
    pub abi_version: u32,
    /// 由 `ZrRuntimeViewportPickPurposeV1` 编码的输入用途。
    pub purpose: u32,
    /// 发起拾取的目标视口句柄。
    pub viewport: ZrRuntimeViewportHandle,
    /// 该像素坐标所属的视口宽高。
    pub viewport_size: ZrRuntimeViewportSizeV1,
    /// 请求查询的视口像素。
    pub pixel: ZrRuntimeViewportPixelV1,
    /// 请求绑定的已展示帧代数。
    pub frame_generation: u64,
    /// 产生此请求的输入序号，须非零并由回执原样带回。
    pub input_sequence: u64,
    /// 已知策略标志的位掩码。
    pub policy_flags: u32,
    /// 为未来 ABI 扩展保留；V1 请求必须为零。
    pub reserved: u32,
}

impl ZrRuntimeViewportPickRequestV1 {
    /// 从视口、帧身份和输入意图构造 V1 请求。
    ///
    /// # 参数
    ///
    /// `viewport`、`viewport_size` 与 `pixel` 描述目标像素；`frame_generation` 锁定已展示帧，
    /// `input_sequence` 标记输入序号，`purpose` 指定用途，`policy_flags` 选择拾取策略。
    ///
    /// # 返回值
    ///
    /// 返回填入 V1 版本、用途标签和零保留位的请求；尺寸、像素和策略仍需单独验证。
    pub const fn new(
        viewport: ZrRuntimeViewportHandle,
        viewport_size: ZrRuntimeViewportSizeV1,
        pixel: ZrRuntimeViewportPixelV1,
        frame_generation: u64,
        input_sequence: u64,
        purpose: ZrRuntimeViewportPickPurposeV1,
        policy_flags: u32,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            purpose: purpose.raw(),
            viewport,
            viewport_size,
            pixel,
            frame_generation,
            input_sequence,
            policy_flags,
            reserved: 0,
        }
    }

    /// 解码请求中的用途标签；未知整数返回 `None`。
    pub const fn purpose(self) -> Option<ZrRuntimeViewportPickPurposeV1> {
        ZrRuntimeViewportPickPurposeV1::from_raw(self.purpose)
    }

    /// 验证 ABI 版本、视口/像素范围、帧与输入身份、策略掩码和保留字段。
    ///
    /// Runtime FFI 和 Editor Gateway 在跨边界前调用此检查。它检查请求结构，但不证明对应帧仍
    /// 存在；Runtime 后续的拾取帧解析负责判定是否返回 `StaleFrame`。
    pub const fn validate_viewport_pick(self) -> bool {
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.purpose().is_some()
            && self.viewport.is_valid()
            && self.viewport_size.width > 0
            && self.viewport_size.height > 0
            && self.pixel.x < self.viewport_size.width
            && self.pixel.y < self.viewport_size.height
            && self.frame_generation != 0
            && self.input_sequence != 0
            && self.policy_flags & !ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1 == 0
            && self.reserved == 0
    }
}

/// Fixed-layout completion for one viewport pick ticket.
///
/// The owning gateway supplies the runtime-session identity. This payload repeats every remaining
/// identity needed to reject a completion from another viewport, input edge, frame, or world.
///
/// Runtime FFI 在写出前验证状态与命中数据组合；Editor Gateway 先核对票据，固定路由再用
/// `matches_request` 对照原请求身份。`Hit` 携带目标与几何值；其他状态必须清零目标和几何字段，
/// 但可保留查询时的世界代数。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeViewportPickResultV1 {
    /// ABI 版本；有效回执为 V1。
    pub abi_version: u32,
    /// 由 `ZrRuntimeViewportPickDispositionV1` 编码的进度或终态。
    pub disposition: u32,
    /// 产生此回执的会话内票据。
    pub ticket: ZrRuntimeViewportPickTicket,
    /// 原请求中的视口句柄。
    pub viewport: ZrRuntimeViewportHandle,
    /// 原请求中的视口宽高。
    pub viewport_size: ZrRuntimeViewportSizeV1,
    /// 原请求查询的像素。
    pub pixel: ZrRuntimeViewportPixelV1,
    /// 原请求的用途 ABI 标签。
    pub purpose: u32,
    /// 原请求绑定的已展示帧代数。
    pub frame_generation: u64,
    /// 原请求携带的输入序号。
    pub input_sequence: u64,
    /// 完成拾取所用帧对应的 Runtime 世界代数；非命中回执可为零。
    pub world_generation: u64,
    /// 命中实体编号；非命中状态必须为零。
    pub entity: u64,
    /// 命中实例编号；非命中状态必须为零。
    pub instance: u64,
    /// 命中子对象编号；非命中状态必须为零。
    pub subobject: u64,
    /// 命中深度；命中时须有限且非负，非命中时必须为零。
    pub depth: f32,
    /// 命中位置的世界坐标；所有分量必须有限，非命中时必须全零。
    pub world_position: [f32; 3],
    /// 命中法线的世界坐标；所有分量必须有限，非命中时必须全零。
    pub world_normal: [f32; 3],
    /// Runtime 实际应用的拾取策略位，须为已知标志。
    pub applied_policy_flags: u32,
    /// 为未来 ABI 扩展保留；有效回执必须为零。
    pub reserved: u32,
}

impl ZrRuntimeViewportPickResultV1 {
    /// 构造全零无效哨兵，供 FFI 在写入真实结果前初始化输出。
    ///
    /// 返回值故意不通过 `validate_viewport_pick`，可用于失败时避免调用方误读未写输出。
    pub const fn invalid() -> Self {
        Self {
            abi_version: 0,
            disposition: 0,
            ticket: ZrRuntimeViewportPickTicket::invalid(),
            viewport: ZrRuntimeViewportHandle::invalid(),
            viewport_size: ZrRuntimeViewportSizeV1::new(0, 0),
            pixel: ZrRuntimeViewportPixelV1::new(0, 0),
            purpose: 0,
            frame_generation: 0,
            input_sequence: 0,
            world_generation: 0,
            entity: 0,
            instance: 0,
            subobject: 0,
            depth: 0.0,
            world_position: [0.0; 3],
            world_normal: [0.0; 3],
            applied_policy_flags: 0,
            reserved: 0,
        }
    }

    /// 构造不含命中几何的回执，并从请求复制关联身份和策略位。
    ///
    /// # 参数
    ///
    /// `disposition` 应是非 `Hit` 状态；`ticket` 和 `request` 标识会话内请求，
    /// `world_generation` 是完成路径提供的世界代数，可对非命中状态使用零值。
    ///
    /// # 返回值
    ///
    /// 返回目标编号、深度、位置和法线均清零的回执。本构造器不验证参数组合；调用者须在输出前
    /// 使用 `validate_viewport_pick` 检查。
    pub const fn empty(
        disposition: ZrRuntimeViewportPickDispositionV1,
        ticket: ZrRuntimeViewportPickTicket,
        request: ZrRuntimeViewportPickRequestV1,
        world_generation: u64,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            disposition: disposition.raw(),
            ticket,
            viewport: request.viewport,
            viewport_size: request.viewport_size,
            pixel: request.pixel,
            purpose: request.purpose,
            frame_generation: request.frame_generation,
            input_sequence: request.input_sequence,
            world_generation,
            entity: 0,
            instance: 0,
            subobject: 0,
            depth: 0.0,
            world_position: [0.0; 3],
            world_normal: [0.0; 3],
            applied_policy_flags: request.policy_flags,
            reserved: 0,
        }
    }

    // EXEMPT(GEN-Q7): viewport-pick receipt construction mirrors the fixed runtime ABI fields.
    #[allow(clippy::too_many_arguments)]
    /// 构造包含命中身份与几何值的 `Hit` 回执。
    ///
    /// # 参数
    ///
    /// `ticket` 与 `request` 关联会话请求；`world_generation`、实体/实例/子对象编号和深度、
    /// 世界位置、世界法线来自该请求绑定帧的拾取完成值。
    ///
    /// # 返回值
    ///
    /// 返回复制请求身份并设置命中载荷的 V1 回执。构造器不验证这些参数；有效回执须满足
    /// `validate_viewport_pick` 的命中条件。
    pub const fn hit(
        ticket: ZrRuntimeViewportPickTicket,
        request: ZrRuntimeViewportPickRequestV1,
        world_generation: u64,
        entity: u64,
        instance: u64,
        subobject: u64,
        depth: f32,
        world_position: [f32; 3],
        world_normal: [f32; 3],
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            disposition: ZrRuntimeViewportPickDispositionV1::Hit.raw(),
            ticket,
            viewport: request.viewport,
            viewport_size: request.viewport_size,
            pixel: request.pixel,
            purpose: request.purpose,
            frame_generation: request.frame_generation,
            input_sequence: request.input_sequence,
            world_generation,
            entity,
            instance,
            subobject,
            depth,
            world_position,
            world_normal,
            applied_policy_flags: request.policy_flags,
            reserved: 0,
        }
    }

    /// 解码回执状态标签；未知整数返回 `None`。
    pub const fn disposition(self) -> Option<ZrRuntimeViewportPickDispositionV1> {
        ZrRuntimeViewportPickDispositionV1::from_raw(self.disposition)
    }

    /// FFI 出口与会话轮询用它检查 disposition、身份字段及命中数据组合是否合法；请求关联另由 `matches_request` 检查。
    ///
    /// 命中值要求非零世界代数、非零实体和有限非负深度，但不约束深度上限或法线长度；所有状态的
    /// 几何值都必须有限，非命中状态的目标和几何值必须为零。此方法不验证回执是否对应某一请求。
    pub fn validate_viewport_pick(self) -> bool {
        let Some(disposition) = self.disposition() else {
            return false;
        };
        let identity_is_valid = self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.ticket.is_valid()
            && self.viewport.is_valid()
            && self.viewport_size.width > 0
            && self.viewport_size.height > 0
            && self.pixel.x < self.viewport_size.width
            && self.pixel.y < self.viewport_size.height
            && ZrRuntimeViewportPickPurposeV1::from_raw(self.purpose).is_some()
            && self.frame_generation != 0
            && self.input_sequence != 0
            && self.applied_policy_flags & !ZR_RUNTIME_VIEWPORT_PICK_POLICY_KNOWN_FLAGS_V1 == 0
            && self.reserved == 0;
        if !identity_is_valid {
            return false;
        }

        let geometry_is_finite = self.depth.is_finite()
            && self.world_position.iter().all(|value| value.is_finite())
            && self.world_normal.iter().all(|value| value.is_finite());
        if !geometry_is_finite {
            return false;
        }

        if disposition == ZrRuntimeViewportPickDispositionV1::Hit {
            self.world_generation != 0 && self.entity != 0 && self.depth >= 0.0
        } else {
            self.entity == 0
                && self.instance == 0
                && self.subobject == 0
                && self.depth == 0.0
                && self.world_position == [0.0; 3]
                && self.world_normal == [0.0; 3]
        }
    }

    /// 编辑器消费异步 ticket 前逐项核对原请求身份，拒绝迟到帧或其他输入边沿的结果作用于当前视口。
    ///
    /// 方法先验证请求和回执，再比较视口、尺寸、像素、用途、帧代数、输入序号和应用策略；它不比较
    /// `ticket`、`world_generation` 或会话身份。调用者须按自身会话和世界策略另行筛选；Editor 固定
    /// `EditorRuntimeViewportPickRoute` 经捕获的会话 origin 轮询，并独立比较 ticket 和原请求，但不
    /// 增加 `world_generation` 比较。
    pub fn matches_request(self, request: ZrRuntimeViewportPickRequestV1) -> bool {
        request.validate_viewport_pick()
            && self.validate_viewport_pick()
            && self.viewport == request.viewport
            && self.viewport_size == request.viewport_size
            && self.pixel == request.pixel
            && self.purpose == request.purpose
            && self.frame_generation == request.frame_generation
            && self.input_sequence == request.input_sequence
            && self.applied_policy_flags == request.policy_flags
    }
}

#[cfg(test)]
#[path = "tests/viewport_pick.rs"]
mod tests;
