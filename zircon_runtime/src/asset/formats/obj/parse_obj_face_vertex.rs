use super::error::ObjDecodeResult;
use super::obj_vertex_key::ObjVertexKey;
use super::resolve_obj_index::resolve_obj_index;

/// 将单个面顶点的索引绑定到目前已读入的源数组；解析后的键供解码器保留接缝。
pub(super) fn parse_obj_face_vertex(
    token: &str,
    position_count: usize,
    uv_count: usize,
    normal_count: usize,
) -> ObjDecodeResult<ObjVertexKey> {
    let (position_value, uv_value, normal_value) = obj_face_vertex_components(token);
    let position = resolve_obj_index(position_value, position_count, "position index")?;
    let uv = match uv_value {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, uv_count, "uv index")?),
    };
    let normal = match normal_value {
        Some("") | None => None,
        Some(value) => Some(resolve_obj_index(value, normal_count, "normal index")?),
    };

    Ok(ObjVertexKey {
        position,
        uv,
        normal,
    })
}

// 面可能有大量顶点，拆分时只定位分隔符以避免构造临时集合。
fn obj_face_vertex_components(token: &str) -> (&str, Option<&str>, Option<&str>) {
    let mut separators = [0usize; 3];
    let mut separator_count = 0usize;
    for (index, byte) in token.bytes().enumerate() {
        if byte == b'/' {
            separators[separator_count] = index;
            separator_count += 1;
            if separator_count == separators.len() {
                break;
            }
        }
    }

    if separator_count == 0 {
        return (token, None, None);
    }
    let first_separator = separators[0];
    let position = &token[..first_separator];
    let uv_start = first_separator + 1;
    if separator_count == 1 {
        return (position, Some(&token[uv_start..]), None);
    }
    let second_separator = separators[1];
    let normal_start = second_separator + 1;
    // TODO: [CR-ASSET-COOK-0003] 确认第三个斜杠后的尾段被忽略是否为预期兼容行为；缺少异常 OBJ 输入契约；下一步对照项目导入器和格式测试。
    let normal_end = if separator_count == 3 {
        separators[2]
    } else {
        token.len()
    };
    (
        position,
        Some(&token[uv_start..second_separator]),
        Some(&token[normal_start..normal_end]),
    )
}

#[cfg(test)]
#[path = "tests/parse_obj_face_vertex.rs"]
mod tests;
