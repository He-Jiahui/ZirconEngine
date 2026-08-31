pub trait ProductReceiptVerifier {
    fn verify(
        &self,
        signer_id: &str,
        algorithm: &str,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>>;
}
