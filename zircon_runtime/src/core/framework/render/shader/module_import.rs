use std::fmt;

pub const GENERATED_MATERIAL_MODULE_IMPORT_PATH: &str = "self::material";
pub const SHADER_IMPORT_PROJECT_NAMESPACE_SETTING: &str = "__zircon_shader_project_namespace";
pub const SHADER_SELF_MODULE_NAMESPACE: &str = "self";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderImportPathDerivation {
    pub import_path: String,
    pub folded_terminal_directory: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ShaderImportPathDerivationError {
    EmptyProjectNamespace,
    EmptyAssetPath,
    MissingShaderRoot { path: String },
    EmptyModulePath { path: String },
    EmptyModuleSegment { path: String },
    ReservedNamespace { namespace: String },
}

impl fmt::Display for ShaderImportPathDerivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProjectNamespace => {
                write!(formatter, "shader import path project namespace is empty")
            }
            Self::EmptyAssetPath => write!(formatter, "shader import path asset path is empty"),
            Self::MissingShaderRoot { path } => write!(
                formatter,
                "shader import path asset `{path}` is outside the shaders/ root"
            ),
            Self::EmptyModulePath { path } => {
                write!(
                    formatter,
                    "shader import path asset `{path}` has no module path"
                )
            }
            Self::EmptyModuleSegment { path } => write!(
                formatter,
                "shader import path asset `{path}` produces an empty module segment"
            ),
            Self::ReservedNamespace { namespace } => write!(
                formatter,
                "shader import path namespace `{namespace}` is reserved"
            ),
        }
    }
}

impl std::error::Error for ShaderImportPathDerivationError {}

/// include 解析只读取非注释行并返回逻辑路径；剥离指令后再交给 WGSL 编译，模块路径推导另行校验 shaders 根和保留命名空间。
pub fn wgsl_include_paths(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(wgsl_include_path_from_line)
        .collect()
}

pub fn strip_wgsl_include_directives(source: &str) -> String {
    let mut stripped = String::with_capacity(source.len());
    let mut has_output_line = false;
    for line in source
        .lines()
        .filter(|line| wgsl_include_path_from_line(line).is_none())
    {
        if has_output_line {
            stripped.push('\n');
        }
        stripped.push_str(line);
        has_output_line = true;
    }
    stripped
}

#[cfg(test)]
#[path = "module_import/tests/include_strip_tests.rs"]
mod include_strip_tests;

pub fn is_generated_shader_module_token(token: &str) -> bool {
    token
        .strip_prefix(SHADER_SELF_MODULE_NAMESPACE)
        .is_some_and(|rest| rest.starts_with("::"))
}

pub fn is_builtin_shader_module_token(token: &str) -> bool {
    token.starts_with("zr_") || token.ends_with(".wgsl") && token.starts_with("zr")
}

pub fn shader_project_namespace_from_name(name: &str) -> String {
    let mut namespace = String::new();
    let mut previous_underscore = false;
    for character in name.chars() {
        if character.is_ascii_alphanumeric() {
            namespace.push(character.to_ascii_lowercase());
            previous_underscore = false;
        } else if !previous_underscore && !namespace.is_empty() {
            namespace.push('_');
            previous_underscore = true;
        }
    }
    while namespace.ends_with('_') {
        namespace.pop();
    }
    if namespace.is_empty() {
        namespace.push_str("project");
    }
    if namespace
        .as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_digit())
    {
        namespace.insert(0, '_');
    }
    namespace
}

pub fn derive_shader_import_path(
    project_namespace: &str,
    asset_path: &str,
) -> Result<ShaderImportPathDerivation, ShaderImportPathDerivationError> {
    let namespace = shader_import_namespace(project_namespace)?;
    let normalized_path = normalized_shader_asset_path(asset_path)?;
    let (module_segments, strip_terminal_extension) =
        shader_module_path_segments(&normalized_path)?;
    let capacity = namespace.len()
        + module_segments.iter().map(String::len).sum::<usize>()
        + module_segments.len() * 3;
    let mut import_path = String::with_capacity(capacity);
    import_path.push_str(&namespace);
    for (index, segment) in module_segments.iter().enumerate() {
        import_path.push_str("::");
        let segment = if strip_terminal_extension && index + 1 == module_segments.len() {
            strip_shader_asset_extension(segment)
        } else {
            segment
        };
        shader_module_segment(&mut import_path, &normalized_path, segment)?;
    }
    Ok(ShaderImportPathDerivation {
        import_path,
        folded_terminal_directory: terminal_directory_was_folded(&normalized_path),
    })
}

fn wgsl_include_path_from_line(line: &str) -> Option<String> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("//") {
        return None;
    }
    let rest = trimmed.strip_prefix("#include")?.trim_start();
    let rest = rest.strip_prefix('<')?;
    let (path, _) = rest.split_once('>')?;
    let path = path.trim();
    (!path.is_empty()).then(|| path.to_string())
}

fn shader_import_namespace(namespace: &str) -> Result<String, ShaderImportPathDerivationError> {
    let namespace = shader_project_namespace_from_name(namespace);
    if namespace.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyProjectNamespace);
    }
    if is_reserved_shader_import_namespace(&namespace) {
        return Err(ShaderImportPathDerivationError::ReservedNamespace { namespace });
    }
    Ok(namespace)
}

fn is_reserved_shader_import_namespace(namespace: &str) -> bool {
    namespace == SHADER_SELF_MODULE_NAMESPACE
        || namespace == "zircon"
        || namespace.starts_with("zr_")
}

fn normalized_shader_asset_path(
    asset_path: &str,
) -> Result<Vec<String>, ShaderImportPathDerivationError> {
    let without_label = asset_path
        .split_once('#')
        .map_or(asset_path, |(path, _)| path);
    let without_scheme = without_label
        .split_once("://")
        .map_or(without_label, |(_, path)| path);
    let path = without_scheme.replace('\\', "/");
    let segments = path
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments.is_empty() {
        Err(ShaderImportPathDerivationError::EmptyAssetPath)
    } else {
        Ok(segments)
    }
}

fn shader_module_path_segments(
    normalized_path: &[String],
) -> Result<(&[String], bool), ShaderImportPathDerivationError> {
    let root_index = normalized_path
        .iter()
        .position(|segment| segment.eq_ignore_ascii_case("shaders"))
        .ok_or_else(|| ShaderImportPathDerivationError::MissingShaderRoot {
            path: normalized_path.join("/"),
        })?;
    let module_segments = &normalized_path[root_index + 1..];
    if module_segments.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModulePath {
            path: normalized_path.join("/"),
        });
    }
    let folded = module_segments.len() >= 2
        && module_segments[module_segments.len() - 2].eq_ignore_ascii_case(
            strip_shader_asset_extension(&module_segments[module_segments.len() - 1]),
        );
    if folded {
        Ok((&module_segments[..module_segments.len() - 1], false))
    } else {
        Ok((module_segments, true))
    }
}

fn strip_shader_asset_extension(segment: &str) -> &str {
    segment
        .strip_suffix(".zshader")
        .or_else(|| segment.strip_suffix(".wgsl"))
        .unwrap_or(segment)
}

fn shader_module_segment(
    output: &mut String,
    path_segments: &[String],
    segment: &str,
) -> Result<(), ShaderImportPathDerivationError> {
    let start = output.len();
    let mut previous_underscore = false;
    for character in segment.chars() {
        if character.is_ascii_alphanumeric() {
            if output.len() == start && character.is_ascii_digit() {
                output.push('_');
            }
            output.push(character.to_ascii_lowercase());
            previous_underscore = false;
        } else if !previous_underscore && output.len() > start {
            output.push('_');
            previous_underscore = true;
        }
    }
    while output.len() > start && output.ends_with('_') {
        output.pop();
    }
    if output.len() == start {
        return Err(ShaderImportPathDerivationError::EmptyModuleSegment {
            path: path_segments.join("/"),
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "module_import/tests/direct_path_tests.rs"]
mod direct_path_tests;

fn terminal_directory_was_folded(path_segments: &[String]) -> bool {
    let Some(last_segment) = path_segments.last() else {
        return false;
    };
    let terminal = strip_shader_asset_extension(last_segment);
    path_segments
        .iter()
        .rev()
        .nth(1)
        .is_some_and(|directory| directory.eq_ignore_ascii_case(terminal))
}

#[cfg(test)]
#[path = "tests/module_import.rs"]
mod tests;
