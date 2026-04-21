use toml::Value;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum UiAssetTomlPathSegment {
    Key(String),
    Index(usize),
}

pub(crate) fn parse_value_path(path: &str) -> Option<Vec<UiAssetTomlPathSegment>> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut segments = Vec::new();
    let chars = trimmed.chars().collect::<Vec<_>>();
    let mut index = 0usize;
    while index < chars.len() {
        match chars[index] {
            '.' => index += 1,
            '[' => {
                index += 1;
                let start = index;
                while index < chars.len() && chars[index] != ']' {
                    index += 1;
                }
                if index == start || index >= chars.len() {
                    return None;
                }
                let value = chars[start..index].iter().collect::<String>();
                let parsed = value.trim().parse::<usize>().ok()?;
                segments.push(UiAssetTomlPathSegment::Index(parsed));
                index += 1;
            }
            _ => {
                let start = index;
                while index < chars.len() && chars[index] != '.' && chars[index] != '[' {
                    index += 1;
                }
                let value = chars[start..index].iter().collect::<String>();
                let value = value.trim();
                if value.is_empty() {
                    return None;
                }
                segments.push(UiAssetTomlPathSegment::Key(value.to_string()));
            }
        }
    }

    (!segments.is_empty()).then_some(segments)
}

pub(crate) fn get_value_at_path<'a>(
    value: &'a Value,
    path: &[UiAssetTomlPathSegment],
) -> Option<&'a Value> {
    let mut current = value;
    for segment in path {
        current = match segment {
            UiAssetTomlPathSegment::Key(key) => current.as_table()?.get(key)?,
            UiAssetTomlPathSegment::Index(index) => current.as_array()?.get(*index)?,
        };
    }
    Some(current)
}

pub(crate) fn set_value_at_path(
    value: &mut Value,
    path: &[UiAssetTomlPathSegment],
    next_value: Option<Value>,
) -> Result<(), String> {
    if path.is_empty() {
        return Err("value path is required".to_string());
    }

    let _ = mutate_value_at_path(value, path, next_value)?;
    Ok(())
}

fn mutate_value_at_path(
    value: &mut Value,
    path: &[UiAssetTomlPathSegment],
    next_value: Option<Value>,
) -> Result<bool, String> {
    let Some((head, tail)) = path.split_first() else {
        return Err("value path is required".to_string());
    };
    match head {
        UiAssetTomlPathSegment::Key(key) => {
            let Some(table) = value.as_table_mut() else {
                return Err(format!("path segment `{key}` requires an object"));
            };
            if tail.is_empty() {
                match next_value {
                    Some(next_value) => {
                        let _ = table.insert(key.clone(), next_value);
                    }
                    None => {
                        if table.remove(key).is_none() {
                            return Err(format!("object entry `{key}` is missing"));
                        }
                    }
                }
                return Ok(table.is_empty());
            }

            if !table.contains_key(key) {
                let Some(container) = default_container_for(tail.first()) else {
                    return Err(format!("object entry `{key}` is missing"));
                };
                let _ = table.insert(key.clone(), container);
            }
            let remove_child = {
                let child = table
                    .get_mut(key)
                    .expect("child should exist after initialization");
                mutate_value_at_path(child, tail, next_value)?
            };
            if remove_child {
                let _ = table.remove(key);
            }
            Ok(table.is_empty())
        }
        UiAssetTomlPathSegment::Index(index) => {
            let Some(array) = value.as_array_mut() else {
                return Err(format!("path segment `[{index}]` requires a collection"));
            };
            if tail.is_empty() {
                match next_value {
                    Some(next_value) => {
                        if *index < array.len() {
                            array[*index] = next_value;
                        } else if *index == array.len() {
                            array.push(next_value);
                        } else {
                            return Err(format!("collection entry index {index} is out of range"));
                        }
                    }
                    None => {
                        if *index >= array.len() {
                            return Err(format!("collection entry index {index} is out of range"));
                        }
                        array.remove(*index);
                    }
                }
                return Ok(array.is_empty());
            }

            while array.len() <= *index {
                let Some(container) = default_container_for(tail.first()) else {
                    return Err(format!("collection entry index {index} is out of range"));
                };
                array.push(container);
            }
            let remove_child = {
                let child = array
                    .get_mut(*index)
                    .expect("collection child should exist after initialization");
                mutate_value_at_path(child, tail, next_value)?
            };
            if remove_child {
                array.remove(*index);
            }
            Ok(array.is_empty())
        }
    }
}

fn default_container_for(segment: Option<&UiAssetTomlPathSegment>) -> Option<Value> {
    match segment {
        Some(UiAssetTomlPathSegment::Key(_)) => Some(Value::Table(Default::default())),
        Some(UiAssetTomlPathSegment::Index(_)) => Some(Value::Array(Vec::new())),
        None => None,
    }
}
