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

pub fn wgsl_include_paths(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(wgsl_include_path_from_line)
        .collect()
}

pub fn strip_wgsl_include_directives(source: &str) -> String {
    source
        .lines()
        .filter(|line| wgsl_include_path_from_line(line).is_none())
        .collect::<Vec<_>>()
        .join("\n")
}

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
    let module_segments = shader_module_path_segments(&normalized_path)?;
    let mut segments = Vec::with_capacity(module_segments.len() + 1);
    segments.push(namespace);
    for segment in module_segments {
        segments.push(shader_module_segment(&normalized_path, segment)?);
    }
    Ok(ShaderImportPathDerivation {
        import_path: segments.join("::"),
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
) -> Result<Vec<String>, ShaderImportPathDerivationError> {
    let root_index = normalized_path
        .iter()
        .position(|segment| segment.eq_ignore_ascii_case("shaders"))
        .ok_or_else(|| ShaderImportPathDerivationError::MissingShaderRoot {
            path: normalized_path.join("/"),
        })?;
    let mut module_segments = normalized_path[root_index + 1..].to_vec();
    if module_segments.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModulePath {
            path: normalized_path.join("/"),
        });
    }
    if let Some(last) = module_segments.last_mut() {
        *last = strip_shader_asset_extension(last).to_string();
    }
    if module_segments.len() >= 2
        && module_segments[module_segments.len() - 2]
            .eq_ignore_ascii_case(&module_segments[module_segments.len() - 1])
    {
        module_segments.pop();
    }
    if module_segments.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModulePath {
            path: normalized_path.join("/"),
        });
    }
    Ok(module_segments)
}

fn strip_shader_asset_extension(segment: &str) -> &str {
    segment
        .strip_suffix(".zshader")
        .or_else(|| segment.strip_suffix(".wgsl"))
        .unwrap_or(segment)
}

fn shader_module_segment(
    path_segments: &[String],
    segment: String,
) -> Result<String, ShaderImportPathDerivationError> {
    let mut output = String::new();
    let mut previous_underscore = false;
    for character in segment.chars() {
        if character.is_ascii_alphanumeric() {
            output.push(character.to_ascii_lowercase());
            previous_underscore = false;
        } else if !previous_underscore && !output.is_empty() {
            output.push('_');
            previous_underscore = true;
        }
    }
    while output.ends_with('_') {
        output.pop();
    }
    if output.is_empty() {
        return Err(ShaderImportPathDerivationError::EmptyModuleSegment {
            path: path_segments.join("/"),
        });
    }
    if output
        .as_bytes()
        .first()
        .is_some_and(|first| first.is_ascii_digit())
    {
        output.insert(0, '_');
    }
    Ok(output)
}

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
mod tests {
    use super::{
        derive_shader_import_path, is_builtin_shader_module_token,
        shader_project_namespace_from_name, strip_wgsl_include_directives, wgsl_include_paths,
        ShaderImportPathDerivationError, GENERATED_MATERIAL_MODULE_IMPORT_PATH,
    };

    #[test]
    fn shader_module_imports_parse_line_directives_only() {
        let source =
            "// #include <ignored>\n#include <project::math>\nlet s = \"#include <ignored>\";";

        assert_eq!(
            wgsl_include_paths(source),
            vec!["project::math".to_string()]
        );
    }

    #[test]
    fn shader_module_imports_strip_directives_without_touching_comments() {
        let source = format!(
            "// #include <ignored>\n#include <{}>\nfn surface() {{}}",
            GENERATED_MATERIAL_MODULE_IMPORT_PATH
        );

        assert_eq!(
            strip_wgsl_include_directives(&source),
            "// #include <ignored>\nfn surface() {}"
        );
    }

    #[test]
    fn shader_module_imports_classify_builtin_tokens() {
        assert!(is_builtin_shader_module_token("zr_surface_types.wgsl"));
        assert!(is_builtin_shader_module_token("zr_shadow.wgsl"));
        assert!(!is_builtin_shader_module_token("project::shadow"));
    }

    #[test]
    fn render_shader_import_path_derivation_uses_project_namespace_and_asset_path() {
        let derived =
            derive_shader_import_path("My Shader Project", "res://shaders/cloth/common.zshader")
                .expect("shader path should derive import path");

        assert_eq!(derived.import_path, "my_shader_project::cloth::common");
        assert!(!derived.folded_terminal_directory);
        assert_eq!(
            shader_project_namespace_from_name(" 12 My Shader Project! "),
            "_12_my_shader_project"
        );
    }

    #[test]
    fn render_shader_import_path_derivation_folds_matching_directory_and_file_name() {
        let derived = derive_shader_import_path("MyProj", "assets/shaders/noise/noise.zshader")
            .expect("same terminal directory and file should fold");

        assert_eq!(derived.import_path, "myproj::noise");
        assert!(derived.folded_terminal_directory);
    }

    #[test]
    fn render_shader_import_path_derivation_rejects_reserved_project_namespace() {
        let error = derive_shader_import_path("self", "shaders/cloth/common.zshader")
            .expect_err("self namespace is reserved");

        assert_eq!(
            error,
            ShaderImportPathDerivationError::ReservedNamespace {
                namespace: "self".to_string()
            }
        );
    }
}
