use serde::{Deserialize, Serialize};

use super::diagnostic::{UiResourceDiagnostic, UiResourceDiagnosticSeverity};
use super::fallback_policy::{UiResourceFallbackMode, UiResourceFallbackPolicy};
use super::resource_kind::UiResourceKind;

/// 资产声明的资源引用；收集器检查 URI 形状，解析器再查找资源句柄。
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// 资源解析缓存按完整声明区分身份；验证不会把修整后的 URI 写回，也不自动合并声明。
pub struct UiResourceRef {
    /// 调用方声明的粗种类；不会根据主 URI 自动重写。
    pub kind: UiResourceKind,
    /// 拥有的原始 URI；验证借用修整后的视图，保留此原值。
    pub uri: String,
    #[serde(default)]
    /// 缺失时采用默认策略；派生 Deserialize 不自动调用声明验证。
    pub fallback: UiResourceFallbackPolicy,
}

impl UiResourceRef {
    /// 只检查 URI scheme、占位回退要求及回退种类，不访问实际资源存储。
    ///
    /// # 参数与返回
    /// `path` 仅作为错误位置标签；成功表示通过此处的声明检查，失败返回拥有代码、消息和位置的诊断。
    /// # 调用与生命周期
    /// 收集器在插入声明前显式调用；本方法只借用载荷，不修改 URI，也不缓存或解析资源句柄。
    /// # 约束与参考
    /// Scheme 按大小写敏感的字面前缀检查，不验证其后的定位串或资源存在性；主资源种类不会重新推断。
    /// 回退分类使用 [种类推断](UiResourceKind::infer_from_path_and_uri)，仅占位模式要求回退种类相等。
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

        // 缺失和 Some 空串分开处理，模式决定缺失是否可接受。
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
        // 已声明的回退 URI 在所有模式下都检查前缀和自引用，即使解析器不会消费它。
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
        // 只有占位回退检查推断种类；这不校验主 URI 后缀是否符合声明种类。
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

// 仅识别大小写敏感的前缀；不验证前缀后的定位串，也不检查文件存在性。
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
