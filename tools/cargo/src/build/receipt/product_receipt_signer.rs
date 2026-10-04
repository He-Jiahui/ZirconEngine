//! 签发者的最小能力接口。
//! 收据规范负载生成后，调用者提供签发者 ID、算法与签名字节；具体密钥保管由实现负责。

/// 签发者提供稳定身份、算法与规范负载签名；收据本体和验收策略由调用方负责。
pub trait ProductReceiptSigner {
    fn signer_id(&self) -> &str;

    fn algorithm(&self) -> &str;

    fn sign(&self, attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
