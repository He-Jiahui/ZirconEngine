// 测试专用的静态贡献视图，只包含三个与运行时包清单逐项比较的集合。
use super::{StaticDependency, StaticEventCatalog, StaticModule};

pub(in crate::tests::manifest) struct StaticSoundContributions {
    pub(in crate::tests::manifest) dependencies: Vec<StaticDependency>,
    pub(in crate::tests::manifest) event_catalogs: Vec<StaticEventCatalog>,
    pub(in crate::tests::manifest) modules: Vec<StaticModule>,
}
