use zircon_runtime_interface::{
    ZrByteSlice, ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1,
    ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1,
};

use super::{AbiDecodeError, AbiDecodeResult};

/// 把插件提供的字符串列表复制到宿主内存，供系统注册与排序约束使用。
///
/// # Safety
///
/// 非空列表指针必须指向调用期间有效、已初始化的 `count` 个元素；本函数另行拒绝空指针、超限数量和未对齐地址。
pub(in super::super) unsafe fn read_byte_slices(
    values: *const ZrByteSlice,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null()
        || count > ZR_RUNTIME_NATIVE_STRING_LIST_MAX_ITEMS_V1
        || count > isize::MAX as usize / std::mem::size_of::<ZrByteSlice>()
        || values.align_offset(std::mem::align_of::<ZrByteSlice>()) != 0
    {
        return Err(AbiDecodeError::InvalidV4StringListPointer {
            field: "string list",
            count,
        });
    }
    // SAFETY: 已排除空指针、未对齐地址及不可表示的跨度；已初始化且同分配区的存储由列表输入合约限定。
    unsafe { std::slice::from_raw_parts(values, count) }
        .iter()
        .copied()
        .map(|slice| unsafe { read_utf8(slice) })
        .collect()
}

/// V4 字段名会进入解码错误；实际内存形状由共用列表读取器检查。
pub(in super::super) unsafe fn read_v4_byte_slices(
    field: &'static str,
    values: *const ZrByteSlice,
    count: usize,
) -> AbiDecodeResult<Vec<String>> {
    if count == 0 {
        return Ok(Vec::new());
    }
    if values.is_null() {
        return Err(AbiDecodeError::InvalidV4StringListPointer { field, count });
    }
    unsafe { read_byte_slices(values, count) }
}

pub(in super::super) unsafe fn read_utf8(slice: ZrByteSlice) -> AbiDecodeResult<String> {
    unsafe { read_utf8_with(slice, str::to_string) }
}

/// 在借用的 ABI 字节视图仍有效时执行映射，避免无条件建立中间字符串。
///
/// # Safety
///
/// 非空字节指针必须在本次同步映射期间可读；形状、长度与 UTF-8 由下层检查。
pub(in super::super) unsafe fn read_utf8_with<T>(
    slice: ZrByteSlice,
    visitor: impl FnOnce(&str) -> T,
) -> AbiDecodeResult<T> {
    // SAFETY: 输入合约要求非空 data 覆盖 len 个已初始化字节；checked_slice 另外校验空指针和长度上限。
    let bytes = unsafe { slice.checked_slice(ZR_RUNTIME_NATIVE_STRING_MAX_ENCODED_BYTES_V1) }
        .map_err(|_| AbiDecodeError::InvalidV4StringListPointer {
            field: "string value",
            count: slice.len,
        })?;
    let value =
        std::str::from_utf8(bytes).map_err(|source| AbiDecodeError::InvalidUtf8 { source })?;
    Ok(visitor(value))
}

#[cfg(test)]
#[path = "tests/read.rs"]
mod tests;
