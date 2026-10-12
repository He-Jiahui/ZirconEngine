//! typed facade 只负责把 ResourceManager 的 marker、状态、事件和 readiness 视图投影成资产 API。
//! 具体载荷仍由底层资源管理器持有，因而 handle 转换和事件过滤必须保持 kind 一致。

#[path = "asset.rs"]
mod asset;
#[path = "assets.rs"]
mod assets;
#[path = "event.rs"]
mod event;
#[path = "handle.rs"]
mod handle;
#[path = "impls.rs"]
mod impls;
#[path = "load_state.rs"]
mod load_state;
#[path = "manager.rs"]
mod manager;
#[path = "readiness.rs"]
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
