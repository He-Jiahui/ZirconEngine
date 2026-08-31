pub trait ProductReceiptSigner {
    fn signer_id(&self) -> &str;

    fn algorithm(&self) -> &str;

    fn sign(&self, attestation_payload: &[u8]) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
}
