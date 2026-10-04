use serde::{Deserialize, Serialize};
use zircon_runtime_interface::project::ProjectGuid;

const CLOUD_BINDING_FORMAT_VERSION: u8 = 1;

/// Non-secret service identity. A different service or client cannot inherit a local link.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudBindingEnvironment {
    pub issuer: String,
    pub client_id: String,
    pub service_url: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudAccountScope {
    pub environment: CloudBindingEnvironment,
    pub subject: String,
}

/// Hub-owned link; the local manifest GUID and remote service UUID are distinct identities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CloudProjectBinding {
    format_version: u8,
    pub account: CloudAccountScope,
    pub local_path_key: String,
    pub local_project_guid: ProjectGuid,
    pub organization_id: String,
    pub project_id: String,
}

impl CloudProjectBinding {
    pub fn new(
        account: CloudAccountScope,
        local_path_key: String,
        local_project_guid: ProjectGuid,
        organization_id: String,
        project_id: String,
    ) -> Option<Self> {
        let binding = Self {
            format_version: CLOUD_BINDING_FORMAT_VERSION,
            account,
            local_path_key,
            local_project_guid,
            organization_id,
            project_id,
        };
        binding.is_valid().then_some(binding)
    }

    pub fn is_valid(&self) -> bool {
        self.format_version == CLOUD_BINDING_FORMAT_VERSION
            && !self.account.environment.issuer.is_empty()
            && !self.account.environment.client_id.is_empty()
            && !self.account.environment.service_url.is_empty()
            && !self.account.subject.is_empty()
            && !self.local_path_key.is_empty()
            && canonical_uuid(&self.organization_id)
            && canonical_uuid(&self.project_id)
    }
}

fn canonical_uuid(value: &str) -> bool {
    value.len() == 36
        && value.bytes().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) {
                byte == b'-'
            } else {
                byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
            }
        })
}

#[cfg(test)]
#[path = "tests/cloud_binding.rs"]
mod tests;
