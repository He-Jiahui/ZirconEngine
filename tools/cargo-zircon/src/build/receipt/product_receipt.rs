use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    canonical::{
        attestation_bytes, bytes_to_hex, canonical_receipt_sha256,
        canonical_receipt_sha256_from_receipt_matches, canonical_receipt_sha256_matches,
        decode_hex, decode_hex_into, INLINE_SIGNATURE_CAPACITY, PRODUCT_RECEIPT_KIND,
        PRODUCT_RECEIPT_SCHEMA_VERSION,
    },
    materialization, receipt_writer,
    validation::{
        normalize_and_validate, normalize_and_validate_after_batch_shape_with_validated_utc,
        validate_receipt_if_normalized, validate_required_text, ValidatedCreatedUtc,
    },
    BuildAction, ProducerIdentity, ProductReceiptDraft, ProductReceiptError, ProductReceiptSigner,
    ProductReceiptVerifier, ReceiptArtifact, ReceiptAttestation, TargetProfile, ToolchainSet,
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductReceipt {
    pub schema_version: u32,
    pub receipt_kind: String,
    pub receipt_id: String,
    pub created_utc: String,
    pub build_set_id: String,
    pub toolchain: ToolchainSet,
    pub target_profile: TargetProfile,
    pub action: BuildAction,
    pub producer: ProducerIdentity,
    pub build_products: Vec<ReceiptArtifact>,
    pub runtime_dependencies: Vec<ReceiptArtifact>,
    pub symbols: Vec<ReceiptArtifact>,
    pub sbom: Option<ReceiptArtifact>,
    pub attestation: ReceiptAttestation,
}

#[derive(Debug)]
pub struct VerifiedProductReceiptPublication(ProductReceipt);

pub(crate) struct FreshAttestation {
    payload: Vec<u8>,
    signature: Vec<u8>,
}

// This state exists only while an issued receipt is still owned by the issuance pipeline.
pub(crate) struct FreshProductReceipt {
    receipt: ProductReceipt,
    attestation: FreshAttestation,
}

pub(crate) struct ValidatedProductReceiptSigner<'a> {
    signer: &'a dyn ProductReceiptSigner,
    signer_id: &'a str,
    algorithm: &'a str,
}

impl VerifiedProductReceiptPublication {
    pub fn receipt_id(&self) -> &str {
        &self.0.receipt_id
    }

    pub fn write_new(&self, output_path: impl AsRef<Path>) -> Result<(), ProductReceiptError> {
        receipt_writer::write_new_after_verification(&self.0, output_path.as_ref())
    }
}

impl FreshAttestation {
    pub(crate) fn new(payload: Vec<u8>, signature: Vec<u8>) -> Self {
        Self { payload, signature }
    }

    pub(crate) fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub(crate) fn signature(&self) -> &[u8] {
        &self.signature
    }
}

impl FreshProductReceipt {
    fn into_inner(self) -> ProductReceipt {
        self.receipt
    }

    pub(crate) fn into_parts(self) -> (ProductReceipt, FreshAttestation) {
        (self.receipt, self.attestation)
    }

    pub(crate) fn verify_attestation(
        self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<VerifiedProductReceiptPublication, ProductReceiptError> {
        self.receipt.verify_attestation_payload_after_integrity(
            verifier,
            self.attestation.payload(),
            self.attestation.signature(),
        )?;
        Ok(VerifiedProductReceiptPublication(self.receipt))
    }
}

impl<'a> ValidatedProductReceiptSigner<'a> {
    pub(crate) fn new(signer: &'a dyn ProductReceiptSigner) -> Result<Self, ProductReceiptError> {
        let signer_id = signer.signer_id();
        let algorithm = signer.algorithm();
        validate_required_text("receipt signer id", signer_id)?;
        validate_required_text("receipt signature algorithm", algorithm)?;
        Ok(Self {
            signer,
            signer_id,
            algorithm,
        })
    }

    pub(crate) fn signer_id(&self) -> &str {
        self.signer_id
    }

    pub(crate) fn algorithm(&self) -> &str {
        self.algorithm
    }

    pub(crate) fn sign(
        &self,
        attestation_payload: &[u8],
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        self.signer.sign(attestation_payload)
    }
}

impl ProductReceipt {
    pub fn issue(
        draft: ProductReceiptDraft,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<Self, ProductReceiptError> {
        Self::issue_fresh(draft, created_utc, signer).map(FreshProductReceipt::into_inner)
    }

    pub fn issue_verified(
        draft: ProductReceiptDraft,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<VerifiedProductReceiptPublication, ProductReceiptError> {
        Self::issue_fresh(draft, created_utc, signer)?.verify_attestation(verifier)
    }

    pub(crate) fn issue_fresh(
        mut draft: ProductReceiptDraft,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<FreshProductReceipt, ProductReceiptError> {
        let created_utc = created_utc.into();
        normalize_and_validate(&mut draft, &created_utc)?;
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        Self::issue_normalized(draft, created_utc, &signer)
    }

    pub(crate) fn issue_fresh_after_batch_shape_with_signer(
        mut draft: ProductReceiptDraft,
        created_utc: impl Into<String>,
        validated_created_utc: &ValidatedCreatedUtc,
        signer: &ValidatedProductReceiptSigner<'_>,
    ) -> Result<FreshProductReceipt, ProductReceiptError> {
        let created_utc = created_utc.into();
        normalize_and_validate_after_batch_shape_with_validated_utc(
            &mut draft,
            validated_created_utc,
        )?;
        Self::issue_normalized(draft, created_utc, signer)
    }

    fn issue_normalized(
        draft: ProductReceiptDraft,
        created_utc: String,
        signer: &ValidatedProductReceiptSigner<'_>,
    ) -> Result<FreshProductReceipt, ProductReceiptError> {
        let receipt_id = canonical_receipt_sha256(&draft, &created_utc)?;
        let attestation_payload =
            attestation_bytes(&receipt_id, signer.signer_id(), signer.algorithm())?;
        let signature = signer.sign(&attestation_payload).map_err(|error| {
            ProductReceiptError::new(format!("product receipt signing failed: {error}"))
        })?;
        if signature.is_empty() {
            return Err(ProductReceiptError::new(
                "product receipt signer returned an empty signature",
            ));
        }

        let receipt = Self {
            schema_version: PRODUCT_RECEIPT_SCHEMA_VERSION,
            receipt_kind: PRODUCT_RECEIPT_KIND.to_string(),
            receipt_id,
            created_utc,
            build_set_id: draft.build_set_id,
            toolchain: draft.toolchain,
            target_profile: draft.target_profile,
            action: draft.action,
            producer: draft.producer,
            build_products: draft.build_products,
            runtime_dependencies: draft.runtime_dependencies,
            symbols: draft.symbols,
            sbom: draft.sbom,
            attestation: ReceiptAttestation {
                signer_id: signer.signer_id().to_string(),
                algorithm: signer.algorithm().to_string(),
                signature_hex: bytes_to_hex(&signature),
            },
        };
        Ok(FreshProductReceipt {
            receipt,
            attestation: FreshAttestation::new(attestation_payload, signature),
        })
    }

    pub fn verify_integrity(&self) -> Result<(), ProductReceiptError> {
        if self.schema_version != PRODUCT_RECEIPT_SCHEMA_VERSION
            || self.receipt_kind != PRODUCT_RECEIPT_KIND
        {
            return Err(ProductReceiptError::new(
                "product receipt has an unsupported schema or kind",
            ));
        }
        let normalized_draft = if validate_receipt_if_normalized(self)? {
            None
        } else {
            Some(self.normalized_draft()?)
        };
        validate_required_text("receipt attestation signer id", &self.attestation.signer_id)?;
        validate_required_text("receipt attestation algorithm", &self.attestation.algorithm)?;
        validate_required_text(
            "receipt attestation signature",
            &self.attestation.signature_hex,
        )?;

        let matches = match normalized_draft.as_ref() {
            Some(draft) => {
                canonical_receipt_sha256_matches(draft, &self.created_utc, &self.receipt_id)?
            }
            None => canonical_receipt_sha256_from_receipt_matches(self, &self.receipt_id)?,
        };
        if !matches {
            return Err(ProductReceiptError::new(
                "product receipt identity does not match its declared build closure",
            ));
        }
        Ok(())
    }

    pub fn verify_attestation(
        &self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        self.verify_attestation_after_integrity(verifier)
    }

    pub fn verify_attestation_and_materialization(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        artifact_root: impl AsRef<Path>,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        self.verify_attestation_after_integrity(verifier)?;
        materialization::verify(self, artifact_root.as_ref())
    }

    pub(crate) fn verify_attestation_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        let mut inline_signature = [0_u8; INLINE_SIGNATURE_CAPACITY];
        if let Some(signature_len) =
            decode_hex_into(&self.attestation.signature_hex, &mut inline_signature)?
        {
            return self.verify_attestation_bytes_after_integrity(
                verifier,
                &inline_signature[..signature_len],
            );
        }
        let signature = decode_hex(&self.attestation.signature_hex)?;
        self.verify_attestation_bytes_after_integrity(verifier, &signature)
    }

    pub(crate) fn verify_attestation_bytes_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        signature: &[u8],
    ) -> Result<(), ProductReceiptError> {
        let attestation_payload = attestation_bytes(
            &self.receipt_id,
            &self.attestation.signer_id,
            &self.attestation.algorithm,
        )?;
        self.verify_attestation_payload_after_integrity(verifier, &attestation_payload, signature)
    }

    pub(crate) fn verify_attestation_payload_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        attestation_payload: &[u8],
        signature: &[u8],
    ) -> Result<(), ProductReceiptError> {
        verifier
            .verify(
                &self.attestation.signer_id,
                &self.attestation.algorithm,
                attestation_payload,
                signature,
            )
            .map_err(|error| {
                ProductReceiptError::new(format!(
                    "product receipt attestation verification failed: {error}"
                ))
            })
    }

    pub fn verify_materialization(
        &self,
        artifact_root: impl AsRef<Path>,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        materialization::verify(self, artifact_root.as_ref())
    }

    pub fn write_new_verified(
        &self,
        output_path: impl AsRef<Path>,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        self.verify_attestation(verifier)?;
        receipt_writer::write_new_after_verification(self, output_path.as_ref())
    }

    fn draft(&self) -> ProductReceiptDraft {
        ProductReceiptDraft {
            build_set_id: self.build_set_id.clone(),
            toolchain: self.toolchain.clone(),
            target_profile: self.target_profile.clone(),
            action: self.action.clone(),
            producer: self.producer.clone(),
            build_products: self.build_products.clone(),
            runtime_dependencies: self.runtime_dependencies.clone(),
            symbols: self.symbols.clone(),
            sbom: self.sbom.clone(),
        }
    }

    fn normalized_draft(&self) -> Result<ProductReceiptDraft, ProductReceiptError> {
        let mut draft = self.draft();
        normalize_and_validate(&mut draft, &self.created_utc)?;
        Ok(draft)
    }

    #[cfg(test)]
    fn verify_integrity_with_normalized_preflight(&self) -> Result<(), ProductReceiptError> {
        use super::validation::is_normalized_receipt_for_benchmark;

        if self.schema_version != PRODUCT_RECEIPT_SCHEMA_VERSION
            || self.receipt_kind != PRODUCT_RECEIPT_KIND
        {
            return Err(ProductReceiptError::new(
                "product receipt has an unsupported schema or kind",
            ));
        }
        if !is_normalized_receipt_for_benchmark(self) {
            return self.verify_integrity_with_owned_normalization();
        }
        if !validate_receipt_if_normalized(self)? {
            return Err(ProductReceiptError::new(
                "normalized receipt preflight disagreed with validation",
            ));
        }
        validate_required_text("receipt attestation signer id", &self.attestation.signer_id)?;
        validate_required_text("receipt attestation algorithm", &self.attestation.algorithm)?;
        validate_required_text(
            "receipt attestation signature",
            &self.attestation.signature_hex,
        )?;
        let actual = canonical_receipt_sha256_from_receipt(self)?;
        if actual != self.receipt_id {
            return Err(ProductReceiptError::new(
                "product receipt identity does not match its declared build closure",
            ));
        }
        Ok(())
    }

    #[cfg(test)]
    fn verify_integrity_with_owned_normalization(&self) -> Result<(), ProductReceiptError> {
        let draft = self.normalized_draft()?;
        validate_required_text("receipt attestation signer id", &self.attestation.signer_id)?;
        validate_required_text("receipt attestation algorithm", &self.attestation.algorithm)?;
        validate_required_text(
            "receipt attestation signature",
            &self.attestation.signature_hex,
        )?;
        let actual = canonical_receipt_sha256(&draft, &self.created_utc)?;
        if actual != self.receipt_id {
            return Err(ProductReceiptError::new(
                "product receipt identity does not match its declared build closure",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;
