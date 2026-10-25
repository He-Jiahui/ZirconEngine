use thiserror::Error;
use zircon_plugin_ai_runtime::behavior_tree::BehaviorTreeAssetError;

#[derive(Debug, Error)]
pub enum AiBehaviorTreeEditError {
    #[error("behavior-tree source could not be parsed: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("behavior-tree source could not be serialized: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error(transparent)]
    Runtime(#[from] BehaviorTreeAssetError),
    #[error("unsupported source field would be lost on save: {path}")]
    UnknownField { path: String },
    #[error("behavior node `{id}` does not exist")]
    MissingNode { id: String },
    #[error("behavior node `{id}` already exists")]
    DuplicateNode { id: String },
    #[error("child index {index} exceeds {count} slots in `{parent}`")]
    ChildIndex {
        parent: String,
        index: usize,
        count: usize,
    },
    #[error("the root `{id}` cannot be moved or deleted")]
    RootEdit { id: String },
    #[error("new node `{id}` must not already contain children")]
    NewNodeHasChildren { id: String },
    #[error("node `{id}` must be a single-child decorator")]
    NotDecorator { id: String },
    #[error("weighted random selector `{id}` needs an explicit weight-preserving topology edit")]
    WeightedTopology { id: String },
}
