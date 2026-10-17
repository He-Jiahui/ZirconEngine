//! 组件适配器把写入结果和投影请求交给宿主；宿主按刷新标志安排展示更新。

use serde::{Deserialize, Serialize};

use super::UiComponentProjectionPatch;
use crate::ui::component::UiValidationState;

/// Reports adapter mutation status and optional projection updates to the host UI.
///
/// # 使用与生命周期
/// 在适配器返回后交给宿主处理；标志、可选诊断和局部投影都随结果移交所有权。
/// 三个布尔位可独立设置，构造结果不会执行刷新、保存或事务提交。
/// 工厂的默认值不代表线格式中的全部字段都能省略；集合的省略规则由字段属性决定。
///
/// # 参考
/// [`Self::changed`]、[`Self::unchanged`]、[`UiComponentProjectionPatch`]。
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiComponentAdapterResult {
    /// 目标数据是否已发生变化；宿主也可独立请求投影刷新。
    pub changed: bool,
    /// 宿主据此刷新展示投影，即使没有局部 patch 也可能需要刷新。
    pub refresh_projection: bool,
    /// 目标资源是否应标记为待保存；已处理的命令不一定修改该资源。
    pub dirty: bool,
    /// 供宿主关联操作的自有标识；此值不创建或提交事务。
    pub transaction_id: Option<String>,
    /// 宿主提供的变更来源标签，不携带对源数据的借用。
    pub mutation_source: Option<String>,
    /// 可展示的结果文字；消费端决定其显示时机。
    pub status_text: Option<String>,
    /// 可选校验结果快照；不在此结构中重新校验输入。
    pub validation: Option<UiValidationState>,
    /// 按加入顺序交付的自有投影列表；空列表仍可与刷新标志组合。
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub patches: Vec<UiComponentProjectionPatch>,
}

impl UiComponentAdapterResult {
    /// 创建三个标志均关闭、可选信息为空且没有投影的独立结果。
    /// 返回值尚未触发宿主动作，后续可通过值接收构建器补充字段。
    pub fn unchanged() -> Self {
        Self::default()
    }

    /// 创建变更、刷新和待保存标志均开启的结果，其余字段保留默认值。
    /// 返回的只是报告值；实际数据写入及投影应用由宿主完成。
    pub fn changed() -> Self {
        Self {
            changed: true,
            refresh_projection: true,
            dirty: true,
            ..Self::default()
        }
    }

    /// 消费当前结果并仅替换待保存位，返回更新后的自有结果；其他标志和投影保持原值。
    pub fn dirty(mut self, dirty: bool) -> Self {
        self.dirty = dirty;
        self
    }

    /// 将参数转换为自有事务标识，替换旧标识并返回结果；不检查标识内容或操作事务。
    pub fn with_transaction(mut self, transaction_id: impl Into<String>) -> Self {
        self.transaction_id = Some(transaction_id.into());
        self
    }

    /// 将参数转换为自有来源标签，替换旧标签并返回结果；不修改变更标志。
    pub fn with_mutation_source(mut self, mutation_source: impl Into<String>) -> Self {
        self.mutation_source = Some(mutation_source.into());
        self
    }

    /// 将参数转换为自有状态文字，替换旧文字并返回结果；显示由消费端决定。
    pub fn with_status(mut self, status_text: impl Into<String>) -> Self {
        self.status_text = Some(status_text.into());
        self
    }

    /// 接管一个局部投影并追加到列表尾部，返回结果；不合并同控件项，也不修改三个标志。
    pub fn with_patch(mut self, patch: UiComponentProjectionPatch) -> Self {
        self.patches.push(patch);
        self
    }
}
