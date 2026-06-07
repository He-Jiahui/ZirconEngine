use super::{StaticDependency, StaticEventCatalog, StaticModule};

pub(in crate::tests::manifest) struct StaticSoundContributions {
    pub(in crate::tests::manifest) dependencies: Vec<StaticDependency>,
    pub(in crate::tests::manifest) event_catalogs: Vec<StaticEventCatalog>,
    pub(in crate::tests::manifest) modules: Vec<StaticModule>,
}
