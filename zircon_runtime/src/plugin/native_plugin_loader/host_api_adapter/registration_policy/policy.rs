/// Host-owned authority inputs for a V4 native registration scope.
///
/// Component IDs are read from the runtime registry at each registration so components declared
/// earlier in the same entry callback become immediately usable. Resource IDs and capability
/// grants are host policy, never plugin-provided inputs.
#[derive(Clone, Debug, Default)]
pub struct NativeHostApiV4RegistrationPolicy {
    pub(in super::super) granted_capabilities: Vec<String>,
    pub(in super::super) known_resource_ids: Vec<String>,
}

impl NativeHostApiV4RegistrationPolicy {
    pub fn new<C, R, CS, RS>(granted_capabilities: C, known_resource_ids: R) -> Self
    where
        C: IntoIterator<Item = CS>,
        R: IntoIterator<Item = RS>,
        CS: Into<String>,
        RS: Into<String>,
    {
        let mut granted_capabilities = granted_capabilities
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        let mut known_resource_ids = known_resource_ids
            .into_iter()
            .map(Into::into)
            .collect::<Vec<String>>();
        granted_capabilities.sort();
        granted_capabilities.dedup();
        known_resource_ids.sort();
        known_resource_ids.dedup();
        Self {
            granted_capabilities,
            known_resource_ids,
        }
    }
}
