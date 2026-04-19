use crate::core::resource::RuntimeResourceState;

#[derive(Clone, Debug, Default)]
pub(super) struct ResourceRuntimeSlot {
    pub(super) ref_count: usize,
    pub(super) state: RuntimeResourceState,
}
