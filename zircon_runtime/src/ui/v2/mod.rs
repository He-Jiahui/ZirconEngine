//! 把版本化 UI 资源连接到保留式表面：加载、导入展开、样式解析与树构建在创建阶段完成。
//! 宿主帧更新复用已编译结构；本模块不承担文件监听或每帧资源重读。

mod cache;
mod compiler;
mod component_instancer;
mod component_reference;
mod file_cache;
mod loader;
mod style;
mod surface_builder;
mod surface_tree;

pub(crate) use cache::source_path_identity_for_path;
pub use cache::{UiV2PrototypeStore, UiV2PrototypeStoreBuilder};
pub use compiler::UiV2DocumentCompiler;
pub use component_instancer::UiV2ComponentInstancer;
pub use file_cache::{
    UiV2PrototypeStoreFileCache, UiV2PrototypeStoreLoadOutcome, UiV2SourceFileReceipt,
    UiV2UnresolvedSourceImport,
};
pub use loader::{UiV2AssetLoader, UiZuiAssetLoader};
pub(crate) use style::UiV2RuntimeStyleIndex;
pub use style::UiV2StyleResolver;
pub use surface_builder::UiV2SurfaceBuilder;
pub use zircon_runtime_interface::ui::v2::UiV2CompiledDocument;
