//! Versioned AppSession V2 configuration negotiated before native IME events.
//!
//! The descriptor owns the public candidate rectangle topology.  It is passed through a
//! standalone symbol so the frozen V8 function table and the V3 session creation ABI remain
//! unchanged.  Every field is bounded and validated before a runtime session is touched.
//! # 职责与入口
//!
//! 本模块定义宿主提交给已有 AppSession 的 IME 契约和窗口候选矩形。
//! 宿主在发送原生 V2 事件前，通过独立的配置符号提交描述符；匹配 V8 函数表本身不代表支持该操作。
//!
//! # 构造、验证与生命周期
//!
//! 描述符和矩形按值携带数据，不持有窗口句柄或借用缓冲区。
//! 构造器只填充字段；本地验证只检查描述符内容，安装到哪个仍存活的会话由配置入口核对。
//! 宿主负责提供当前窗口的非零生命周期代数，并完成屏幕到窗口坐标的转换。
//!
//! # 参考
//!
//! 参见 [`ZrRuntimeAppSessionConfigurationV2::validate`]、
//! [`ZrRuntimeAppSessionConfigurationV2::negotiation`] 和 [`ZR_RUNTIME_CONFIGURE_APP_SESSION_SYMBOL_V2`]。

use core::mem::size_of;

use super::ime_composition_capability_v2::ZrRuntimeImeCompositionNegotiation;
use super::ime_composition_capability_v2::ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION;

/// 独立 AppSession 配置描述符使用的 ABI 版本号。
pub const ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2: u32 = 2;
/// 描述符中的组合 V2 能力位；提交时还须同时声明候选矩形能力。
pub const ZR_RUNTIME_APP_SESSION_CAPABILITY_IME_COMPOSITION_V2: u64 = 1 << 0;
/// 描述符中的候选矩形能力位；与组合 V2 能力共同构成必需集合。
pub const ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2: u64 = 1 << 1;
/// 本实现接受的组合载荷 schema 编号。
pub const ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2: u16 = 2;
/// 本实现接受的候选矩形 schema 编号。
pub const ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2: u16 = 1;
/// 候选矩形以窗口内像素坐标表达时使用的空间标签。
pub const ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1: u32 = 1;
/// 本地校验允许的原生有符号坐标最大值，边缘也必须不超过此值。
pub const ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2: i64 = i32::MAX as i64;
/// 独立导出配置符号的 NUL 结尾名称，不属于冻结的 V8 函数表。
pub const ZR_RUNTIME_CONFIGURE_APP_SESSION_SYMBOL_V2: &[u8] =
    b"zircon_runtime_configure_app_session_v2\0";

const REQUIRED_CAPABILITIES: u64 = ZR_RUNTIME_APP_SESSION_CAPABILITY_IME_COMPOSITION_V2
    | ZR_RUNTIME_APP_SESSION_CAPABILITY_CANDIDATE_RECT_V2;

/// 为原生 IME 候选窗口描述当前文本或插入点的窗口内像素矩形。
///
/// # 使用时机与约束
///
/// 宿主将矩形放入 [`ZrRuntimeAppSessionConfigurationV2`] 后提交给所属会话。
/// `origin_x`、`origin_y` 是有符号窗口坐标；宽度可为零，高度在提交时必须大于零。
/// 宽、高及右边、下边必须能由原生有符号 32 位坐标表示；当前验证不要求原点非负或矩形位于窗口可视区。
/// `coordinate_space` 只接受窗口坐标标签，屏幕坐标转换由宿主完成；`reserved` 必须为零。
///
/// # 构造与生命周期
///
/// 该结构按值复制，不使窗口存活，也不证明矩形仍对应当前插入点。
/// [`Self::window_relative`] 只组装字段，提交前由 [`ZrRuntimeAppSessionConfigurationV2::validate`] 校验。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeImeCandidateRectV2 {
    /// Window-relative left edge of the text bounds in pixels.
    /// 候选矩形左边缘的窗口内有符号像素坐标。
    pub origin_x: i32,
    /// Window-relative top edge of the text bounds in pixels.
    /// 候选矩形上边缘的窗口内有符号像素坐标。
    pub origin_y: i32,
    /// Text-bound width.  Windows IMM uses a zero-width exclusion area at the caret.
    /// 矩形像素宽度；零宽可表示插入点锚定区域。
    pub extent_width: u32,
    /// Text-bound height; must be nonzero for a candidate popup anchor.
    /// 矩形像素高度；完整配置验证要求该值非零。
    pub extent_height: u32,
    /// Explicit coordinate-space tag; screen to window conversion is a host responsibility.
    /// 必须使用窗口坐标标签；屏幕到窗口的转换由宿主负责。
    pub coordinate_space: u32,
    /// 为 ABI 扩展保留，当前提交时必须为零。
    pub reserved: u32,
}

impl ZrRuntimeImeCandidateRectV2 {
    /// 生成窗口坐标矩形；构造不检查边界及非零高度，提交前须由所属配置的 validate 检查。
    /// # 参数
    ///
    /// - `origin_x`、`origin_y`：窗口内的有符号像素坐标，按原值保留。
    /// - `width`：像素宽度，可为零以描述插入点。
    /// - `height`：像素高度；构造时可为零，但配置验证会拒绝零高度。
    ///
    /// # 返回与使用时机
    ///
    /// 返回设置了窗口坐标标签且保留字段清零的矩形；本函数无错误返回，也不截断或校验尺寸。
    /// 宿主完成坐标转换后可调用本函数，随后通过 [`ZrRuntimeAppSessionConfigurationV2::validate`] 检查原生边界。
    pub const fn window_relative(origin_x: i32, origin_y: i32, width: u32, height: u32) -> Self {
        Self {
            origin_x,
            origin_y,
            extent_width: width,
            extent_height: height,
            coordinate_space: ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1,
            reserved: 0,
        }
    }

    /// 检查候选矩形的协议标签和原生坐标可表示性，供所属配置验证使用。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 `Ok(())`，只证明当前字段满足本地矩形契约。
    /// 坐标空间标签不匹配返回 `UnsupportedCoordinateSpace`；零高度返回 `InvalidCandidateRect`。
    /// 宽、高或右边、下边超出 `i32::MAX` 返回 `UnrepresentableCandidateRect`；保留字段非零返回 `NonZeroReserved`。
    /// 计算边界时先提升到 `i64`，因此校验本身不会用有符号 32 位加法溢出。
    ///
    /// # 职责边界与参考
    ///
    /// 本函数不改变矩形，也不检查窗口存活、当前插入点或矩形是否落在可视区。
    /// 参见 [`ZrRuntimeAppSessionConfigurationV2::validate`] 和 [`ZrRuntimeAppSessionConfigurationError`]。
    fn validate(&self) -> Result<(), ZrRuntimeAppSessionConfigurationError> {
        if self.coordinate_space != ZR_RUNTIME_APP_SESSION_COORDINATE_SPACE_WINDOW_V1 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCoordinateSpace);
        }
        if self.extent_height == 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::InvalidCandidateRect);
        }
        if self.extent_width > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2 as u32
            || self.extent_height > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2 as u32
            || i64::from(self.origin_x) + i64::from(self.extent_width)
                > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2
            || i64::from(self.origin_y) + i64::from(self.extent_height)
                > ZR_RUNTIME_APP_SESSION_MAX_NATIVE_COORDINATE_V2
        {
            return Err(ZrRuntimeAppSessionConfigurationError::UnrepresentableCandidateRect);
        }
        if self.reserved != 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::NonZeroReserved);
        }
        Ok(())
    }
}

/// 绑定宿主窗口与候选矩形的 V2 描述符，经独立符号提交，使用前须校验版本、大小和 schema。
/// 两项 IME 能力必须同时声明，保留字清零；window_generation 来自宿主窗口生命周期。
/// # 使用时机与字段关系
///
/// 宿主为已创建的会话安装原生 IME V2 支持时提交该描述符，并在对应的窗口生命周期内提供候选矩形。
/// `abi_version` 与 `size_bytes` 标识描述符本身；`peer_api_version`、两种 schema 和能力位描述宿主承诺的契约。
/// 当前能力位必须恰好包含组合 V2 与候选矩形 V2 两项，所有保留字段清零。
/// `candidate_rect` 使用窗口内像素坐标，`window_generation` 标识该矩形及后续组合事件所属窗口生命周期。
///
/// # 所有权、构造与验证
///
/// 结构及矩形都按值传递，不借用宿主内存，也不持有使窗口或 Runtime 会话继续存活的资源。
/// [`Self::ime_composition`] 生成待验证描述符；公开字段可被修改，所以每次提交前仍需 [`Self::validate`]。
/// [`Self::negotiation`] 返回经本地检查的协商值，实际会话安装仍须成功调用独立配置入口。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAppSessionConfigurationV2 {
    /// 描述符 ABI 版本；验证只接受当前 V2 常量。
    pub abi_version: u32,
    /// 本描述符在当前 ABI 上的结构字节数；验证要求精确等于 [`Self::SIZE_BYTES`]，不接受仅含前缀的结构。
    pub size_bytes: usize,
    /// 宿主声明的能力位；必须恰含当前两项必需能力且无未知位。
    pub capability_bits: u64,
    /// 提交方声明的 Runtime API 协议版本，须不小于组合 V2 的最低版本；单独满足此字段不代表完成能力协商。
    pub peer_api_version: u32,
    /// 宿主声明的组合载荷 schema；验证要求与本实现匹配。
    pub composition_schema: u16,
    /// 宿主声明的候选矩形 schema；验证要求与本实现匹配。
    pub candidate_rect_schema: u16,
    /// 描述符保留字段；验证要求为零。
    pub reserved: u32,
    /// 宿主为窗口生命周期分配的非零代数；后续组合上下文须使用相应代数，由会话路由拒绝旧窗口事件。
    pub window_generation: u64,
    /// 随配置按值提交的窗口内候选矩形，不借用宿主缓冲区。
    pub candidate_rect: ZrRuntimeImeCandidateRectV2,
    /// 为描述符布局扩展保留的尾部字段；当前验证要求两项均为零。
    pub reserved_tail: [u64; 2],
}

impl ZrRuntimeAppSessionConfigurationV2 {
    /// 当前描述符类型在本编译目标上的完整字节尺寸，验证要求精确匹配。
    pub const SIZE_BYTES: usize = size_of::<Self>();

    /// 保留传入的窗口代数与矩形；提交会话配置前仍须通过 validate 或 negotiation。
    /// # 参数
    ///
    /// - `window_generation`：宿主窗口的生命周期代数；零值会被后续验证拒绝。
    /// - `candidate_rect`：宿主已经转换到窗口坐标的候选矩形，按值保存；本函数不校验其高度、保留字段或边界。
    ///
    /// # 返回与使用时机
    ///
    /// 返回声明两项必需能力、当前 ABI/schema、最低 peer API 版本且保留字段清零的待验证描述符。
    /// 本函数没有错误返回，不安装会话、不分配窗口代数，也不证明对端动态库支持配置符号。
    /// 宿主在原生组合事件发送前，用 [`Self::validate`] 或 [`Self::negotiation`] 检查内容，再提交配置入口。
    pub const fn ime_composition(
        window_generation: u64,
        candidate_rect: ZrRuntimeImeCandidateRectV2,
    ) -> Self {
        Self {
            abi_version: ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2,
            size_bytes: Self::SIZE_BYTES,
            capability_bits: REQUIRED_CAPABILITIES,
            peer_api_version: ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION,
            composition_schema: ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2,
            candidate_rect_schema: ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2,
            reserved: 0,
            window_generation,
            candidate_rect,
            reserved_tail: [0; 2],
        }
    }

    /// 检查完整 AppSession V2 描述符，供宿主预检及 Runtime 配置入口在进入会话前使用。
    ///
    /// # 返回与错误
    ///
    /// 成功返回 `Ok(())`，表示当前字段满足本实现的描述符契约；本函数只读 `self`，不会安装能力或修改会话。
    /// 检查按源码顺序短路，遇到第一项不符合的字段立即返回 [`ZrRuntimeAppSessionConfigurationError`]：
    ///
    /// - ABI 版本及精确结构尺寸不匹配，分别返回 `UnsupportedVersion`、`SizeMismatch`。
    /// - 包含未知能力位或缺少任一必需能力，分别返回 `UnknownCapability`、`MissingRequiredCapability`。
    /// - 描述符保留字段非零、窗口代数为零，分别返回 `NonZeroReserved`、`InvalidWindowGeneration`。
    /// - peer API 低于最低版本或两种 schema 不匹配，返回对应的版本/schema 错误。
    /// - 其余几何错误由候选矩形验证返回，包括坐标标签、高度、原生边界及矩形保留字段错误。
    ///
    /// # 使用时机与责任边界
    ///
    /// 字段可公开修改，调用方应在每次提交前检查当前值；修改不满足条件的字段后可再次调用。
    /// 成功不证明动态符号存在、Runtime 会话句柄有效或窗口仍存活，这些由加载器、配置入口及宿主分别负责。
    /// 参见 [`Self::ime_composition`]、[`Self::negotiation`] 和 [`ZrRuntimeImeCandidateRectV2`]。
    pub fn validate(&self) -> Result<(), ZrRuntimeAppSessionConfigurationError> {
        if self.abi_version != ZR_RUNTIME_APP_SESSION_CONFIGURATION_ABI_VERSION_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedVersion);
        }
        if self.size_bytes != Self::SIZE_BYTES {
            return Err(ZrRuntimeAppSessionConfigurationError::SizeMismatch);
        }
        if self.capability_bits & !REQUIRED_CAPABILITIES != 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnknownCapability);
        }
        if self.capability_bits & REQUIRED_CAPABILITIES != REQUIRED_CAPABILITIES {
            return Err(ZrRuntimeAppSessionConfigurationError::MissingRequiredCapability);
        }
        if self.reserved != 0 || self.reserved_tail != [0; 2] {
            return Err(ZrRuntimeAppSessionConfigurationError::NonZeroReserved);
        }
        if self.window_generation == 0 {
            return Err(ZrRuntimeAppSessionConfigurationError::InvalidWindowGeneration);
        }
        if self.peer_api_version < ZR_RUNTIME_IME_COMPOSITION_V2_MIN_API_VERSION {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedPeerApiVersion);
        }
        if self.composition_schema != ZR_RUNTIME_APP_SESSION_COMPOSITION_SCHEMA_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCompositionSchema);
        }
        if self.candidate_rect_schema != ZR_RUNTIME_APP_SESSION_CANDIDATE_RECT_SCHEMA_V2 {
            return Err(ZrRuntimeAppSessionConfigurationError::UnsupportedCandidateRectSchema);
        }
        self.candidate_rect.validate()
    }

    /// 从通过完整验证的描述符生成组合 V2 协商值，供会话配置入口安装路由。
    ///
    /// # 返回与错误
    ///
    /// 先调用 [`Self::validate`]；失败原样返回 [`ZrRuntimeAppSessionConfigurationError`]，不创建协商结果。
    /// 成功返回保存 peer API、组合 schema 和显式能力的 [`ZrRuntimeImeCompositionNegotiation`]。
    /// 当前验证约束保证该结果的 [`ZrRuntimeImeCompositionNegotiation::supports_v2`] 为真。
    ///
    /// # 生命周期与参考
    ///
    /// 返回值拥有所需字段，不借用 `self`；本函数只构造协商值，不安装会话或发送事件。
    /// 窗口代数和候选矩形仍由原描述符独立传入会话配置入口；参见 [`Self::validate`]。
    pub fn negotiation(
        &self,
    ) -> Result<ZrRuntimeImeCompositionNegotiation, ZrRuntimeAppSessionConfigurationError> {
        self.validate()?;
        Ok(ZrRuntimeImeCompositionNegotiation::from_raw(
            self.peer_api_version,
            Some(self.composition_schema),
            true,
        ))
    }
}

/// 描述 AppSession V2 的本地协议或几何验证失败。
///
/// # 使用时机与责任
///
/// [`ZrRuntimeAppSessionConfigurationV2::validate`] 按检查顺序返回首个错误，
/// [`ZrRuntimeAppSessionConfigurationV2::negotiation`] 传播同一错误。
/// 版本/schema、能力、结构尺寸/保留字段/代数和候选几何是不同的失败原因，可据此修正描述符后重新验证。
/// 该枚举不包含动态符号缺失、会话句柄失效或运行时派发失败；配置 FFI 另将这些本地错误映射到 ABI 状态。
///
/// # 参考
///
/// 参见 [`ZrRuntimeAppSessionConfigurationV2`] 和 [`ZrRuntimeImeCandidateRectV2`]。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeAppSessionConfigurationError {
    /// 描述符 ABI 版本不受本实现支持。
    UnsupportedVersion,
    /// 描述符尺寸不是当前类型的精确尺寸。
    SizeMismatch,
    /// 能力位包含当前实现未定义的位。
    UnknownCapability,
    /// 缺少组合 V2 或候选矩形 V2 任一必需能力。
    MissingRequiredCapability,
    /// 描述符、矩形或尾部保留字段不为零。
    NonZeroReserved,
    /// 窗口生命周期代数为零。
    InvalidWindowGeneration,
    /// 对端声明的 API 版本低于组合 V2 最低版本。
    UnsupportedPeerApiVersion,
    /// 组合 schema 与本实现不匹配。
    UnsupportedCompositionSchema,
    /// 候选矩形 schema 与本实现不匹配。
    UnsupportedCandidateRectSchema,
    /// 候选矩形坐标空间不是受支持的窗口坐标。
    UnsupportedCoordinateSpace,
    /// 候选矩形高度无效；当前校验拒绝零高度。
    InvalidCandidateRect,
    /// 宽、高或右边、下边超出原生有符号坐标范围。
    UnrepresentableCandidateRect,
}

impl core::fmt::Display for ZrRuntimeAppSessionConfigurationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let message = match self {
            Self::UnsupportedVersion => "unsupported AppSession V2 configuration ABI version",
            Self::SizeMismatch => "AppSession V2 configuration size does not match",
            Self::UnknownCapability => "AppSession V2 configuration contains an unknown capability",
            Self::MissingRequiredCapability => {
                "AppSession V2 configuration is missing a required capability"
            }
            Self::NonZeroReserved => "AppSession V2 configuration reserved fields must be zero",
            Self::InvalidWindowGeneration => "AppSession V2 window generation is invalid",
            Self::UnsupportedPeerApiVersion => "AppSession V2 peer API version is unsupported",
            Self::UnsupportedCompositionSchema => "AppSession V2 composition schema is unsupported",
            Self::UnsupportedCandidateRectSchema => {
                "AppSession V2 candidate rectangle schema is unsupported"
            }
            Self::UnsupportedCoordinateSpace => {
                "AppSession V2 candidate rectangle coordinate space is unsupported"
            }
            Self::InvalidCandidateRect => "AppSession V2 candidate rectangle height is invalid",
            Self::UnrepresentableCandidateRect => {
                "AppSession V2 candidate rectangle exceeds native coordinate bounds"
            }
        };
        formatter.write_str(message)
    }
}

#[cfg(test)]
#[path = "tests/app_session_config_v2.rs"]
mod tests;
