//! 宿主文本和整数进入 Runtime ABI 前的局部适配。

use zircon_runtime_interface::ZrByteSlice;

/// 借用本次回调的文本；返回的 ABI 视图不得保存到同步事件调用结束之后。
pub(in crate::entry::runtime_entry_app) fn byte_slice(value: &str) -> ZrByteSlice {
    ZrByteSlice {
        data: value.as_bytes().as_ptr(),
        len: value.len(),
    }
}

// TODO: [CR-APP-ENTRY-0020] 核对 IME 游标是否可达到 u32 顶值；helper 对可表示顶值原样返回，而 Runtime 将其解释为隐藏游标，尚缺上游范围与负载预算共同排除该值的证据。
/// IME 的 usize 字节计数不可表示为 u32 时回退到 MAX-1；可表示值原样传递，包括顶值。
pub(in crate::entry::runtime_entry_app) fn usize_to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX - 1)
}

#[cfg(test)]
#[path = "tests/abi.rs"]
mod tests;
