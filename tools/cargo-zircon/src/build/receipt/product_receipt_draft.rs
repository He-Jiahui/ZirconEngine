use super::{
    BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptSigner, ProductReceiptVerifier,
    ReceiptArtifact, TargetProfile, ToolchainSet, VerifiedProductReceiptPublication,
};
use serde::{Deserialize, Serialize};
use std::path::Path;

use super::{
    canonical::{serialized_sha256_matches, sha256_bytes_matches, sha256_serialized},
    receipt_writer, ProductReceiptError,
};

const HANDOFF_SERIALIZATION_ERROR: &str =
    "could not serialize product receipt draft handoff identity";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductReceiptDraft {
    pub build_set_id: String,
    pub toolchain: ToolchainSet,
    pub target_profile: TargetProfile,
    pub action: BuildAction,
    pub producer: ProducerIdentity,
    pub build_products: Vec<ReceiptArtifact>,
    pub runtime_dependencies: Vec<ReceiptArtifact>,
    pub symbols: Vec<ReceiptArtifact>,
    pub sbom: Option<ReceiptArtifact>,
}

pub struct VerifiedProductReceiptDraftHandoff(ProductReceiptDraft);

impl ProductReceiptDraft {
    pub fn handoff_sha256(&self) -> Result<String, ProductReceiptError> {
        sha256_serialized(self, HANDOFF_SERIALIZATION_ERROR)
    }

    pub fn verify_handoff_sha256(&self, expected: &str) -> Result<(), ProductReceiptError> {
        if !serialized_sha256_matches(self, expected, HANDOFF_SERIALIZATION_ERROR)? {
            return Err(ProductReceiptError::new(
                "product receipt draft does not match the build-owner handoff digest",
            ));
        }
        Ok(())
    }

    pub fn verify_handoff_sha256_owned(
        self,
        expected: &str,
    ) -> Result<VerifiedProductReceiptDraftHandoff, ProductReceiptError> {
        self.verify_handoff_sha256(expected)?;
        Ok(VerifiedProductReceiptDraftHandoff(self))
    }

    pub fn parse_and_verify_handoff_sha256(
        serialized: &[u8],
        expected: &str,
    ) -> Result<VerifiedProductReceiptDraftHandoff, ProductReceiptError> {
        let draft: Self = serde_json::from_slice(serialized).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not parse product receipt draft handoff: {error}"
            ))
        })?;
        if !sha256_bytes_matches(serialized, expected)
            && !serialized_sha256_matches(&draft, expected, HANDOFF_SERIALIZATION_ERROR)?
        {
            return Err(ProductReceiptError::new(
                "product receipt draft does not match the build-owner handoff digest",
            ));
        }
        Ok(VerifiedProductReceiptDraftHandoff(draft))
    }

    pub fn write_new_with_handoff_sha256(
        &self,
        output_path: impl AsRef<Path>,
    ) -> Result<String, ProductReceiptError> {
        receipt_writer::write_new_canonical_json_with_sha256(self, output_path.as_ref())
    }

    pub fn write_new(&self, output_path: impl AsRef<Path>) -> Result<(), ProductReceiptError> {
        self.write_new_with_handoff_sha256(output_path).map(drop)
    }
}

impl VerifiedProductReceiptDraftHandoff {
    pub fn issue(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<ProductReceipt, ProductReceiptError> {
        ProductReceipt::issue(self.0, created_utc, signer)
    }

    pub fn issue_verified(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<VerifiedProductReceiptPublication, ProductReceiptError> {
        ProductReceipt::issue_verified(self.0, created_utc, signer, verifier)
    }
}
