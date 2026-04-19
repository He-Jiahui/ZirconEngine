pub(super) fn parse_obj_scalar(
    value: Option<&str>,
    path: &str,
    line_index: usize,
    label: &str,
) -> Result<f32, String> {
    let value = value.ok_or_else(|| format!("missing {label} at {path}:{}", line_index + 1))?;
    value.parse::<f32>().map_err(|error| {
        format!(
            "invalid {label} '{value}' at {path}:{}: {error}",
            line_index + 1
        )
    })
}
