//! 产品收据 CLI 的构建、交接、签发和核验编排。
//! 主程序路由七种子命令：构建写草案摘要，草案签发先核对交接与信任，验收再核对签名及物化闭包；仅成功后输出身份。

use std::process::ExitCode;

use cargo_zircon::build::product_build::{
    build_product_receipt_draft, build_product_receipt_draft_batch, ProductBuildBatchRequest,
    ProductBuildDraftBatch, ProductBuildRequest,
};
use cargo_zircon::build::receipt::{
    Ed25519ProductReceiptSigner, ProductReceipt, ProductReceiptBatch, ProductReceiptClosure,
    ProductReceiptDraft, ProductReceiptTrustRegistry,
};

use crate::usage_error;

use super::input::read_bounded;
use super::options::{
    parse_product_receipt_batch_verify_options, parse_product_receipt_build_options,
    parse_product_receipt_draft_batch_issue_options, parse_product_receipt_draft_issue_options,
    parse_product_receipt_issue_options, parse_product_receipt_verify_options,
    ProductReceiptBatchVerifyOptions, ProductReceiptBuildOptions,
    ProductReceiptDraftBatchIssueOptions, ProductReceiptDraftIssueOptions,
    ProductReceiptIssueOptions, ProductReceiptVerifyOptions,
};

const PRODUCT_RECEIPT_JSON_LIMIT: usize = 16 * 1024 * 1024;
const PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT: usize = 1024 * 1024;
const PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT: usize = 16 * 1024;

/// 主程序移除子命令前缀后调用；各路径分别建立构建、交接、签发或验收边界。
pub(crate) fn run(arguments: Vec<String>) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut arguments = arguments.into_iter();
    let command = arguments.next().ok_or_else(usage_error)?;
    match command.as_str() {
        "build" => {
            build_product_receipt_draft_file(parse_product_receipt_build_options(arguments)?)
        }
        "build-batch" => {
            build_product_receipt_draft_batch_file(parse_product_receipt_build_options(arguments)?)
        }
        "issue-draft" => {
            issue_product_receipt_draft(parse_product_receipt_draft_issue_options(arguments)?)
        }
        "issue-draft-batch" => issue_product_receipt_draft_batch(
            parse_product_receipt_draft_batch_issue_options(arguments)?,
        ),
        "issue" => issue_product_receipt(parse_product_receipt_issue_options(arguments)?),
        "verify" => verify_product_receipt(parse_product_receipt_verify_options(arguments)?),
        "verify-batch" => {
            verify_product_receipt_batch(parse_product_receipt_batch_verify_options(arguments)?)
        }
        _ => Err(usage_error().into()),
    }
}

fn build_product_receipt_draft_file(
    options: ProductReceiptBuildOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let request_bytes = read_bounded(
        &options.request,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product build request",
        &mut input,
    )?;
    let request: ProductBuildRequest = serde_json::from_slice(&request_bytes)?;
    drop(input);
    let draft = build_product_receipt_draft(request)?;
    let handoff_sha256 = draft.write_new_with_handoff_sha256(&options.output)?;
    println!("{handoff_sha256}");
    Ok(ExitCode::SUCCESS)
}

fn build_product_receipt_draft_batch_file(
    options: ProductReceiptBuildOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let request_bytes = read_bounded(
        &options.request,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product build batch request",
        &mut input,
    )?;
    let request: ProductBuildBatchRequest = serde_json::from_slice(&request_bytes)?;
    drop(input);
    let batch = build_product_receipt_draft_batch(request)?;
    let handoff_sha256 = batch.write_new_with_handoff_sha256(&options.output)?;
    println!("{handoff_sha256}");
    Ok(ExitCode::SUCCESS)
}

/// 先核对构建交接摘要，再以独立信任表自检签名；成功发布后才打印收据身份。
fn issue_product_receipt_draft(
    options: ProductReceiptDraftIssueOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let draft_bytes = read_bounded(
        &options.draft,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product receipt draft",
        &mut input,
    )?;
    let draft = ProductReceiptDraft::parse_and_verify_handoff_sha256(
        &draft_bytes,
        &options.expected_draft_sha256,
    )?;
    let trust_bytes = read_bounded(
        &options.trust_registry,
        PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT,
        "product receipt trust registry",
        &mut input,
    )?;
    let registry = ProductReceiptTrustRegistry::from_json(&trust_bytes)?;
    let key_bytes = read_bounded(
        &options.private_key,
        PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT,
        "product receipt private key",
        &mut input,
    )?;
    let signer = Ed25519ProductReceiptSigner::from_pkcs8(options.signer_id, &key_bytes)?;
    drop(input);
    let publication = draft.issue_verified(options.created_utc, &signer, &registry)?;
    publication.write_new(&options.output)?;
    println!("{}", publication.receipt_id());
    Ok(ExitCode::SUCCESS)
}

fn issue_product_receipt_draft_batch(
    options: ProductReceiptDraftBatchIssueOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let draft_bytes = read_bounded(
        &options.draft_batch,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product receipt draft batch",
        &mut input,
    )?;
    let draft_batch = ProductBuildDraftBatch::parse_and_verify_handoff_sha256(
        &draft_bytes,
        &options.expected_draft_sha256,
    )?;
    let trust_bytes = read_bounded(
        &options.trust_registry,
        PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT,
        "product receipt trust registry",
        &mut input,
    )?;
    let registry = ProductReceiptTrustRegistry::from_json(&trust_bytes)?;
    let key_bytes = read_bounded(
        &options.private_key,
        PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT,
        "product receipt private key",
        &mut input,
    )?;
    let signer = Ed25519ProductReceiptSigner::from_pkcs8(options.signer_id, &key_bytes)?;
    drop(input);
    let publication = draft_batch.issue_verified(options.created_utc, &signer, &registry)?;
    publication.write_new(&options.output)?;
    println!("{}", publication.batch_id());
    Ok(ExitCode::SUCCESS)
}

fn issue_product_receipt(
    options: ProductReceiptIssueOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let closure_bytes = read_bounded(
        &options.closure,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product receipt closure",
        &mut input,
    )?;
    let closure: ProductReceiptClosure = serde_json::from_slice(&closure_bytes)?;
    let draft = closure.capture()?;
    let key_bytes = read_bounded(
        &options.private_key,
        PRODUCT_RECEIPT_PRIVATE_KEY_LIMIT,
        "product receipt private key",
        &mut input,
    )?;
    let signer = Ed25519ProductReceiptSigner::from_pkcs8(options.signer_id, &key_bytes)?;
    drop(input);
    let publication = ProductReceipt::issue_verified(draft, options.created_utc, &signer, &signer)?;
    publication.write_new(&options.output)?;
    println!("{}", publication.receipt_id());
    Ok(ExitCode::SUCCESS)
}

/// 外部验收同时要求可信签名和完整落盘集合，不能只凭收据 JSON 自称成功。
fn verify_product_receipt(
    options: ProductReceiptVerifyOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let receipt_bytes = read_bounded(
        &options.receipt,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product receipt",
        &mut input,
    )?;
    let receipt: ProductReceipt = serde_json::from_slice(&receipt_bytes)?;
    let trust_bytes = read_bounded(
        &options.trust_registry,
        PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT,
        "product receipt trust registry",
        &mut input,
    )?;
    let registry = ProductReceiptTrustRegistry::from_json(&trust_bytes)?;
    drop(input);
    receipt.verify_attestation_and_materialization(&registry, &options.artifact_root)?;
    println!("{}", receipt.receipt_id);
    Ok(ExitCode::SUCCESS)
}

fn verify_product_receipt_batch(
    options: ProductReceiptBatchVerifyOptions,
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut input = Vec::new();
    let receipt_bytes = read_bounded(
        &options.receipt_batch,
        PRODUCT_RECEIPT_JSON_LIMIT,
        "product receipt batch",
        &mut input,
    )?;
    let receipt_batch: ProductReceiptBatch = serde_json::from_slice(&receipt_bytes)?;
    let trust_bytes = read_bounded(
        &options.trust_registry,
        PRODUCT_RECEIPT_TRUST_REGISTRY_LIMIT,
        "product receipt trust registry",
        &mut input,
    )?;
    let registry = ProductReceiptTrustRegistry::from_json(&trust_bytes)?;
    drop(input);
    receipt_batch.verify_attestations_and_materialization(&registry, &options.artifact_root)?;
    println!("{}", receipt_batch.batch_id);
    Ok(ExitCode::SUCCESS)
}
