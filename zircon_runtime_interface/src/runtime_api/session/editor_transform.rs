//! 编辑器变换事件的固定布局载体；数值转换、本地结构校验与会话交互接纳各有边界。
//! 请求按值拥有变换，事件 payload 只借用该请求；参见 [`ZrRuntimeEditorTransformWriteV1::as_payload`]。

use core::mem::size_of;

use thiserror::Error;

use crate::buffer::{ZrByteSlice, ZrByteSliceError};
use crate::math::{Quat, Transform, Vec3};
use crate::ZIRCON_RUNTIME_ABI_VERSION_V1;

/// 编辑器交互阶段的固定整数标签，未知值由 [`Self::from_raw`] 返回 `None`。
///
/// Begin/Apply 的本地序号必须为 1，其他阶段大于 1；会话另核对连续序号、世界代数和期望变换。
#[repr(u32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZrRuntimeEditorTransformPhaseV1 {
    /// 登记交互起点；当前会话要求 expected、target 与实体当前局部变换一致。
    Begin = 1,
    /// 在活动交互内写入预览，成功后保留交互与最新前值。
    Preview = 2,
    /// 在活动交互内写入目标，成功后结束交互。
    Commit = 3,
    /// 在活动交互内恢复起始变换；target 必须为该起始值。
    Cancel = 4,
    /// 无活动交互时执行一次期望前值匹配的写入。
    Apply = 5,
}

impl ZrRuntimeEditorTransformPhaseV1 {
    /// 返回既定阶段整数，不读取或改变交互状态。
    pub const fn raw(self) -> u32 {
        self as u32
    }

    /// 将已知整数还原为阶段；未定义值返回 `None`，不检查请求序号。
    pub const fn from_raw(value: u32) -> Option<Self> {
        match value {
            1 => Some(Self::Begin),
            2 => Some(Self::Preview),
            3 => Some(Self::Commit),
            4 => Some(Self::Cancel),
            5 => Some(Self::Apply),
            _ => None,
        }
    }
}

/// Fixed-layout math carrier for editor transform traffic across the runtime ABI.
/// 以数组保存局部平移、四元数分量和缩放，按值复制，不借用数学对象。
/// [`Self::new`] 与 [`Self::transform`] 只转换表示；数值边界由 [`Self::is_valid`] 单独判断。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeTransformV1 {
    /// 局部平移三个分量；构造和还原都不改变数值。
    pub translation: [f32; 3],
    /// 四元数四分量，按 Quat 数组接口成对转换；本载体不归一化。
    pub rotation: [f32; 4],
    /// 局部缩放三个分量；结构验证拒绝零，但不拒绝负缩放。
    pub scale: [f32; 3],
}

impl ZrRuntimeTransformV1 {
    /// 从 `transform` 逐字段复制数组并返回载体，不验证或归一化；参见 [`Self::is_valid`]。
    pub fn new(transform: Transform) -> Self {
        Self {
            translation: transform.translation.to_array(),
            rotation: transform.rotation.to_array(),
            scale: transform.scale.to_array(),
        }
    }

    /// 按保存的数组还原并返回数学 Transform，不附加有效性保证或归一化。
    pub fn transform(self) -> Transform {
        Transform {
            translation: Vec3::from_array(self.translation),
            rotation: Quat::from_array(self.rotation),
            scale: Vec3::from_array(self.scale),
        }
    }

    /// 检查所有分量有限、四元数分量平方和大于 `f32::EPSILON` 且各缩放非零。
    /// 返回布尔值，不要求单位四元数，也不检查世界、实体或写入状态。
    pub fn is_valid(self) -> bool {
        self.translation
            .into_iter()
            .chain(self.rotation)
            .chain(self.scale)
            .all(f32::is_finite)
            && self
                .rotation
                .into_iter()
                .map(|component| component * component)
                .sum::<f32>()
                > f32::EPSILON
            && self.scale.into_iter().all(|component| component != 0.0)
    }
}

/// Allocation-free compare-and-set transform write for editor-owned runtime interaction.
/// 固定布局的局部变换比较后写入请求，expected 是前值，target 是目标值。
///
/// # 约束与生命周期
/// 构造不建立实体、交互或世界租约；[`Self::validate_editor_transform_write`] 只检查本地结构。
/// 当前会话再核对世界替换代数、活动交互、连续序号及当前局部变换；通过结构检查仍可能被拒绝。
/// 请求拥有两份数组，借给事件时须保持原地址和内容有效直至同步调用返回。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ZrRuntimeEditorTransformWriteV1 {
    /// 当前请求的 V1 ABI 版本，本地校验要求精确匹配。
    pub abi_version: u32,
    /// 阶段 raw 编码，由 [`Self::phase`] 解析，未知编码无效。
    pub phase: u32,
    /// 目标实体数值 ID；本地只检查非零，存在性由会话查世界。
    pub entity: u64,
    /// 宿主给定的非零交互身份；活动阶段须与原交互相同。
    pub interaction_id: u64,
    /// 交互序号；起始/单次写入为 1，活动阶段由会话检查逐次加一。
    pub sequence: u64,
    /// 期望的非零世界替换代数，由会话与当前世界核对。
    pub world_replacement_epoch: u64,
    /// 比较所用的局部变换前值；结构校验不读取世界当前值。
    pub expected: ZrRuntimeTransformV1,
    /// 待写入的局部变换；取消时由会话要求其等于交互起始值。
    pub target: ZrRuntimeTransformV1,
    /// 当前必须为零的保留字段。
    pub reserved: u64,
}

impl ZrRuntimeEditorTransformWriteV1 {
    /// 按给定身份、序号、代数、阶段与两份变换构造请求，并设置 V1、清零保留字段。
    /// 返回值尚未校验或提交；输入有效性参见 [`Self::validate_editor_transform_write`]。
    // EXEMPT(GEN-Q7): editor transform write construction mirrors the fixed runtime ABI fields.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        entity: u64,
        interaction_id: u64,
        sequence: u64,
        world_replacement_epoch: u64,
        phase: ZrRuntimeEditorTransformPhaseV1,
        expected: Transform,
        target: Transform,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            phase: phase.raw(),
            entity,
            interaction_id,
            sequence,
            world_replacement_epoch,
            expected: ZrRuntimeTransformV1::new(expected),
            target: ZrRuntimeTransformV1::new(target),
            reserved: 0,
        }
    }

    /// 只解析 phase 字段；未知值返回 `None`，不校验请求其他字段。
    pub const fn phase(self) -> Option<ZrRuntimeEditorTransformPhaseV1> {
        ZrRuntimeEditorTransformPhaseV1::from_raw(self.phase)
    }

    /// 按值还原 expected，不查询世界或重新校验；参见 [`ZrRuntimeTransformV1::transform`]。
    pub fn expected_transform(self) -> Transform {
        self.expected.transform()
    }

    /// 按值还原 target，不执行写入或重新校验；参见 [`ZrRuntimeTransformV1::transform`]。
    pub fn target_transform(self) -> Transform {
        self.target.transform()
    }

    /// ABI 事件入口在状态机前检查 phase 与序号约束；连续交互的期望步进还要由 runtime 会话状态核对。
    /// 返回当前版本、非零身份/代数、阶段序号规则、两份数值谓词和零保留字段的合取。
    /// 不检查实体存在、期望前值匹配或下一连续序号；这些由 Runtime 会话在消费时检查。
    pub fn validate_editor_transform_write(self) -> bool {
        let Some(phase) = self.phase() else {
            return false;
        };
        let sequence_valid = match phase {
            ZrRuntimeEditorTransformPhaseV1::Begin | ZrRuntimeEditorTransformPhaseV1::Apply => {
                self.sequence == 1
            }
            ZrRuntimeEditorTransformPhaseV1::Preview
            | ZrRuntimeEditorTransformPhaseV1::Commit
            | ZrRuntimeEditorTransformPhaseV1::Cancel => self.sequence > 1,
        };
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.entity != 0
            && self.interaction_id != 0
            && self.world_replacement_epoch != 0
            && sequence_valid
            && self.expected.is_valid()
            && self.target.is_valid()
            && self.reserved == 0
    }

    /// Borrows this request as a payload for one synchronous `handle_event` call.
    ///
    /// 返回指向 `self` 的精确结构尺寸借用，不分配、不复制，也不把 Rust 借用寿命编码进返回类型。
    /// 调用方须保持请求不移动、内容稳定且可读直至同步事件处理返回；参见 [`Self::from_payload`]。
    pub fn as_payload(&self) -> ZrByteSlice {
        ZrByteSlice {
            data: core::ptr::from_ref(self).cast(),
            len: size_of::<Self>(),
        }
    }

    /// Decodes one fixed-layout request borrowed for the duration of the ABI call.
    ///
    /// # Safety
    ///
    /// A non-empty payload pointer must address initialized bytes for the duration of this call.
    ///
    /// 以非对齐读取复制请求后执行本地结构验证；成功结果不借用原 payload。
    /// 调用方仍须保证区域位于同一分配内、已初始化且调用中可读不变，形状检查不能证明这些前提。
    /// 超长或非法描述符返回 Payload，短载荷返回 Size，本地结构错误返回 InvalidRequest。
    pub unsafe fn from_payload(
        payload: ZrByteSlice,
    ) -> Result<Self, ZrRuntimeEditorTransformError> {
        let bytes = unsafe { payload.checked_slice(size_of::<Self>()) }
            .map_err(ZrRuntimeEditorTransformError::Payload)?;
        if bytes.len() != size_of::<Self>() {
            return Err(ZrRuntimeEditorTransformError::Size {
                observed: bytes.len(),
                expected: size_of::<Self>(),
            });
        }
        let request = unsafe { core::ptr::read_unaligned(bytes.as_ptr().cast::<Self>()) };
        if !request.validate_editor_transform_write() {
            return Err(ZrRuntimeEditorTransformError::InvalidRequest);
        }
        Ok(request)
    }
}

/// 编辑器变换载荷的描述符、精确尺寸或本地结构错误；不包含会话状态机的接纳错误。
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ZrRuntimeEditorTransformError {
    #[error("invalid editor transform payload carrier: {0:?}")]
    /// 字节描述符未满足有界读取形状要求，保留具体载体错误。
    Payload(ZrByteSliceError),
    #[error("editor transform payload has {observed} bytes; expected {expected}")]
    /// 描述符通过有界检查后，载荷仍短于固定布局尺寸；携带观察值与要求值。
    Size { observed: usize, expected: usize },
    #[error("editor transform request is structurally invalid")]
    /// 请求字段未通过本地结构谓词；未查询世界状态。
    InvalidRequest,
}
