use serde::{Deserialize, Serialize};

use super::{ProjectEngineCompatibilityDisposition, ProjectEngineVersion};

/// Immutable engine-version decision that an admission policy must consume before activation.
/// 项目预检形成的版本判定快照；Editor 在激活前消费其兼容结论。
///
/// # 使用时机与生命周期
/// 由 [`crate::project::assess_project_engine_compatibility`] 创建，并随预检结果传给宿主准入策略；此值记录当次比较使用的要求、引擎版本和结论。
///
/// # 字段
/// - `requirement`：原始清单要求；没有声明时为 `None`。
/// - `running_engine`：本次预检选定的引擎版本。
/// - `disposition`：该引擎对要求的分类，见 [`ProjectEngineCompatibilityDisposition`]。
///
/// # 约束
/// [`is_compatible`](Self::is_compatible) 只读取保存的判定，不会重新解析要求或查询另一引擎版本。该值支持反序列化，因此本身不证明输入来源可信或项目已获准激活；Editor 仍须按宿主流程验证来源并执行其他准入检查。
///
/// # 参考
/// Editor 的 `zircon_editor/src/core/project/authority/preflight.rs` 中
/// `validate_launch_receipt` 在激活前消费评估结果；
/// `zircon_runtime_interface/src/project/tests/engine_compatibility.rs` 验证其保留的输入身份。
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectEngineCompatibility {
    requirement: Option<String>,
    running_engine: ProjectEngineVersion,
    disposition: ProjectEngineCompatibilityDisposition,
}

impl ProjectEngineCompatibility {
    /// 组装已计算的兼容性快照；生产路径由评估函数传入，单元测试用它表达期望值。
    ///
    /// # Parameters
    /// - `requirement`：需要保留在结果中的原始要求文本，或表示未声明要求的 `None`。
    /// - `running_engine`：形成判定时使用的引擎版本。
    /// - `disposition`：评估器得到的兼容性分类。
    pub(crate) fn new(
        requirement: Option<String>,
        running_engine: ProjectEngineVersion,
        disposition: ProjectEngineCompatibilityDisposition,
    ) -> Self {
        Self {
            requirement,
            running_engine,
            disposition,
        }
    }

    /// 返回预检时保存的原始要求文本；没有声明要求时返回 `None`。
    pub fn requirement(&self) -> Option<&str> {
        self.requirement.as_deref()
    }

    /// 返回形成此值时比较的引擎版本，不读取宿主当前版本。
    pub fn running_engine(&self) -> &ProjectEngineVersion {
        &self.running_engine
    }

    /// 返回评估时保存的方向或兼容分类。
    pub const fn disposition(&self) -> ProjectEngineCompatibilityDisposition {
        self.disposition
    }

    /// 返回快照中的预检结论，不重新解析要求或读取当前引擎版本。
    pub const fn is_compatible(&self) -> bool {
        self.disposition.is_compatible()
    }
}
