use super::ParamLayout;

/// Dense reflected field address resolved once while a VM package is loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompiledCallSite {
    token: u64,
    /// Dense type slot in the package-local public reflection table.
    pub type_slot: u32,
    /// Dense field slot in the reflected type registration.
    pub member_slot: u32,
    /// Expected reflected value layout and write permission.
    pub layout: ParamLayout,
}

impl CompiledCallSite {
    pub(crate) fn new(token: u64, type_slot: u32, member_slot: u32, layout: ParamLayout) -> Self {
        Self {
            token,
            type_slot,
            member_slot,
            layout,
        }
    }

    /// Returns the non-reusable opaque integer passed through the VM ABI.
    pub const fn token(&self) -> u64 {
        self.token
    }
}
