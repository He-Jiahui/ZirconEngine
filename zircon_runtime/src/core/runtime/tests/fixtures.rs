use super::super::PluginContext;

#[derive(Debug)]
pub(super) struct TestDriver {
    pub(super) order: usize,
}

#[derive(Debug)]
pub(super) struct TestManager;

#[derive(Debug)]
pub(super) struct RecordedPlugin(pub(super) PluginContext);
