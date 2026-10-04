use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
/// 阶段描述变化需要经过的编译或 UI 工作类别；失效图据此生成影响摘要，测试锁定分类范围。
pub enum UiInvalidationStage {
    SourceParse,
    DocumentShape,
    ImportGraph,
    DescriptorRegistry,
    ComponentContract,
    ResourceDependency,
    SelectorMatch,
    StyleValue,
    Layout,
    Render,
    Interaction,
    Projection,
}
