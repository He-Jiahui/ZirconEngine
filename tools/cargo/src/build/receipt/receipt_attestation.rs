//! 签发者对规范收据身份的证明。
//! 签名覆盖规范负载而非文件路径；公开核验必须结合信任注册表与实际物化结果。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptAttestation {
    pub signer_id: String,
    pub algorithm: String,
    pub signature_hex: String,
}
