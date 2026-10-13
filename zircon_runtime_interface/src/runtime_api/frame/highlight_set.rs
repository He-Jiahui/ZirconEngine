//! 定义 Editor 到 Runtime 的视口高亮 ABI 值及临时实体数组借用。
//!
//! `submit_highlight_set` 是同步入口：调用方须让实体数组在 ABI 调用中可读，Runtime FFI 会在进入
//! 会话闭包前复制编号，再以拥有型数据验证并提交；会话状态不持有这里的借用指针。实体与样式的
//! 合法性由本模块的 ABI 检查和 Runtime 按视口代数更新的高亮存储共同约束。

use crate::handles::ZrRuntimeViewportHandle;
use crate::version::ZIRCON_RUNTIME_ABI_VERSION_V1;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// 跨 ABI 临时借用的实体 ID 数组；切片本身不拥有数据。
/// 提交端须保持数组在同步调用返回前可读，运行时入口在交给会话 owner 前复制它。
///
/// `from_slice` 仅保存切片指针和长度，不分配或复制实体编号。Runtime 的
/// `submit_highlight_set` FFI 在进入会话闭包前立即复制这些编号，因此提交端只需保证源数组
/// 在该同步 ABI 调用完成前持续有效且不被不兼容地修改。空切片的指针不被解引用。
pub struct ZrRuntimeEntityIdSliceV1 {
    /// 借用数组首元素的指针；`len == 0` 时不要求该指针可解引用。
    pub data: *const u64,
    /// 从 `data` 开始可读取的实体编号数。
    pub len: usize,
}

impl ZrRuntimeEntityIdSliceV1 {
    /// 构造不含实体编号的空描述符。
    pub const fn empty() -> Self {
        Self {
            data: core::ptr::null(),
            len: 0,
        }
    }

    /// 创建借用 `values` 的 ABI 描述符，不复制底层编号。
    ///
    /// # 参数
    ///
    /// `values` 是同步提交时供 Runtime 读取的编号切片。
    ///
    /// # 使用时机与生命周期
    ///
    /// 返回值不延长 `values` 的生命周期；调用者须让数组保持可读，直到接收方完成读取。Runtime
    /// FFI 在提交调用内复制数据后，会话状态不再借用该数组。
    pub fn from_slice(values: &[u64]) -> Self {
        Self {
            data: values.as_ptr(),
            len: values.len(),
        }
    }

    /// # Safety
    ///
    /// The caller must ensure `data` points to `len` readable `u64` values for the returned
    /// slice lifetime.
    ///
    /// 当 `len > 0` 时，调用者还须保证 `data` 指向同一存活分配中的 `len` 个已初始化、可读的
    /// `u64`，地址满足 `u64` 对齐，且该内存至少在返回切片的借用期内保持有效并且不被不兼容地
    /// 修改。此方法会拒绝空指针、未对齐指针及超过 Rust 切片上限的长度，但无法验证分配大小、
    /// 初始化状态或有效期。`len == 0` 时返回空切片而不读取或解引用 `data`。
    pub unsafe fn as_slice(&self) -> Option<&[u64]> {
        if self.len == 0 {
            return Some(&[]);
        }
        let max_elements = isize::MAX as usize / core::mem::size_of::<u64>();
        if self.data.is_null()
            || (self.data as usize) % core::mem::align_of::<u64>() != 0
            || self.len > max_elements
        {
            return None;
        }
        // SAFETY: 调用者须保证该指针覆盖同一存活分配中的已初始化可读元素并在返回借用期内有效；
        // 上方仅能筛除空指针、未对齐和超出 Rust 切片长度上限的形状，不能证明分配范围或初始化。
        Some(unsafe { core::slice::from_raw_parts(self.data, self.len) })
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// 渲染高亮覆盖层使用的样式值；验证只接受已定义的描边开关和有限颜色分量。
pub struct ZrRuntimeHighlightRenderAttributesV1 {
    /// 描边开关的 ABI 整数值，仅 `OUTLINE_DISABLED` 或 `OUTLINE_ENABLED` 有效。
    pub outline_enabled: u32,
    /// RGBA 颜色分量；有效性要求每项有限，不额外约束取值区间。
    pub tint_rgba: [f32; 4],
}

impl ZrRuntimeHighlightRenderAttributesV1 {
    /// 描边关闭时 `outline_enabled` 使用的 ABI 值。
    pub const OUTLINE_DISABLED: u32 = 0;
    /// 描边开启时 `outline_enabled` 使用的 ABI 值。
    pub const OUTLINE_ENABLED: u32 = 1;

    /// 构造开启描边且使用给定 RGBA 分量的样式值。
    ///
    /// 此构造器保留输入分量；提交前可用 `is_valid` 拒绝非有限值。
    pub const fn outlined(tint_rgba: [f32; 4]) -> Self {
        Self {
            outline_enabled: Self::OUTLINE_ENABLED,
            tint_rgba,
        }
    }

    /// 检查描边开关是否已定义且所有颜色分量均为有限数。
    ///
    /// 返回 `false` 时 Runtime 的视口高亮存储不会接受该样式；该检查不限制颜色分量区间。
    pub fn is_valid(self) -> bool {
        matches!(
            self.outline_enabled,
            Self::OUTLINE_DISABLED | Self::OUTLINE_ENABLED
        ) && self.tint_rgba.iter().all(|value| value.is_finite())
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq)]
/// 编辑器叠加高亮的视口、提交代数和借用实体集；渲染状态存储拒绝用旧代数覆盖较新高亮。
///
/// Editor Gateway 在报告高亮能力后同步提交此值。Runtime FFI 先将实体切片复制到拥有型
/// `Vec`，再在会话线程内重新构造并验证描述符，随后交给 `RuntimeDynamicSession`；后者构造
/// 排序去重的 `HighlightSet`，并按视口替换最新值。低于当前代数的提交被忽略，等代提交仍可
/// 更新样式或实体集。
pub struct ZrRuntimeHighlightSetV1 {
    /// ABI 版本；`new` 填入当前 V1 常量。
    pub abi_version: u32,
    /// 高亮状态所属视口句柄。
    pub viewport: ZrRuntimeViewportHandle,
    /// 提交该覆盖层时对应的选择代数；按视口比较以拒绝过期提交。
    pub generation: u64,
    /// 借用的实体编号数组；生命周期约束与 `ZrRuntimeEntityIdSliceV1` 相同。
    pub entities: ZrRuntimeEntityIdSliceV1,
    /// 本次覆盖层使用的描边与颜色属性。
    pub attributes: ZrRuntimeHighlightRenderAttributesV1,
}

impl ZrRuntimeHighlightSetV1 {
    /// 将视口、代数、借用切片和样式组合为 V1 ABI 请求。
    ///
    /// # 参数
    ///
    /// `viewport` 标识目标视口，`generation` 标识提交代数，`entities` 是借用的实体 ID 切片，
    /// `attributes` 是渲染样式。此方法不复制切片或验证这些值。
    ///
    /// # 使用时机与生命周期
    ///
    /// 返回值通常只在同步 `submit_highlight_set` 调用期间有效；调用者须保持 `entities` 的
    /// 底层数组可读，直到 Runtime FFI 完成前置复制。
    pub fn new(
        viewport: ZrRuntimeViewportHandle,
        generation: u64,
        entities: &[u64],
        attributes: ZrRuntimeHighlightRenderAttributesV1,
    ) -> Self {
        Self {
            abi_version: ZIRCON_RUNTIME_ABI_VERSION_V1,
            viewport,
            generation,
            entities: ZrRuntimeEntityIdSliceV1::from_slice(entities),
            attributes,
        }
    }

    /// # Safety
    ///
    /// The borrowed entity slice must remain readable for this validation.
    ///
    /// 当实体长度非零时，调用者必须保证底层分配含有完整长度的已初始化 `u64` 数组，满足对齐且
    /// 在验证期间保持有效、可读和不被不兼容地修改。验证会检查 ABI 版本、视口句柄、样式和切片
    /// 形状；它不能证明实体编号当前存在，也不拥有或延长数组生命周期。
    pub unsafe fn validate(self) -> bool {
        // SAFETY: validate 的调用者必须为非空实体切片保证完整、已初始化、对齐且存活的底层分配；
        // as_slice 的形状筛选本身不验证这些内存前提。
        self.abi_version == ZIRCON_RUNTIME_ABI_VERSION_V1
            && self.viewport.is_valid()
            && self.attributes.is_valid()
            && unsafe { self.entities.as_slice() }.is_some()
    }
}

#[cfg(test)]
#[path = "tests/highlight_set.rs"]
mod tests;
