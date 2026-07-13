use super::ParamLayout;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledCallSite {
    pub type_slot: u32,
    pub member_slot: u32,
    pub layout: ParamLayout,
}

impl CompiledCallSite {
    pub(crate) fn new(type_slot: u32, member_slot: u32, layout: ParamLayout) -> Self {
        Self {
            type_slot,
            member_slot,
            layout,
        }
    }
}
