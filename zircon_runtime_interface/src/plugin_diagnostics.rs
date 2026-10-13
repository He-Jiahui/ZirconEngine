//! Editor 插件注册的结构化诊断；目录能力校验生产记录，能力报告按错误级别判定成功。
//! 文本拥有自己的 `String`，可在目录借用结束后保存或序列化；构造不执行权限校验。
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 诊断严重度的 serde 枚举；当前能力报告仅将 Error 判为失败。
pub enum RegistrationDiagnosticSeverity {
    /// 用于说明性记录，不令能力报告失败。
    Info,
    /// 保留给调用方的警告，不令当前能力报告失败。
    Warning,
    /// 错误记录；EditorCapabilityReport::is_success 据此返回 false。
    Error,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
/// 拥有文本的注册诊断，不借用插件实例或原生报告内存。
/// # 参考
/// Editor 的 `core/plugin/catalog.rs::validate_capabilities` 与能力报告消费者。
pub struct RegistrationDiagnostic {
    /// 供消费者判定严重度的枚举。
    pub severity: RegistrationDiagnosticSeverity,
    /// 机器可识别的诊断代码；通用构造器不限制其格式。
    pub code: String,
    /// 诊断关联的插件身份文本；构造不验证包注册状态。
    pub plugin_id: String,
    /// 面向调用者的诊断内容，独立拥有内存。
    pub message: String,
}

impl RegistrationDiagnostic {
    /// 将严重度和三段文本收为拥有记录，返回值不依赖输入借用。
    /// # 参数
    /// `severity` 决定分类；`code`、`plugin_id`、`message` 通过 Into 转成 String，不校验格式。
    pub fn new(
        severity: RegistrationDiagnosticSeverity,
        code: impl Into<String>,
        plugin_id: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            severity,
            code: code.into(),
            plugin_id: plugin_id.into(),
            message: message.into(),
        }
    }

    /// 为目录未启用的能力生成 Error 记录，固定代码为 editor.capability.missing。
    /// # 参数与返回值
    /// 输入插件 ID 和 capability 转为 String；返回记录保留插件 ID，并把两者写进独立消息。
    /// 目录调用方负责先判断能力缺席，构造器本身不查询权限或修改注册表。
    pub fn missing_capability(plugin_id: impl Into<String>, capability: impl Into<String>) -> Self {
        let plugin_id = plugin_id.into();
        let capability = capability.into();
        // 预估只影响预留容量；饱和加法不改变最终消息，plugin_id 随后移动进记录而非克隆。
        let mut message = String::with_capacity(
            "editor plugin `` requires missing capability ``"
                .len()
                .saturating_add(plugin_id.len())
                .saturating_add(capability.len()),
        );
        message.push_str("editor plugin `");
        message.push_str(&plugin_id);
        message.push_str("` requires missing capability `");
        message.push_str(&capability);
        message.push('`');
        Self::new(
            RegistrationDiagnosticSeverity::Error,
            "editor.capability.missing",
            plugin_id,
            message,
        )
    }

    /// 仅比较 Error 枚举值；供 Editor 能力报告聚合判定，不检查诊断代码或文本。
    pub fn is_error(&self) -> bool {
        self.severity == RegistrationDiagnosticSeverity::Error
    }
}

// 私有测试从该模块的 super 访问诊断；性能用例只有显式运行 ignored 时才参与。
#[cfg(test)]
#[path = "tests/plugin_diagnostics.rs"]
mod tests;
