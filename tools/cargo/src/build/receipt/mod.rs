//! 构建收据模块的导出面。
//! 公开草案、签发、核验与信任接口；规范字节、落盘核验和文件摘要保留 crate 内边界。

#[path = "artifact_kind.rs"]
mod artifact_kind;
#[path = "build_action.rs"]
mod build_action;
#[path = "canonical.rs"]
pub(crate) mod canonical;
#[path = "ed25519_authority.rs"]
mod ed25519_authority;
#[path = "file_digest.rs"]
mod file_digest;
#[path = "materialization.rs"]
mod materialization;
#[path = "producer_identity.rs"]
mod producer_identity;
#[path = "product_receipt.rs"]
mod product_receipt;
#[path = "product_receipt_batch.rs"]
mod product_receipt_batch;
#[path = "product_receipt_closure.rs"]
mod product_receipt_closure;
#[path = "product_receipt_draft.rs"]
mod product_receipt_draft;
#[path = "product_receipt_error.rs"]
mod product_receipt_error;
#[path = "product_receipt_signer.rs"]
mod product_receipt_signer;
#[path = "product_receipt_verifier.rs"]
mod product_receipt_verifier;
#[path = "receipt_artifact.rs"]
mod receipt_artifact;
#[path = "receipt_attestation.rs"]
mod receipt_attestation;
#[path = "receipt_writer.rs"]
pub(crate) mod receipt_writer;
#[path = "target_profile.rs"]
mod target_profile;
#[path = "toolchain_set.rs"]
mod toolchain_set;
#[path = "validation.rs"]
mod validation;

pub use artifact_kind::ArtifactKind;
pub use build_action::BuildAction;
pub use ed25519_authority::{
    Ed25519ProductReceiptSigner, ProductReceiptTrustRegistry, ED25519_RECEIPT_ALGORITHM,
};
pub(crate) use file_digest::{
    digest_open_file_handle_with_buffer, digest_open_file_with_buffer, FileDigestBuffer,
};
pub use producer_identity::ProducerIdentity;
pub(crate) use product_receipt::{
    FreshAttestation, FreshProductReceipt, ValidatedProductReceiptSigner,
};
pub use product_receipt::{ProductReceipt, VerifiedProductReceiptPublication};
pub(crate) use product_receipt_batch::FreshProductReceiptBatch;
pub use product_receipt_batch::{ProductReceiptBatch, VerifiedProductReceiptBatchPublication};
pub use product_receipt_closure::{ProductReceiptClosure, ReceiptArtifactSource, ToolchainSource};
pub use product_receipt_draft::{ProductReceiptDraft, VerifiedProductReceiptDraftHandoff};
pub use product_receipt_error::ProductReceiptError;
pub use product_receipt_signer::ProductReceiptSigner;
pub use product_receipt_verifier::ProductReceiptVerifier;
pub use receipt_artifact::ReceiptArtifact;
pub use receipt_attestation::ReceiptAttestation;
pub use target_profile::TargetProfile;
pub(crate) use toolchain_set::ToolchainComponentDigests;
pub use toolchain_set::ToolchainSet;
pub(crate) use validation::{validate_created_utc_for_batch, ValidatedCreatedUtc};
