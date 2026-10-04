//! 独立验签信任边界。
//! 签发后发布和外部 verify 都须重新核对签发者、算法、签名；签发者实现也可用于本地自检。

/// 发布和外部验收共用的信任决策；必须核对签发者、算法和原始签名字节。
pub trait ProductReceiptVerifier {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>>;
}
