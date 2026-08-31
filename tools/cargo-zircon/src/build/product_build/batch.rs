use std::collections::HashSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::{
    build_product_receipt_draft_in_build_set, build_set, validate_build_request,
    PreparedProductBuildToolchain, ProductBuildRequest,
};
use crate::build::receipt::{
    canonical::{
        canonical_build_action_key, serialized_sha256_matches, sha256_bytes_matches,
        sha256_serialized,
    },
    receipt_writer, FreshProductReceiptBatch, ProductReceiptBatch, ProductReceiptDraft,
    ProductReceiptError, ProductReceiptSigner, ProductReceiptVerifier,
    VerifiedProductReceiptBatchPublication,
};

const PRODUCT_BUILD_BATCH_REQUEST_SCHEMA_VERSION: u32 = 1;
const PRODUCT_BUILD_BATCH_LIMIT: usize = 16;
const PRODUCT_BUILD_DRAFT_BATCH_KIND: &str = "zircon_product_build_draft_batch";
const HANDOFF_SERIALIZATION_ERROR: &str =
    "could not serialize product build draft batch handoff identity";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildBatchRequest {
    pub schema_version: u32,
    pub builds: Vec<ProductBuildRequest>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProductBuildDraftBatch {
    pub schema_version: u32,
    pub draft_batch_kind: String,
    pub build_set_id: String,
    pub drafts: Vec<ProductReceiptDraft>,
}

pub struct VerifiedProductBuildDraftBatchHandoff(ProductBuildDraftBatch);

impl ProductBuildDraftBatch {
    pub fn handoff_sha256(&self) -> Result<String, ProductReceiptError> {
        self.validate_shape()?;
        sha256_serialized(self, HANDOFF_SERIALIZATION_ERROR)
    }

    pub fn write_new_with_handoff_sha256(
        &self,
        output_path: impl AsRef<Path>,
    ) -> Result<String, ProductReceiptError> {
        self.validate_shape()?;
        receipt_writer::write_new_canonical_json_with_sha256(self, output_path.as_ref())
    }

    pub fn verify_handoff_sha256(&self, expected: &str) -> Result<(), ProductReceiptError> {
        self.validate_shape()?;
        if !serialized_sha256_matches(self, expected, HANDOFF_SERIALIZATION_ERROR)? {
            return Err(ProductReceiptError::new(
                "product build draft batch does not match the build-owner handoff digest",
            ));
        }
        Ok(())
    }

    pub fn verify_handoff_sha256_owned(
        self,
        expected: &str,
    ) -> Result<VerifiedProductBuildDraftBatchHandoff, ProductReceiptError> {
        self.verify_handoff_sha256(expected)?;
        Ok(VerifiedProductBuildDraftBatchHandoff(self))
    }

    pub fn parse_and_verify_handoff_sha256(
        serialized: &[u8],
        expected: &str,
    ) -> Result<VerifiedProductBuildDraftBatchHandoff, ProductReceiptError> {
        let batch: Self = serde_json::from_slice(serialized).map_err(|error| {
            ProductReceiptError::new(format!(
                "could not parse product build draft batch handoff: {error}"
            ))
        })?;
        batch.validate_shape()?;
        if !sha256_bytes_matches(serialized, expected)
            && !serialized_sha256_matches(&batch, expected, HANDOFF_SERIALIZATION_ERROR)?
        {
            return Err(ProductReceiptError::new(
                "product build draft batch does not match the build-owner handoff digest",
            ));
        }
        Ok(VerifiedProductBuildDraftBatchHandoff(batch))
    }

    pub fn write_new(&self, output_path: impl AsRef<Path>) -> Result<(), ProductReceiptError> {
        self.validate_shape()?;
        receipt_writer::write_new_canonical_json_with_sha256(self, output_path.as_ref()).map(drop)
    }

    pub fn issue(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<ProductReceiptBatch, ProductReceiptError> {
        self.validate_shape()?;
        self.issue_after_shape_validation(created_utc, signer)
    }

    fn issue_after_shape_validation(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<ProductReceiptBatch, ProductReceiptError> {
        self.issue_fresh_after_shape_validation(created_utc, signer)
            .map(FreshProductReceiptBatch::into_inner)
    }

    fn issue_fresh_after_shape_validation(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<FreshProductReceiptBatch, ProductReceiptError> {
        // Batch shape already proves exact artifact names and paths are globally unique.
        ProductReceiptBatch::issue_fresh_from_batch_shape_drafts(
            self.build_set_id,
            self.drafts,
            created_utc.into(),
            signer,
        )
    }

    fn validate_shape(&self) -> Result<(), ProductReceiptError> {
        if self.schema_version != PRODUCT_BUILD_BATCH_REQUEST_SCHEMA_VERSION
            || self.draft_batch_kind != PRODUCT_BUILD_DRAFT_BATCH_KIND
        {
            return Err(ProductReceiptError::new(
                "product build draft batch has an unsupported schema or kind",
            ));
        }
        if self.drafts.len() < 2 || self.drafts.len() > PRODUCT_BUILD_BATCH_LIMIT {
            return Err(ProductReceiptError::new(format!(
                "product build draft batch requires between 2 and {PRODUCT_BUILD_BATCH_LIMIT} drafts"
            )));
        }
        let mut actions = HashSet::with_capacity(self.drafts.len());
        let mut operation_ids = HashSet::with_capacity(self.drafts.len());
        let artifact_count = self.drafts.iter().fold(0_usize, |count, draft| {
            count
                .saturating_add(draft.build_products.len())
                .saturating_add(draft.runtime_dependencies.len())
                .saturating_add(draft.symbols.len())
                .saturating_add(usize::from(draft.sbom.is_some()))
        });
        let mut artifact_names = HashSet::with_capacity(artifact_count);
        let mut artifact_paths = HashSet::with_capacity(artifact_count);
        for draft in &self.drafts {
            if draft.build_set_id != self.build_set_id {
                return Err(ProductReceiptError::new(
                    "product build draft batch must bind every draft to its declared BuildSet",
                ));
            }
            if !actions.insert(canonical_build_action_key(&draft.action)) {
                return Err(ProductReceiptError::new(
                    "product build draft batch contains a duplicate build action",
                ));
            }
            if !operation_ids.insert(draft.producer.operation_id.as_str()) {
                return Err(ProductReceiptError::new(
                    "product build draft batch contains a duplicate producer operation id",
                ));
            }
            for artifact in draft
                .build_products
                .iter()
                .chain(&draft.runtime_dependencies)
                .chain(&draft.symbols)
                .chain(draft.sbom.as_ref())
            {
                insert_batch_artifact_identity(
                    &mut artifact_names,
                    &mut artifact_paths,
                    &artifact.logical_name,
                    &artifact.relative_path,
                )?;
            }
        }
        Ok(())
    }
}

impl VerifiedProductBuildDraftBatchHandoff {
    pub fn issue(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
    ) -> Result<ProductReceiptBatch, ProductReceiptError> {
        self.0.issue_after_shape_validation(created_utc, signer)
    }

    pub fn issue_verified(
        self,
        created_utc: impl Into<String>,
        signer: &dyn ProductReceiptSigner,
        verifier: &dyn ProductReceiptVerifier,
    ) -> Result<VerifiedProductReceiptBatchPublication, ProductReceiptError> {
        self.0
            .issue_fresh_after_shape_validation(created_utc, signer)?
            .verify_attestations(verifier)
    }
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests;

pub fn build_product_receipt_draft_batch(
    mut request: ProductBuildBatchRequest,
) -> Result<ProductBuildDraftBatch, ProductReceiptError> {
    validate_build_batch_request(&mut request)?;
    let build_set = build_set::ValidatedBuildSet::open(&request.builds[0].build_set_manifest_path)?;
    let mut prepared_toolchain =
        PreparedProductBuildToolchain::open(&mut request.builds[0].toolchain)?;
    let mut drafts = Vec::with_capacity(request.builds.len());
    for build in request.builds {
        drafts.push(build_product_receipt_draft_in_build_set(
            build,
            &build_set,
            &mut prepared_toolchain,
        )?);
    }
    let build_set_id = build_set.build_set_id;
    Ok(ProductBuildDraftBatch {
        schema_version: PRODUCT_BUILD_BATCH_REQUEST_SCHEMA_VERSION,
        draft_batch_kind: PRODUCT_BUILD_DRAFT_BATCH_KIND.to_string(),
        build_set_id,
        drafts,
    })
}

fn validate_build_batch_request(
    request: &mut ProductBuildBatchRequest,
) -> Result<(), ProductReceiptError> {
    if request.schema_version != PRODUCT_BUILD_BATCH_REQUEST_SCHEMA_VERSION {
        return Err(ProductReceiptError::new(format!(
            "unsupported product build batch request schema version {}",
            request.schema_version
        )));
    }
    if request.builds.len() < 2 || request.builds.len() > PRODUCT_BUILD_BATCH_LIMIT {
        return Err(ProductReceiptError::new(format!(
            "product build batch requires between 2 and {PRODUCT_BUILD_BATCH_LIMIT} builds"
        )));
    }
    for build in &mut request.builds {
        validate_build_request(build)?;
    }

    let first = &request.builds[0];
    let mut actions = HashSet::with_capacity(request.builds.len());
    let mut targets = HashSet::with_capacity(request.builds.len());
    let mut operation_ids = HashSet::with_capacity(request.builds.len());
    let artifact_count = request.builds.iter().fold(0_usize, |count, build| {
        count
            .saturating_add(1)
            .saturating_add(build.runtime_dependencies.len())
            .saturating_add(usize::from(build.sbom.is_some()))
    });
    let mut artifact_names = HashSet::with_capacity(artifact_count);
    let mut artifact_paths = HashSet::with_capacity(artifact_count);
    let mut symbol_directories = HashSet::with_capacity(request.builds.len());
    for build in &request.builds {
        if build.build_set_manifest_path != first.build_set_manifest_path {
            return Err(ProductReceiptError::new(
                "product build batch must use one BuildSet manifest",
            ));
        }
        if build.manifest_path != first.manifest_path
            || build.toolchain != first.toolchain
            || build.target != first.target
            || build.environment_policy != first.environment_policy
        {
            return Err(ProductReceiptError::new(
                "product build batch must share one manifest, toolchain, target profile, and environment policy",
            ));
        }
        if !actions.insert(canonical_build_action_key(&build.action)) {
            return Err(ProductReceiptError::new(
                "product build batch contains a duplicate build action",
            ));
        }
        if !targets.insert(build.target_directory.as_path()) {
            return Err(ProductReceiptError::new(
                "product build batch must use one fresh target directory per build action",
            ));
        }
        if !operation_ids.insert(build.producer.operation_id.as_str()) {
            return Err(ProductReceiptError::new(
                "product build batch contains a duplicate producer operation id",
            ));
        }
        if !symbol_directories.insert(build.product.symbol_relative_directory.as_str()) {
            return Err(ProductReceiptError::new(
                "product build batch must use a distinct symbol directory per build action",
            ));
        }
        insert_batch_artifact_identity(
            &mut artifact_names,
            &mut artifact_paths,
            &build.product.logical_name,
            &build.product.relative_path,
        )?;
        for dependency in &build.runtime_dependencies {
            insert_batch_artifact_identity(
                &mut artifact_names,
                &mut artifact_paths,
                &dependency.logical_name,
                &dependency.relative_path,
            )?;
        }
        if let Some(sbom) = &build.sbom {
            insert_batch_artifact_identity(
                &mut artifact_names,
                &mut artifact_paths,
                &sbom.logical_name,
                &sbom.relative_path,
            )?;
        }
    }
    Ok(())
}

fn insert_batch_artifact_identity<'a>(
    names: &mut HashSet<&'a str>,
    paths: &mut HashSet<&'a str>,
    logical_name: &'a str,
    relative_path: &'a str,
) -> Result<(), ProductReceiptError> {
    if !names.insert(logical_name) {
        return Err(ProductReceiptError::new(format!(
            "product build batch contains duplicate artifact logical name `{logical_name}`"
        )));
    }
    if !paths.insert(relative_path) {
        return Err(ProductReceiptError::new(format!(
            "product build batch contains duplicate artifact relative path `{relative_path}`"
        )));
    }
    Ok(())
}
