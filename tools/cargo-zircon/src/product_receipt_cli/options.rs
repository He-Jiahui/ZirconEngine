use std::io;
use std::path::PathBuf;

use crate::usage_error;

pub(super) struct ProductReceiptIssueOptions {
    pub(super) closure: PathBuf,
    pub(super) private_key: PathBuf,
    pub(super) signer_id: String,
    pub(super) created_utc: String,
    pub(super) output: PathBuf,
}

pub(super) struct ProductReceiptBuildOptions {
    pub(super) request: PathBuf,
    pub(super) output: PathBuf,
}

pub(super) struct ProductReceiptDraftIssueOptions {
    pub(super) draft: PathBuf,
    pub(super) expected_draft_sha256: String,
    pub(super) private_key: PathBuf,
    pub(super) trust_registry: PathBuf,
    pub(super) signer_id: String,
    pub(super) created_utc: String,
    pub(super) output: PathBuf,
}

pub(super) struct ProductReceiptDraftBatchIssueOptions {
    pub(super) draft_batch: PathBuf,
    pub(super) expected_draft_sha256: String,
    pub(super) private_key: PathBuf,
    pub(super) trust_registry: PathBuf,
    pub(super) signer_id: String,
    pub(super) created_utc: String,
    pub(super) output: PathBuf,
}

pub(super) struct ProductReceiptVerifyOptions {
    pub(super) receipt: PathBuf,
    pub(super) trust_registry: PathBuf,
    pub(super) artifact_root: PathBuf,
}

pub(super) struct ProductReceiptBatchVerifyOptions {
    pub(super) receipt_batch: PathBuf,
    pub(super) trust_registry: PathBuf,
    pub(super) artifact_root: PathBuf,
}

pub(super) fn parse_product_receipt_build_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptBuildOptions, io::Error> {
    let mut request = None;
    let mut output = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--request" => set_once(&mut request, PathBuf::from(value))?,
            "--output" => set_once(&mut output, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptBuildOptions {
        request: request.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
    })
}

pub(super) fn parse_product_receipt_draft_issue_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptDraftIssueOptions, io::Error> {
    let mut draft = None;
    let mut expected_draft_sha256 = None;
    let mut private_key = None;
    let mut trust_registry = None;
    let mut signer_id = None;
    let mut created_utc = None;
    let mut output = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--draft" => set_once(&mut draft, PathBuf::from(value))?,
            "--expected-draft-sha256" => set_once(&mut expected_draft_sha256, value)?,
            "--private-key" => set_once(&mut private_key, PathBuf::from(value))?,
            "--trust-registry" => set_once(&mut trust_registry, PathBuf::from(value))?,
            "--signer-id" => set_once(&mut signer_id, value)?,
            "--created-utc" => set_once(&mut created_utc, value)?,
            "--output" => set_once(&mut output, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptDraftIssueOptions {
        draft: draft.ok_or_else(usage_error)?,
        expected_draft_sha256: expected_draft_sha256.ok_or_else(usage_error)?,
        private_key: private_key.ok_or_else(usage_error)?,
        trust_registry: trust_registry.ok_or_else(usage_error)?,
        signer_id: signer_id.ok_or_else(usage_error)?,
        created_utc: created_utc.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
    })
}

pub(super) fn parse_product_receipt_draft_batch_issue_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptDraftBatchIssueOptions, io::Error> {
    let mut draft_batch = None;
    let mut expected_draft_sha256 = None;
    let mut private_key = None;
    let mut trust_registry = None;
    let mut signer_id = None;
    let mut created_utc = None;
    let mut output = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--draft-batch" => set_once(&mut draft_batch, PathBuf::from(value))?,
            "--expected-draft-sha256" => set_once(&mut expected_draft_sha256, value)?,
            "--private-key" => set_once(&mut private_key, PathBuf::from(value))?,
            "--trust-registry" => set_once(&mut trust_registry, PathBuf::from(value))?,
            "--signer-id" => set_once(&mut signer_id, value)?,
            "--created-utc" => set_once(&mut created_utc, value)?,
            "--output" => set_once(&mut output, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptDraftBatchIssueOptions {
        draft_batch: draft_batch.ok_or_else(usage_error)?,
        expected_draft_sha256: expected_draft_sha256.ok_or_else(usage_error)?,
        private_key: private_key.ok_or_else(usage_error)?,
        trust_registry: trust_registry.ok_or_else(usage_error)?,
        signer_id: signer_id.ok_or_else(usage_error)?,
        created_utc: created_utc.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
    })
}

pub(super) fn parse_product_receipt_issue_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptIssueOptions, io::Error> {
    let mut closure = None;
    let mut private_key = None;
    let mut signer_id = None;
    let mut created_utc = None;
    let mut output = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--closure" => set_once(&mut closure, PathBuf::from(value))?,
            "--private-key" => set_once(&mut private_key, PathBuf::from(value))?,
            "--signer-id" => set_once(&mut signer_id, value)?,
            "--created-utc" => set_once(&mut created_utc, value)?,
            "--output" => set_once(&mut output, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptIssueOptions {
        closure: closure.ok_or_else(usage_error)?,
        private_key: private_key.ok_or_else(usage_error)?,
        signer_id: signer_id.ok_or_else(usage_error)?,
        created_utc: created_utc.ok_or_else(usage_error)?,
        output: output.ok_or_else(usage_error)?,
    })
}

pub(super) fn parse_product_receipt_verify_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptVerifyOptions, io::Error> {
    let mut receipt = None;
    let mut trust_registry = None;
    let mut artifact_root = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--receipt" => set_once(&mut receipt, PathBuf::from(value))?,
            "--trust-registry" => set_once(&mut trust_registry, PathBuf::from(value))?,
            "--artifact-root" => set_once(&mut artifact_root, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptVerifyOptions {
        receipt: receipt.ok_or_else(usage_error)?,
        trust_registry: trust_registry.ok_or_else(usage_error)?,
        artifact_root: artifact_root.ok_or_else(usage_error)?,
    })
}

pub(super) fn parse_product_receipt_batch_verify_options(
    arguments: impl IntoIterator<Item = String>,
) -> Result<ProductReceiptBatchVerifyOptions, io::Error> {
    let mut receipt_batch = None;
    let mut trust_registry = None;
    let mut artifact_root = None;
    let mut arguments = arguments.into_iter();
    while let Some(argument) = arguments.next() {
        let value = arguments.next().ok_or_else(usage_error)?;
        match argument.as_str() {
            "--receipt-batch" => set_once(&mut receipt_batch, PathBuf::from(value))?,
            "--trust-registry" => set_once(&mut trust_registry, PathBuf::from(value))?,
            "--artifact-root" => set_once(&mut artifact_root, PathBuf::from(value))?,
            _ => return Err(usage_error()),
        }
    }
    Ok(ProductReceiptBatchVerifyOptions {
        receipt_batch: receipt_batch.ok_or_else(usage_error)?,
        trust_registry: trust_registry.ok_or_else(usage_error)?,
        artifact_root: artifact_root.ok_or_else(usage_error)?,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T) -> Result<(), io::Error> {
    if slot.replace(value).is_some() {
        return Err(usage_error());
    }
    Ok(())
}

#[cfg(test)]
mod performance_tests;

#[cfg(test)]
mod tests {
    use super::{
        parse_product_receipt_build_options, parse_product_receipt_draft_batch_issue_options,
    };

    #[test]
    fn owned_batch_issue_options_preserve_values() {
        let options = parse_product_receipt_draft_batch_issue_options(strings(&[
            "--draft-batch",
            "drafts.json",
            "--expected-draft-sha256",
            "ABCDEF",
            "--private-key",
            "key.pk8",
            "--trust-registry",
            "trust.json",
            "--signer-id",
            "build-worker-01",
            "--created-utc",
            "2026-08-29T00:00:00Z",
            "--output",
            "receipt.json",
        ]))
        .unwrap();

        assert_eq!(options.draft_batch.to_str(), Some("drafts.json"));
        assert_eq!(options.expected_draft_sha256, "ABCDEF");
        assert_eq!(options.private_key.to_str(), Some("key.pk8"));
        assert_eq!(options.trust_registry.to_str(), Some("trust.json"));
        assert_eq!(options.signer_id, "build-worker-01");
        assert_eq!(options.created_utc, "2026-08-29T00:00:00Z");
        assert_eq!(options.output.to_str(), Some("receipt.json"));
    }

    #[test]
    fn owned_options_reject_duplicates_and_missing_values() {
        assert!(parse_product_receipt_build_options(strings(&[
            "--request",
            "first.json",
            "--request",
            "second.json",
            "--output",
            "draft.json",
        ]))
        .is_err());
        assert!(parse_product_receipt_build_options(strings(&["--request"])).is_err());
    }

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }
}
