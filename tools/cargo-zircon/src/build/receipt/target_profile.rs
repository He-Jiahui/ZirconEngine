use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetProfile {
    pub target_triple: String,
    pub cargo_profile: String,
    pub codegen_flags_digest: String,
    pub cargo_graph_digest: String,
}
