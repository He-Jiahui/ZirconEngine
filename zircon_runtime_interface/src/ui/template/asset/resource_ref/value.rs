use serde::{Deserialize, Serialize};

use super::diagnostic::{UiResourceDiagnostic, UiResourceDiagnosticSeverity};
use super::fallback_policy::{UiResourceFallbackMode, UiResourceFallbackPolicy};
use super::resource_kind::UiResourceKind;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct UiResourceRef {
    pub kind: UiResourceKind,
    pub uri: String,
    #[serde(default)]
    pub fallback: UiResourceFallbackPolicy,
}

impl UiResourceRef {
    pub fn validate(&self, path: &str) -> Result<(), UiResourceDiagnostic> {
        let primary_uri = self.uri.trim();
        if primary_uri.is_empty() {
            return Err(error(
                "empty_resource_uri",
                "resource uri cannot be empty",
                path,
            ));
        }
        if !has_supported_scheme(primary_uri) {
            return Err(error(
                "unsupported_resource_scheme",
                "resource uri must use res://, asset://, or project://",
                path,
            ));
        }

        let Some(fallback_uri) = self.fallback.uri.as_deref() else {
            return match self.fallback.mode {
                UiResourceFallbackMode::Placeholder => Err(error(
                    "placeholder_fallback_missing_uri",
                    "placeholder fallback requires a fallback uri",
                    path,
                )),
                UiResourceFallbackMode::None | UiResourceFallbackMode::Optional => Ok(()),
            };
        };

        let fallback_uri = fallback_uri.trim();
        if fallback_uri.is_empty() {
            return match self.fallback.mode {
                UiResourceFallbackMode::Placeholder => Err(error(
                    "placeholder_fallback_missing_uri",
                    "placeholder fallback requires a fallback uri",
                    path,
                )),
                UiResourceFallbackMode::None | UiResourceFallbackMode::Optional => Err(error(
                    "unsupported_resource_scheme",
                    "fallback uri must use res://, asset://, or project://",
                    path,
                )),
            };
        }
        if !has_supported_scheme(fallback_uri) {
            return Err(error(
                "unsupported_resource_scheme",
                "fallback uri must use res://, asset://, or project://",
                path,
            ));
        }
        if fallback_uri == primary_uri {
            return Err(error(
                "placeholder_fallback_self_reference",
                "fallback uri cannot reference the primary resource uri",
                path,
            ));
        }
        if self.fallback.mode == UiResourceFallbackMode::Placeholder {
            let fallback_kind = UiResourceKind::infer_from_path_and_uri("", fallback_uri);
            if fallback_kind != self.kind {
                return Err(error(
                    "placeholder_fallback_kind_mismatch",
                    "placeholder fallback kind must match the primary resource kind",
                    path,
                ));
            }
        }

        Ok(())
    }
}

fn has_supported_scheme(uri: &str) -> bool {
    uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://")
}

fn error(code: &str, message: &str, path: &str) -> UiResourceDiagnostic {
    UiResourceDiagnostic {
        code: code.to_string(),
        severity: UiResourceDiagnosticSeverity::Error,
        message: message.to_string(),
        path: path.to_string(),
    }
}
