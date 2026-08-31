use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    canonical::{
        batch_attestation_bytes, bytes_to_hex, canonical_build_action_key,
        canonical_receipt_batch_sha256, canonical_receipt_batch_sha256_matches, decode_hex,
        decode_hex_into, INLINE_SIGNATURE_CAPACITY, PRODUCT_RECEIPT_BATCH_KIND,
        PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
    },
    materialization, receipt_writer, validate_created_utc_for_batch, FreshAttestation,
    FreshProductReceipt, ProductReceipt, ProductReceiptDraft, ProductReceiptError,
    ProductReceiptSigner, ProductReceiptVerifier, ReceiptAttestation, ValidatedCreatedUtc,
    ValidatedProductReceiptSigner,
};

const PRODUCT_RECEIPT_BATCH_LIMIT: usize = 16;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductReceiptBatch {
    pub schema_version: u32,
    pub receipt_batch_kind: String,
    pub batch_id: String,
    pub build_set_id: String,
    pub receipts: Vec<ProductReceipt>,
    pub attestation: ReceiptAttestation,
}

#[derive(Debug)]
pub struct VerifiedProductReceiptBatchPublication(ProductReceiptBatch);

// This state is constructible only while an issued batch is still owned by the build pipeline.
pub(crate) struct FreshProductReceiptBatch {
    batch: ProductReceiptBatch,
    batch_attestation: FreshAttestation,
    receipt_attestations: Option<Vec<FreshAttestation>>,
}

impl VerifiedProductReceiptBatchPublication {
    pub fn batch_id(&self) -> &str {
        &self.0.batch_id
    }

    pub fn write_new(&self, output_path: impl AsRef<Path>) -> Result<(), ProductReceiptError> {
        receipt_writer::write_new_json(&self.0, output_path.as_ref())
    }
}

impl FreshProductReceiptBatch {
    pub(crate) fn into_inner(self) -> ProductReceiptBatch {
        self.batch
    }

    pub(crate) fn verify_attestations(
        self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<VerifiedProductReceiptBatchPublication, ProductReceiptError> {
        self.batch
            .verify_batch_attestation_payload_after_integrity(
                verifier,
                self.batch_attestation.payload(),
                self.batch_attestation.signature(),
            )?;
        if let Some(attestations) = &self.receipt_attestations {
            if attestations.len() != self.batch.receipts.len() {
                return Err(ProductReceiptError::new(
                    "fresh product receipt attestations do not match the receipt batch",
                ));
            }
            for (receipt, attestation) in self.batch.receipts.iter().zip(attestations) {
                receipt.verify_attestation_payload_after_integrity(
                    verifier,
                    attestation.payload(),
                    attestation.signature(),
                )?;
            }
        } else {
            for receipt in &self.batch.receipts {
                receipt.verify_attestation_after_integrity(verifier)?;
            }
        }
        Ok(VerifiedProductReceiptBatchPublication(self.batch))
    }
}

impl ProductReceiptBatch {
    pub fn issue(
        build_set_id: String,
        receipts: Vec<ProductReceipt>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<Self, ProductReceiptError> {
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        let batch = Self::unsigned(build_set_id, receipts, &signer)?;
        batch.validate_closure_shape()?;
        for receipt in &batch.receipts {
            receipt.verify_integrity()?;
        }
        batch
            .sign_after_integrity(&signer, None)
            .map(FreshProductReceiptBatch::into_inner)
    }

    pub(crate) fn issue_after_validated_closure(
        build_set_id: String,
        receipts: Vec<ProductReceipt>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<Self, ProductReceiptError> {
        Self::issue_fresh_after_validated_closure(build_set_id, receipts, signer)
            .map(FreshProductReceiptBatch::into_inner)
    }

    pub(crate) fn issue_fresh_after_validated_closure(
        build_set_id: String,
        receipts: Vec<ProductReceipt>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<FreshProductReceiptBatch, ProductReceiptError> {
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        Self::unsigned(build_set_id, receipts, &signer)?.sign_after_integrity(&signer, None)
    }

    pub(crate) fn issue_fresh_from_batch_shape_drafts(
        build_set_id: String,
        drafts: Vec<ProductReceiptDraft>,
        mut created_utc: String,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<FreshProductReceiptBatch, ProductReceiptError> {
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        let validated_created_utc: ValidatedCreatedUtc =
            validate_created_utc_for_batch(&created_utc)?;
        let draft_count = drafts.len();
        let mut drafts = drafts.into_iter().peekable();
        let mut receipts = Vec::with_capacity(draft_count);
        let mut receipt_attestations = Vec::with_capacity(draft_count);
        while let Some(draft) = drafts.next() {
            let draft_created_utc = if drafts.peek().is_some() {
                created_utc.clone()
            } else {
                std::mem::take(&mut created_utc)
            };
            let fresh_receipt = ProductReceipt::issue_fresh_after_batch_shape_with_signer(
                draft,
                draft_created_utc,
                &validated_created_utc,
                &signer,
            )?;
            let (receipt, attestation) = fresh_receipt.into_parts();
            receipts.push(receipt);
            receipt_attestations.push(attestation);
        }
        Self::unsigned(build_set_id, receipts, &signer)?
            .sign_after_integrity(&signer, Some(receipt_attestations))
    }

    pub(crate) fn issue_fresh_after_validated_receipts(
        build_set_id: String,
        fresh_receipts: Vec<FreshProductReceipt>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<FreshProductReceiptBatch, ProductReceiptError> {
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        let mut receipts = Vec::with_capacity(fresh_receipts.len());
        let mut receipt_attestations = Vec::with_capacity(fresh_receipts.len());
        for fresh_receipt in fresh_receipts {
            let (receipt, attestation) = fresh_receipt.into_parts();
            receipts.push(receipt);
            receipt_attestations.push(attestation);
        }
        Self::unsigned(build_set_id, receipts, &signer)?
            .sign_after_integrity(&signer, Some(receipt_attestations))
    }

    #[cfg(test)]
    fn issue_after_receipt_integrity_with_shape_validation(
        build_set_id: String,
        receipts: Vec<ProductReceipt>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<Self, ProductReceiptError> {
        let signer = ValidatedProductReceiptSigner::new(signer)?;
        let batch = Self::unsigned(build_set_id, receipts, &signer)?;
        batch.validate_closure_shape()?;
        batch
            .sign_after_integrity(&signer, None)
            .map(FreshProductReceiptBatch::into_inner)
    }

    fn unsigned(
        build_set_id: String,
        receipts: Vec<ProductReceipt>,
        signer: &ValidatedProductReceiptSigner<'_>,
    ) -> Result<Self, ProductReceiptError> {
        Ok(Self {
            schema_version: PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION,
            receipt_batch_kind: PRODUCT_RECEIPT_BATCH_KIND.to_string(),
            batch_id: String::new(),
            build_set_id,
            receipts,
            attestation: ReceiptAttestation {
                signer_id: signer.signer_id().to_string(),
                algorithm: signer.algorithm().to_string(),
                signature_hex: String::new(),
            },
        })
    }

    fn sign_after_integrity(
        mut self,
        signer: &ValidatedProductReceiptSigner<'_>,
        receipt_attestations: Option<Vec<FreshAttestation>>,
    ) -> Result<FreshProductReceiptBatch, ProductReceiptError> {
        self.batch_id = canonical_receipt_batch_sha256(&self.build_set_id, &self.receipts)?;
        let payload =
            batch_attestation_bytes(&self.batch_id, signer.signer_id(), signer.algorithm())?;
        let signature = signer.sign(&payload).map_err(|error| {
            ProductReceiptError::new(format!("product receipt batch signing failed: {error}"))
        })?;
        if signature.is_empty() {
            return Err(ProductReceiptError::new(
                "product receipt batch signer returned an empty signature",
            ));
        }
        self.attestation.signature_hex = bytes_to_hex(&signature);
        Ok(FreshProductReceiptBatch {
            batch: self,
            batch_attestation: FreshAttestation::new(payload, signature),
            receipt_attestations,
        })
    }

    pub fn verify_integrity(&self) -> Result<(), ProductReceiptError> {
        self.validate_closure_shape()?;
        validate_required_text("receipt batch id", &self.batch_id)?;
        validate_required_text(
            "receipt batch attestation signer id",
            &self.attestation.signer_id,
        )?;
        validate_required_text(
            "receipt batch attestation algorithm",
            &self.attestation.algorithm,
        )?;
        validate_required_text(
            "receipt batch attestation signature",
            &self.attestation.signature_hex,
        )?;
        for receipt in &self.receipts {
            receipt.verify_integrity()?;
        }
        if !canonical_receipt_batch_sha256_matches(
            &self.build_set_id,
            &self.receipts,
            &self.batch_id,
        )? {
            return Err(ProductReceiptError::new(
                "product receipt batch identity does not match its declared receipt set",
            ));
        }
        Ok(())
    }

    pub fn verify_attestations(
        &self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        self.verify_batch_attestation_after_integrity(verifier)?;
        for receipt in &self.receipts {
            receipt.verify_attestation_after_integrity(verifier)?;
        }
        Ok(())
    }

    pub fn verify_materialization(
        &self,
        artifact_root: impl AsRef<Path>,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        materialization::verify_receipts(&self.receipts, artifact_root.as_ref())
    }

    pub fn verify_attestations_and_materialization(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        artifact_root: impl AsRef<Path>,
    ) -> Result<(), ProductReceiptError> {
        self.verify_integrity()?;
        self.verify_batch_attestation_after_integrity(verifier)?;
        for receipt in &self.receipts {
            receipt.verify_attestation_after_integrity(verifier)?;
        }
        materialization::verify_receipts(&self.receipts, artifact_root.as_ref())
    }

    pub fn write_new_verified(
        &self,
        output_path: impl AsRef<Path>,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        self.verify_attestations(verifier)?;
        receipt_writer::write_new_json(self, output_path.as_ref())
    }

    fn verify_batch_attestation_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<(), ProductReceiptError> {
        let mut inline_signature = [0_u8; INLINE_SIGNATURE_CAPACITY];
        if let Some(signature_len) =
            decode_hex_into(&self.attestation.signature_hex, &mut inline_signature)?
        {
            return self.verify_batch_attestation_bytes_after_integrity(
                verifier,
                &inline_signature[..signature_len],
            );
        }
        let signature = decode_hex(&self.attestation.signature_hex)?;
        self.verify_batch_attestation_bytes_after_integrity(verifier, &signature)
    }

    fn verify_batch_attestation_bytes_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        signature: &[u8],
    ) -> Result<(), ProductReceiptError> {
        let payload = batch_attestation_bytes(
            &self.batch_id,
            &self.attestation.signer_id,
            &self.attestation.algorithm,
        )?;
        self.verify_batch_attestation_payload_after_integrity(verifier, &payload, signature)
    }

    fn verify_batch_attestation_payload_after_integrity(
        &self,
        verifier: &dyn ProductReceiptVerifier,
        payload: &[u8],
        signature: &[u8],
    ) -> Result<(), ProductReceiptError> {
        verifier
            .verify(
                &self.attestation.signer_id,
                &self.attestation.algorithm,
                payload,
                signature,
            )
            .map_err(|error| {
                ProductReceiptError::new(format!(
                    "product receipt batch attestation verification failed: {error}"
                ))
            })
    }

    fn validate_closure_shape(&self) -> Result<(), ProductReceiptError> {
        if self.schema_version != PRODUCT_RECEIPT_BATCH_SCHEMA_VERSION
            || self.receipt_batch_kind != PRODUCT_RECEIPT_BATCH_KIND
        {
            return Err(ProductReceiptError::new(
                "product receipt batch has an unsupported schema or kind",
            ));
        }
        if self.receipts.len() < 2 || self.receipts.len() > PRODUCT_RECEIPT_BATCH_LIMIT {
            return Err(ProductReceiptError::new(format!(
                "product receipt batch requires between 2 and {PRODUCT_RECEIPT_BATCH_LIMIT} receipts"
            )));
        }
        let mut receipt_ids = HashSet::with_capacity(self.receipts.len());
        let mut actions = HashSet::with_capacity(self.receipts.len());
        let mut operation_ids = HashSet::with_capacity(self.receipts.len());
        let artifact_count = self.receipts.iter().fold(0_usize, |count, receipt| {
            count
                .saturating_add(receipt.build_products.len())
                .saturating_add(receipt.runtime_dependencies.len())
                .saturating_add(receipt.symbols.len())
                .saturating_add(usize::from(receipt.sbom.is_some()))
        });
        let mut artifact_names = HashSet::with_capacity(artifact_count);
        let mut artifact_paths = HashSet::with_capacity(artifact_count);
        for receipt in &self.receipts {
            if receipt.build_set_id != self.build_set_id {
                return Err(ProductReceiptError::new(
                    "product receipt batch must bind every receipt to its declared BuildSet",
                ));
            }
            if !receipt_ids.insert(receipt.receipt_id.as_str()) {
                return Err(ProductReceiptError::new(
                    "product receipt batch contains a duplicate receipt id",
                ));
            }
            if !actions.insert(canonical_build_action_key(&receipt.action)) {
                return Err(ProductReceiptError::new(
                    "product receipt batch contains a duplicate canonical build action",
                ));
            }
            if !operation_ids.insert(receipt.producer.operation_id.as_str()) {
                return Err(ProductReceiptError::new(
                    "product receipt batch contains a duplicate producer operation id",
                ));
            }
            for artifact in receipt
                .build_products
                .iter()
                .chain(&receipt.runtime_dependencies)
                .chain(&receipt.symbols)
                .chain(receipt.sbom.as_ref())
            {
                if !artifact_names.insert(artifact.logical_name.as_str()) {
                    return Err(ProductReceiptError::new(format!(
                        "product receipt batch contains duplicate artifact logical name `{}`",
                        artifact.logical_name
                    )));
                }
                if !artifact_paths.insert(artifact.relative_path.as_str()) {
                    return Err(ProductReceiptError::new(format!(
                        "product receipt batch contains duplicate artifact relative path `{}`",
                        artifact.relative_path
                    )));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;
