use super::error::{ObjDecodeError, ObjDecodeResult};

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
