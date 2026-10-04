//! typed facade 只负责把 ResourceManager 的 marker、状态、事件和 readiness 视图投影成资产 API。
//! 具体载荷仍由底层资源管理器持有，因而 handle 转换和事件过滤必须保持 kind 一致。

mod asset;
mod assets;
mod event;
mod handle;
mod impls;
mod load_state;
mod manager;
mod readiness;

pub use asset::Asset;
pub use assets::Assets;
pub(crate) use event::{typed_event_receiver, AssetEventPoll};
pub use event::{AssetEvent, AssetEventKind, AssetEventReceiver};
pub use handle::Handle;
pub use load_state::{
    AssetLoadState, AssetLoadStates, DependencyLoadState, RecursiveDependencyLoadState,
};
pub use readiness::{AssetDependencyReadiness, AssetReadinessNode, AssetReadinessReport};
