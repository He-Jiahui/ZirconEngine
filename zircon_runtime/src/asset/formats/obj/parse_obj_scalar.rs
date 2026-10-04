use super::error::{ObjDecodeError, ObjDecodeResult};

/// 对坐标、UV 与法线使用同一带路径和行号的错误边界，供加载层区分内容错误与读取失败。
pub(super) fn parse_obj_scalar(
    value: Option<&str>,
    path: &str,
    line_index: usize,
    label: &str,
) -> ObjDecodeResult<f32> {
    let line = line_index + 1;
    let value = value.ok_or_else(|| ObjDecodeError::MissingScalar {
        path: path.to_string(),
        line,
        label: label.to_string(),
    })?;
    // BUG: [CR-ASSET-COOK-0002] NaN 等非有限标量会被浮点解析接受并进入成功的网格载荷，后续顶点构造未校验有限性；证据：decode_obj_file 与 pipeline/types.rs。
    value
        .parse::<f32>()
        .map_err(|source| ObjDecodeError::InvalidScalar {
            path: path.to_string(),
            line,
            label: label.to_string(),
            value: value.to_string(),
            source,
        })
}
