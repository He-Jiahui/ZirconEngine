//! 定义资源获取、视口帧和无障碍树请求，以及 Runtime-owned 帧输出。
//!
//! 构造器只保存字段；实际请求版本、视口和尺寸由消费路径检查。借用 URI 与 Runtime
//! 分配输出分别受原内存 owner 和原 session 的生命周期约束。
//! # 参考
//! [`ZrRuntimeFrameRequestV1`]、[`ZrRuntimeFrameV2`]、
//! [`crate::runtime_api::ZrRuntimeHostFetchFnV1`]。

use crate::buffer::{ZrByteSlice, ZrOwnedResultV2};
use crate::handles::ZrRuntimeViewportHandle;

use super::viewport::ZrRuntimeViewportSizeV1;

/// Host fetch_resource 槽的固定布局请求，URI 是调用方内存的同步借用。
///
/// # 使用时机与生命周期
/// new/Copy 不复制 URI 字节或延长存活期；实际调用必须保持整段 URI 可读、不变。
/// 当前动态 API 获取入口只核对 Host 表形状，未保存或调用 fetch_resource 槽；
/// 本类型不规定 URI 解码、权限或 flags 组合校验，实际 Host 实现须建立这些约束。
/// # 参考
/// [`crate::runtime_api::ZrRuntimeHostFetchFnV1`]、`runtime_host_fetch_request_declares_streaming_resource_contract`。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeHostFetchRequestV1 {
    /// 原样保存的请求版本，构造器不判定是否被实际 Host 接受。
    pub abi_version: u32,
    /// URI 字节借用；非空和长度检查不能建立地址可读性或编码有效性。
    pub uri: ZrByteSlice,
    /// 资源请求标志位，构造器不屏蔽未知位；streaming 位见公开 constants。
    pub flags: u32,
}

impl ZrRuntimeHostFetchRequestV1 {
    /// 保存版本、URI 描述符和 flags，返回请求，不复制字节或调用资源获取。
    /// # 参考
    /// [`crate::runtime_api::ZrRuntimeHostFetchFnV1`]。
    pub const fn new(abi_version: u32, uri: ZrByteSlice, flags: u32) -> Self {
        Self {
            abi_version,
            uri,
            flags,
        }
    }
}

/// capture_frame 与 present_viewport 共用的固定布局请求。
///
/// # 约束与生命周期
/// 当前动态消费先核对 V1、默认视口和帧尺寸，再调整 Runtime 视口；new 不做这些检查。
/// 请求本身不拥有视口，也不保证捕获时渲染已产出非空帧。
/// # 参考
/// [`crate::runtime_api::ZrRuntimeCaptureFrameFnV2`]、[`crate::runtime_api::ZrRuntimePresentViewportFnV1`]、
/// `RuntimeDynamicSession::capture_frame`。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 同一视口的捕获与呈现请求；宿主传入目标尺寸，运行时负责生成对应帧产品。
pub struct ZrRuntimeFrameRequestV1 {
    /// 当前消费要求的 V1 请求版本，与 FrameV2 输出的 V2 区分。
    pub abi_version: u32,
    /// 目标视口身份，当前动态实现只接受默认视口。
    pub viewport: ZrRuntimeViewportHandle,
    /// 目标像素尺寸，消费时受帧大小/字节预算检查。
    pub size: ZrRuntimeViewportSizeV1,
}

impl ZrRuntimeFrameRequestV1 {
    /// 保存版本、视口和 size，返回请求，不验证尺寸或远端视口归属。
    /// # 参考
    /// `capture_frame`、`present_viewport`、[`ZrRuntimeFrameV2`]。
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
        }
    }
}

/// 无障碍树捕获的固定布局请求，结果由原 Runtime 会话的拥有者 JSON 交付。
///
/// # 使用时机与约束
/// 当前动态消费核对 V1、默认视口和尺寸后生成快照；generation_hint 只随请求保存，
/// 当前快照实现未读取此字段，不能据此推断缓存命中或返回未修改状态。
/// # 参考
/// [`crate::runtime_api::ZrRuntimeCaptureAccessibilityTreeFnV2`]、
/// `RuntimeDynamicSession::capture_accessibility_tree`、`runtime_accessibility_tree_request_constructor_preserves_generation_hint`。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZrRuntimeAccessibilityTreeRequestV1 {
    /// 当前无障碍捕获入口要求的 V1 版本。
    pub abi_version: u32,
    /// 树快照目标视口，当前动态实现接受默认视口。
    pub viewport: ZrRuntimeViewportHandle,
    /// 构建布局时的像素尺寸，由消费路径验证后调整视口。
    pub size: ZrRuntimeViewportSizeV1,
    /// 调用方传入的代数提示，构造器保值；当前 Runtime 快照生成不消费它。
    pub generation_hint: u64,
}

impl ZrRuntimeAccessibilityTreeRequestV1 {
    /// 保存四个输入并返回请求，不验证版本、视口、尺寸或 generation_hint。
    /// # 参考
    /// `runtime_accessibility_tree_request_constructor_preserves_generation_hint`、
    /// [`crate::runtime_api::ZrRuntimeCaptureAccessibilityTreeFnV2`]。
    pub const fn new(
        abi_version: u32,
        viewport: ZrRuntimeViewportHandle,
        size: ZrRuntimeViewportSizeV1,
        generation_hint: u64,
    ) -> Self {
        Self {
            abi_version,
            viewport,
            size,
            generation_hint,
        }
    }
}

/// 捕获得到的固定布局帧与 RGBA allocation 描述符，避免隐式复制释放权。
///
/// # 生命周期与约束
/// Host 先确认调用状态，再核对 V2、尺寸/字节形状和输出所有权；非空 RGBA 在读取
/// 期间由原 provider/session 保持有效，成功或验证失败后都通过原 allocation 槽释放一次。
/// empty/is_empty 不是完整协议验证；无帧产品可用规范空输出表示。
/// # 参考
/// [`crate::runtime_api::ZrRuntimeCaptureFrameFnV2`]、[`ZrOwnedResultV2`]、
/// `validate_runtime_frame_rgba_shape`、`RuntimeFrame::rgba`。
#[repr(C)]
#[derive(Debug)]
/// 捕获帧的输出参数；RGBA 缓冲区由运行时分配，宿主在成功和失败路径均按原会话释放。
pub struct ZrRuntimeFrameV2 {
    /// 帧输出的 V2 版本，构造 empty 时按调用方输入保存。
    pub abi_version: u32,
    /// RGBA 产品宽度，与 height、字节长度共同验证。
    pub width: u32,
    /// RGBA 产品高度，单独非零不能证明完整帧有效。
    pub height: u32,
    /// producer 报告的帧代数；empty 清零，is_empty 不检查此字段。
    pub generation: u64,
    /// 原会话拥有的 RGBA 字节及 allocation ID；此字段承担唯一释放责任。
    pub rgba: ZrOwnedResultV2,
}

impl ZrRuntimeFrameV2 {
    /// 构造清零输出位置，保存指定版本并使尺寸、代数和 RGBA 都为空。
    ///
    /// 不核对版本，也不释放调用方先前仍持有的帧；用于调用前初始化新输出位置。
    /// # 参考
    /// [`Self::is_empty`]、`RuntimeSession::capture_frame`。
    pub const fn empty(abi_version: u32) -> Self {
        Self {
            abi_version,
            width: 0,
            height: 0,
            generation: 0,
            rgba: ZrOwnedResultV2::empty(),
        }
    }

    /// 按尺寸和 RGBA 判断是否为空形状，返回 bool，不读取像素或释放 allocation。
    ///
    /// abi_version 与 generation 不参与本判断；调用方须另行执行完整版本/形状校验。
    /// # 参考
    /// `RuntimeSession::capture_frame`、`validate_runtime_frame_rgba_shape`。
    /// Reports only the canonical cleared out-parameter state, never a malformed frame.
    /// 仅判断尺寸与 RGBA 输出是否为空；调用方仍须核对调用状态、ABI 版本和帧代数。
    pub const fn is_empty(&self) -> bool {
        self.width == 0 && self.height == 0 && self.rgba.is_empty()
    }
}
