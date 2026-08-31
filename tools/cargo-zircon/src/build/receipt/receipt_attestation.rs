use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptAttestation {
    pub signer_id: String,
    pub algorithm: String,
    pub signature_hex: String,
}
