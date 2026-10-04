use super::error::{ObjDecodeError, ObjDecodeResult};

/// OBJ 正索引从一开始，负索引相对当前已读取的数组末尾；面解析须在源数组读取后调用。
pub(super) fn resolve_obj_index(value: &str, len: usize, label: &str) -> ObjDecodeResult<usize> {
    if len == 0 {
        return Err(ObjDecodeError::MissingSourceData {
            label: label.to_string(),
        });
    }
    let index = value
        .parse::<isize>()
        .map_err(|source| ObjDecodeError::InvalidIndex {
            label: label.to_string(),
            value: value.to_string(),
            source,
        })?;
    let resolved = if index > 0 {
        index - 1
    } else if index < 0 {
        len as isize + index
    } else {
        return Err(ObjDecodeError::ZeroIndex {
            label: label.to_string(),
        });
    };
    if !(0..len as isize).contains(&resolved) {
        return Err(ObjDecodeError::IndexOutOfBounds {
            label: label.to_string(),
            value: value.to_string(),
        });
    }
    Ok(resolved as usize)
}
