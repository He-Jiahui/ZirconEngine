use std::path::{Component, Path, PathBuf};

use zircon_runtime_interface::ui::template::{
    UiResourceDependency, UiResourceDiagnostic, UiResourceDiagnosticSeverity,
    UiResourceFallbackMode,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiResourcePathResolver {
    res_root: Option<PathBuf>,
    asset_root: Option<PathBuf>,
    project_root: Option<PathBuf>,
}

impl UiResourcePathResolver {
    pub fn with_res_root(mut self, root: impl AsRef<Path>) -> Self {
        self.res_root = Some(root.as_ref().to_path_buf());
        self
    }

    pub fn with_asset_root(mut self, root: impl AsRef<Path>) -> Self {
        self.asset_root = Some(root.as_ref().to_path_buf());
        self
    }

    pub fn with_project_root(mut self, root: impl AsRef<Path>) -> Self {
        self.project_root = Some(root.as_ref().to_path_buf());
        self
    }

    fn root_for_uri(&self, uri: &str) -> Option<&Path> {
        if uri.starts_with("res://") {
            self.res_root.as_deref()
        } else if uri.starts_with("asset://") {
            self.asset_root.as_deref()
        } else if uri.starts_with("project://") {
            self.project_root.as_deref()
        } else {
            None
        }
    }
}

pub fn validate_resource_dependency_files(
    dependencies: &[UiResourceDependency],
    resolver: &UiResourcePathResolver,
) -> Vec<UiResourceDiagnostic> {
    let mut diagnostics = Vec::new();
    for dependency in dependencies {
        diagnostics.extend(validate_uri(
            resolver,
            &dependency.reference.uri,
            &dependency.path,
            UiResourceDiagnosticSeverity::Error,
        ));
        if let Some(fallback_uri) = dependency.reference.fallback.uri.as_deref() {
            let severity = match dependency.reference.fallback.mode {
                UiResourceFallbackMode::Placeholder => UiResourceDiagnosticSeverity::Error,
                UiResourceFallbackMode::None | UiResourceFallbackMode::Optional => {
                    UiResourceDiagnosticSeverity::Warning
                }
            };
            diagnostics.extend(validate_uri(
                resolver,
                fallback_uri,
                &format!("{}.fallback", dependency.path),
                severity,
            ));
        }
    }
    diagnostics.sort_by(|left, right| {
        (&left.path, &left.code, &left.message).cmp(&(&right.path, &right.code, &right.message))
    });
    diagnostics.dedup_by(|left, right| {
        left.path == right.path && left.code == right.code && left.message == right.message
    });
    diagnostics
}

fn validate_uri(
    resolver: &UiResourcePathResolver,
    uri: &str,
    path: &str,
    severity: UiResourceDiagnosticSeverity,
) -> Option<UiResourceDiagnostic> {
    let root = resolver.root_for_uri(uri)?;
    let Some(relative) = relative_path_from_resource_uri(uri) else {
        return Some(resource_diagnostic(
            "resource_uri_path_escape",
            severity,
            path,
            format!("resource uri {uri} cannot escape its resolver root"),
        ));
    };
    let resolved = root.join(relative);
    if resolved.exists() {
        return None;
    }
    Some(resource_diagnostic(
        "missing_resource_file",
        severity,
        path,
        format!(
            "resource uri {uri} resolved to missing file {}",
            resolved.display()
        ),
    ))
}

fn relative_path_from_resource_uri(uri: &str) -> Option<PathBuf> {
    let path = uri
        .split_once("://")
        .map(|(_, path)| path)
        .unwrap_or(uri)
        .split_once('#')
        .map(|(path, _)| path)
        .unwrap_or_else(|| uri.split_once("://").map(|(_, path)| path).unwrap_or(uri));
    let path = path.split_once('?').map(|(path, _)| path).unwrap_or(path);
    let mut normalized = PathBuf::new();
    for component in Path::new(path).components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir | Component::Prefix(_) | Component::RootDir => {
                return None;
            }
        }
    }
    Some(normalized)
}

fn resource_diagnostic(
    code: &str,
    severity: UiResourceDiagnosticSeverity,
    path: &str,
    message: String,
) -> UiResourceDiagnostic {
    UiResourceDiagnostic {
        code: code.to_string(),
        severity,
        message,
        path: path.to_string(),
    }
}
