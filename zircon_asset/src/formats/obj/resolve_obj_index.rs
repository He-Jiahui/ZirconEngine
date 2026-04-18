pub(super) fn resolve_obj_index(value: &str, len: usize, label: &str) -> Result<usize, String> {
    if len == 0 {
        return Err(format!("missing source data for {label}"));
    }
    let index = value
        .parse::<isize>()
        .map_err(|error| format!("invalid {label} '{value}': {error}"))?;
    let resolved = if index > 0 {
        index - 1
    } else if index < 0 {
        len as isize + index
    } else {
        return Err(format!("{label} cannot be zero"));
    };
    if !(0..len as isize).contains(&resolved) {
        return Err(format!("{label} {value} is out of bounds"));
    }
    Ok(resolved as usize)
}
