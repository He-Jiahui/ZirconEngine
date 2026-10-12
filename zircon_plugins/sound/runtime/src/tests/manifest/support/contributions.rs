// 静态清单贡献的测试投影边界；依赖、事件目录和 runtime 模块分别与 package_manifest 对照。
#[path = "../../support/contributions/dependencies.rs"]
mod dependencies;
#[path = "contributions/entry.rs"]
mod entry;
#[path = "../../support/contributions/event_catalogs.rs"]
mod event_catalogs;
#[path = "contributions/modules.rs"]
mod modules;
#[path = "contributions/types.rs"]
mod types;

pub(super) use entry::static_sound_contributions;
pub(in crate::tests::manifest) use types::StaticSoundContributions;
use types::{StaticDependency, StaticEventCatalog, StaticModule};
