use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
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
