use std::collections::HashSet;

use super::{
    ArtifactKind, BuildAction, ProducerIdentity, ProductReceipt, ProductReceiptDraft,
    ProductReceiptError, ReceiptArtifact, TargetProfile, ToolchainSet,
};

pub(crate) struct ValidatedCreatedUtc(());

pub(crate) fn normalize_and_validate(
    draft: &mut ProductReceiptDraft,
    created_utc: &str,
) -> Result<(), ProductReceiptError> {
    if validate_draft_if_normalized(draft, created_utc)? {
        return Ok(());
    }
    normalize_and_validate_owned(draft, created_utc)
}

pub(crate) fn normalize_and_validate_after_batch_shape(
    draft: &mut ProductReceiptDraft,
    created_utc: &str,
) -> Result<(), ProductReceiptError> {
    let validated_created_utc = validate_created_utc_for_batch(created_utc)?;
    normalize_and_validate_after_batch_shape_with_validated_utc(draft, &validated_created_utc)
}

pub(crate) fn normalize_and_validate_after_batch_shape_with_validated_utc(
    draft: &mut ProductReceiptDraft,
    _validated_created_utc: &ValidatedCreatedUtc,
) -> Result<(), ProductReceiptError> {
    if validate_draft_fields_if_normalized_without_timestamp(draft)? {
        return Ok(());
    }
    normalize_and_validate_owned_fields_without_timestamp(draft)
}

pub(crate) fn validate_created_utc_for_batch(
    created_utc: &str,
) -> Result<ValidatedCreatedUtc, ProductReceiptError> {
    validate_created_utc(created_utc)?;
    Ok(ValidatedCreatedUtc(()))
}

#[cfg(test)]
pub(crate) fn normalize_and_validate_owned_for_benchmark(
    draft: &mut ProductReceiptDraft,
    created_utc: &str,
) -> Result<(), ProductReceiptError> {
    normalize_and_validate_owned(draft, created_utc)
}

fn normalize_and_validate_owned(
    draft: &mut ProductReceiptDraft,
    created_utc: &str,
) -> Result<(), ProductReceiptError> {
    normalize_and_validate_owned_fields(draft, created_utc)?;
    validate_unique_artifact_names(
        &draft.build_products,
        &draft.runtime_dependencies,
        &draft.symbols,
        draft.sbom.as_ref(),
    )
}

fn normalize_and_validate_owned_fields(
    draft: &mut ProductReceiptDraft,
    created_utc: &str,
) -> Result<(), ProductReceiptError> {
    normalize_and_validate_owned_identity_fields(draft)?;
    validate_created_utc(created_utc)?;
    normalize_and_validate_owned_artifact_fields(draft)
}

fn normalize_and_validate_owned_fields_without_timestamp(
    draft: &mut ProductReceiptDraft,
) -> Result<(), ProductReceiptError> {
    normalize_and_validate_owned_identity_fields(draft)?;
    normalize_and_validate_owned_artifact_fields(draft)
}

fn normalize_and_validate_owned_identity_fields(
    draft: &mut ProductReceiptDraft,
) -> Result<(), ProductReceiptError> {
    validate_sha256("build set id", &mut draft.build_set_id)?;
    validate_target_profile(&mut draft.target_profile)?;
    validate_toolchain(&mut draft.toolchain, &draft.target_profile)?;
    validate_action(&mut draft.action)?;
    validate_producer(&draft.producer)
}

fn normalize_and_validate_owned_artifact_fields(
    draft: &mut ProductReceiptDraft,
) -> Result<(), ProductReceiptError> {
    if draft.build_products.is_empty() {
        return Err(ProductReceiptError::new(
            "product receipt requires at least one build product",
        ));
    }

    validate_artifacts(&mut draft.build_products, "build product", |kind| {
        matches!(kind, ArtifactKind::Executable)
    })?;
    validate_artifacts(
        &mut draft.runtime_dependencies,
        "runtime dependency",
        |kind| matches!(kind, ArtifactKind::DynamicLibrary | ArtifactKind::Resource),
    )?;
    validate_artifacts(&mut draft.symbols, "symbol", |kind| {
        matches!(kind, ArtifactKind::SymbolFile)
    })?;
    if let Some(sbom) = &mut draft.sbom {
        validate_artifact(sbom)?;
        validate_artifact_kind(sbom, "SBOM", |kind| matches!(kind, ArtifactKind::Sbom))?;
    }
    Ok(())
}

fn validate_draft_if_normalized(
    draft: &ProductReceiptDraft,
    created_utc: &str,
) -> Result<bool, ProductReceiptError> {
    if !validate_draft_identity_fields_if_normalized(draft)? {
        return Ok(false);
    }
    validate_created_utc(created_utc)?;
    validate_normalized_artifact_closure(
        &draft.build_products,
        &draft.runtime_dependencies,
        &draft.symbols,
        draft.sbom.as_ref(),
    )
}

fn validate_draft_fields_if_normalized(
    draft: &ProductReceiptDraft,
    created_utc: &str,
) -> Result<bool, ProductReceiptError> {
    if !validate_draft_identity_fields_if_normalized(draft)? {
        return Ok(false);
    }
    validate_created_utc(created_utc)?;
    validate_draft_artifact_fields_if_normalized(draft)
}

fn validate_draft_fields_if_normalized_without_timestamp(
    draft: &ProductReceiptDraft,
) -> Result<bool, ProductReceiptError> {
    if !validate_draft_identity_fields_if_normalized(draft)? {
        return Ok(false);
    }
    validate_draft_artifact_fields_if_normalized(draft)
}

fn validate_draft_identity_fields_if_normalized(
    draft: &ProductReceiptDraft,
) -> Result<bool, ProductReceiptError> {
    if !validate_sha256_if_normalized("build set id", &draft.build_set_id)?
        || !validate_target_profile_if_normalized(&draft.target_profile)?
        || !validate_toolchain_if_normalized(&draft.toolchain, &draft.target_profile)?
        || !validate_action_if_normalized(&draft.action)?
    {
        return Ok(false);
    }
    validate_producer(&draft.producer)?;
    Ok(true)
}

fn validate_draft_artifact_fields_if_normalized(
    draft: &ProductReceiptDraft,
) -> Result<bool, ProductReceiptError> {
    if draft.build_products.is_empty() {
        return Err(ProductReceiptError::new(
            "product receipt requires at least one build product",
        ));
    }

    if !validate_artifacts_if_normalized(&draft.build_products, "build product", |kind| {
        matches!(kind, ArtifactKind::Executable)
    })? || !validate_artifacts_if_normalized(
        &draft.runtime_dependencies,
        "runtime dependency",
        |kind| matches!(kind, ArtifactKind::DynamicLibrary | ArtifactKind::Resource),
    )? || !validate_artifacts_if_normalized(&draft.symbols, "symbol", |kind| {
        matches!(kind, ArtifactKind::SymbolFile)
    })? {
        return Ok(false);
    }
    if let Some(sbom) = &draft.sbom {
        if !validate_artifact_if_normalized(sbom)? {
            return Ok(false);
        }
        validate_artifact_kind(sbom, "SBOM", |kind| matches!(kind, ArtifactKind::Sbom))?;
    }
    Ok(true)
}

#[cfg(test)]
pub(crate) fn is_normalized_receipt_for_benchmark(receipt: &ProductReceipt) -> bool {
    is_normalized_sha256(&receipt.build_set_id)
        && receipt.toolchain.is_normalized()
        && is_normalized_sha256(&receipt.target_profile.codegen_flags_digest)
        && is_normalized_sha256(&receipt.target_profile.cargo_graph_digest)
        && strings_are_sorted(&receipt.action.features)
        && artifacts_are_normalized(&receipt.build_products)
        && artifacts_are_normalized(&receipt.runtime_dependencies)
        && artifacts_are_normalized(&receipt.symbols)
        && receipt
            .sbom
            .as_ref()
            .is_none_or(|artifact| is_normalized_sha256(&artifact.sha256))
}

pub(crate) fn validate_receipt_if_normalized(
    receipt: &ProductReceipt,
) -> Result<bool, ProductReceiptError> {
    if !validate_sha256_if_normalized("build set id", &receipt.build_set_id)?
        || !validate_target_profile_if_normalized(&receipt.target_profile)?
        || !validate_toolchain_if_normalized(&receipt.toolchain, &receipt.target_profile)?
        || !validate_action_if_normalized(&receipt.action)?
    {
        return Ok(false);
    }
    validate_producer(&receipt.producer)?;
    validate_created_utc(&receipt.created_utc)?;
    validate_normalized_artifact_closure(
        &receipt.build_products,
        &receipt.runtime_dependencies,
        &receipt.symbols,
        receipt.sbom.as_ref(),
    )
}

pub(crate) fn validate_required_text(label: &str, value: &str) -> Result<(), ProductReceiptError> {
    if value.trim().is_empty() {
        return Err(ProductReceiptError::new(format!(
            "product receipt {label} must not be empty"
        )));
    }
    Ok(())
}

fn validate_toolchain(
    toolchain: &mut ToolchainSet,
    target_profile: &TargetProfile,
) -> Result<(), ProductReceiptError> {
    toolchain.normalize_and_verify_identity()?;
    if toolchain.linker_sha256.is_none()
        && target_profile
            .target_triple
            .split('-')
            .any(|component| component.eq_ignore_ascii_case("windows"))
    {
        return Err(ProductReceiptError::new(
            "product receipt Windows target requires a linker fingerprint",
        ));
    }
    validate_sha256("SDK fingerprint", &mut toolchain.sdk_fingerprint)?;
    validate_sha256("environment digest", &mut toolchain.environment_digest)
}

fn validate_toolchain_if_normalized(
    toolchain: &ToolchainSet,
    target_profile: &TargetProfile,
) -> Result<bool, ProductReceiptError> {
    if !toolchain.validate_identity_if_normalized()? {
        return Ok(false);
    }
    if toolchain.linker_sha256.is_none()
        && target_profile
            .target_triple
            .split('-')
            .any(|component| component.eq_ignore_ascii_case("windows"))
    {
        return Err(ProductReceiptError::new(
            "product receipt Windows target requires a linker fingerprint",
        ));
    }
    Ok(true)
}

fn validate_action(action: &mut BuildAction) -> Result<(), ProductReceiptError> {
    validate_required_text("build package", &action.package)?;
    if let Some(bin) = &action.bin {
        validate_required_text("build binary", bin)?;
    }
    action.features.sort_unstable();
    let mut previous: Option<&str> = None;
    for feature in &action.features {
        validate_required_text("build feature", feature)?;
        if previous == Some(feature.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product receipt contains duplicate build feature `{feature}`"
            )));
        }
        previous = Some(feature.as_str());
    }
    Ok(())
}

fn validate_action_if_normalized(action: &BuildAction) -> Result<bool, ProductReceiptError> {
    validate_required_text("build package", &action.package)?;
    if let Some(bin) = &action.bin {
        validate_required_text("build binary", bin)?;
    }
    let mut previous: Option<&str> = None;
    for feature in &action.features {
        validate_required_text("build feature", feature)?;
        if let Some(previous) = previous {
            if previous > feature.as_str() {
                return Ok(false);
            }
            if previous == feature.as_str() {
                return Err(ProductReceiptError::new(format!(
                    "product receipt contains duplicate build feature `{feature}`"
                )));
            }
        }
        previous = Some(feature.as_str());
    }
    Ok(true)
}

fn validate_target_profile(profile: &mut TargetProfile) -> Result<(), ProductReceiptError> {
    validate_required_text("build target triple", &profile.target_triple)?;
    validate_required_text("Cargo profile", &profile.cargo_profile)?;
    validate_sha256("codegen flags digest", &mut profile.codegen_flags_digest)?;
    validate_sha256("Cargo graph digest", &mut profile.cargo_graph_digest)
}

fn validate_target_profile_if_normalized(
    profile: &TargetProfile,
) -> Result<bool, ProductReceiptError> {
    validate_required_text("build target triple", &profile.target_triple)?;
    validate_required_text("Cargo profile", &profile.cargo_profile)?;
    Ok(
        validate_sha256_if_normalized("codegen flags digest", &profile.codegen_flags_digest)?
            && validate_sha256_if_normalized("Cargo graph digest", &profile.cargo_graph_digest)?,
    )
}

fn validate_producer(producer: &ProducerIdentity) -> Result<(), ProductReceiptError> {
    validate_required_text("producer tool", &producer.tool)?;
    validate_required_text("producer tool version", &producer.tool_version)?;
    validate_required_text("producer worker id", &producer.worker_id)?;
    validate_required_text("producer operation id", &producer.operation_id)
}

fn validate_created_utc(value: &str) -> Result<(), ProductReceiptError> {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
    {
        return Err(invalid_created_utc());
    }

    let fraction = match (bytes[19], bytes.last()) {
        (b'Z', Some(b'Z')) if bytes.len() == 20 => &bytes[20..20],
        (b'.', Some(b'Z')) if bytes.len() > 21 => &bytes[20..bytes.len() - 1],
        _ => return Err(invalid_created_utc()),
    };
    if !fraction.iter().all(u8::is_ascii_digit) {
        return Err(invalid_created_utc());
    }

    let year = parse_decimal_component(bytes, 0, 4).ok_or_else(invalid_created_utc)?;
    let month = parse_decimal_component(bytes, 5, 2).ok_or_else(invalid_created_utc)?;
    let day = parse_decimal_component(bytes, 8, 2).ok_or_else(invalid_created_utc)?;
    let hour = parse_decimal_component(bytes, 11, 2).ok_or_else(invalid_created_utc)?;
    let minute = parse_decimal_component(bytes, 14, 2).ok_or_else(invalid_created_utc)?;
    let second = parse_decimal_component(bytes, 17, 2).ok_or_else(invalid_created_utc)?;
    if month == 0
        || month > 12
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 60
    {
        return Err(invalid_created_utc());
    }
    Ok(())
}

fn invalid_created_utc() -> ProductReceiptError {
    ProductReceiptError::new("product receipt created_utc must be an ISO-8601 UTC timestamp")
}

fn parse_decimal_component(bytes: &[u8], start: usize, length: usize) -> Option<u32> {
    bytes
        .get(start..start + length)?
        .iter()
        .try_fold(0_u32, |total, byte| {
            byte.is_ascii_digit()
                .then(|| total * 10 + u32::from(*byte - b'0'))
        })
}

fn days_in_month(year: u32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn validate_artifacts(
    artifacts: &mut [ReceiptArtifact],
    partition: &str,
    allowed_kind: fn(&ArtifactKind) -> bool,
) -> Result<(), ProductReceiptError> {
    artifacts.sort_unstable_by(|left, right| left.logical_name.cmp(&right.logical_name));
    for artifact in artifacts {
        validate_artifact(artifact)?;
        validate_artifact_kind(artifact, partition, allowed_kind)?;
    }
    Ok(())
}

fn validate_artifacts_if_normalized(
    artifacts: &[ReceiptArtifact],
    partition: &str,
    allowed_kind: fn(&ArtifactKind) -> bool,
) -> Result<bool, ProductReceiptError> {
    let mut previous: Option<&str> = None;
    for artifact in artifacts {
        if previous.is_some_and(|name| name > artifact.logical_name.as_str()) {
            return Ok(false);
        }
        if !validate_artifact_if_normalized(artifact)? {
            return Ok(false);
        }
        validate_artifact_kind(artifact, partition, allowed_kind)?;
        previous = Some(artifact.logical_name.as_str());
    }
    Ok(true)
}

fn validate_normalized_artifact_closure(
    build_products: &[ReceiptArtifact],
    runtime_dependencies: &[ReceiptArtifact],
    symbols: &[ReceiptArtifact],
    sbom: Option<&ReceiptArtifact>,
) -> Result<bool, ProductReceiptError> {
    if build_products.is_empty() {
        return Err(ProductReceiptError::new(
            "product receipt requires at least one build product",
        ));
    }

    let sbom_partition = sbom.map(std::slice::from_ref).unwrap_or(&[]);
    let partitions = [
        build_products,
        runtime_dependencies,
        symbols,
        sbom_partition,
    ];
    let artifact_count = build_products
        .len()
        .saturating_add(runtime_dependencies.len())
        .saturating_add(symbols.len())
        .saturating_add(sbom.is_some() as usize);
    let mut paths = HashSet::with_capacity(artifact_count);
    let mut duplicate_path = None;
    let mut offsets = [0_usize; 4];
    let mut previous: Option<&str> = None;

    loop {
        let mut selected_partition = None;
        for partition_index in 0..partitions.len() {
            let Some(artifact) = partitions[partition_index].get(offsets[partition_index]) else {
                continue;
            };
            let should_select = selected_partition.is_none_or(|selected_index| {
                artifact.logical_name.as_str()
                    < partitions[selected_index][offsets[selected_index]]
                        .logical_name
                        .as_str()
            });
            if should_select {
                selected_partition = Some(partition_index);
            }
        }

        let Some(partition_index) = selected_partition else {
            break;
        };
        let artifact = &partitions[partition_index][offsets[partition_index]];
        if offsets[partition_index] > 0
            && partitions[partition_index][offsets[partition_index] - 1].logical_name
                > artifact.logical_name
        {
            return Ok(false);
        }
        match partition_index {
            0 => {
                if !validate_artifact_if_normalized(artifact)? {
                    return Ok(false);
                }
                validate_artifact_kind(artifact, "build product", |kind| {
                    matches!(kind, ArtifactKind::Executable)
                })?;
            }
            1 => {
                if !validate_artifact_if_normalized(artifact)? {
                    return Ok(false);
                }
                validate_artifact_kind(artifact, "runtime dependency", |kind| {
                    matches!(kind, ArtifactKind::DynamicLibrary | ArtifactKind::Resource)
                })?;
            }
            2 => {
                if !validate_artifact_if_normalized(artifact)? {
                    return Ok(false);
                }
                validate_artifact_kind(artifact, "symbol", |kind| {
                    matches!(kind, ArtifactKind::SymbolFile)
                })?;
            }
            3 => {
                if !validate_artifact_if_normalized(artifact)? {
                    return Ok(false);
                }
                validate_artifact_kind(artifact, "SBOM", |kind| {
                    matches!(kind, ArtifactKind::Sbom)
                })?;
            }
            _ => unreachable!("artifact partition index is bounded"),
        }
        if previous == Some(artifact.logical_name.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product receipt contains duplicate artifact logical name `{}`",
                artifact.logical_name
            )));
        }
        previous = Some(artifact.logical_name.as_str());
        if !paths.insert(artifact.relative_path.as_str()) {
            duplicate_path.get_or_insert(artifact.relative_path.as_str());
        }
        offsets[partition_index] += 1;
    }

    if let Some(relative_path) = duplicate_path {
        return Err(ProductReceiptError::new(format!(
            "product receipt contains duplicate artifact relative path `{relative_path}`"
        )));
    }
    Ok(true)
}

fn validate_artifact_kind(
    artifact: &ReceiptArtifact,
    partition: &str,
    allowed_kind: fn(&ArtifactKind) -> bool,
) -> Result<(), ProductReceiptError> {
    if !allowed_kind(&artifact.kind) {
        return Err(ProductReceiptError::new(format!(
            "product receipt {partition} `{}` has an invalid artifact kind",
            artifact.logical_name
        )));
    }
    Ok(())
}

fn validate_artifact(artifact: &mut ReceiptArtifact) -> Result<(), ProductReceiptError> {
    validate_required_text("artifact logical name", &artifact.logical_name)?;
    validate_relative_path(&artifact.relative_path)?;
    validate_sha256("artifact SHA-256", &mut artifact.sha256)
}

fn validate_artifact_if_normalized(
    artifact: &ReceiptArtifact,
) -> Result<bool, ProductReceiptError> {
    validate_required_text("artifact logical name", &artifact.logical_name)?;
    validate_relative_path(&artifact.relative_path)?;
    validate_sha256_if_normalized("artifact SHA-256", &artifact.sha256)
}

fn validate_unique_artifact_names<'a>(
    build_products: &'a [ReceiptArtifact],
    runtime_dependencies: &'a [ReceiptArtifact],
    symbols: &'a [ReceiptArtifact],
    sbom: Option<&'a ReceiptArtifact>,
) -> Result<(), ProductReceiptError> {
    let artifact_count = build_products
        .len()
        .saturating_add(runtime_dependencies.len())
        .saturating_add(symbols.len())
        .saturating_add(if sbom.is_some() { 1 } else { 0 });
    let mut paths = HashSet::with_capacity(artifact_count);
    let mut duplicate_path = None;
    visit_unique_sorted_artifacts(
        build_products,
        runtime_dependencies,
        symbols,
        sbom,
        |artifact| {
            if !paths.insert(artifact.relative_path.as_str()) {
                duplicate_path.get_or_insert(artifact.relative_path.as_str());
            }
        },
    )?;
    if let Some(relative_path) = duplicate_path {
        return Err(ProductReceiptError::new(format!(
            "product receipt contains duplicate artifact relative path `{relative_path}`"
        )));
    }
    Ok(())
}

fn validate_unique_sorted_artifact_names<'a>(
    build_products: &'a [ReceiptArtifact],
    runtime_dependencies: &'a [ReceiptArtifact],
    symbols: &'a [ReceiptArtifact],
    sbom: Option<&'a ReceiptArtifact>,
) -> Result<(), ProductReceiptError> {
    visit_unique_sorted_artifacts(build_products, runtime_dependencies, symbols, sbom, |_| {})
}

fn visit_unique_sorted_artifacts<'a>(
    build_products: &'a [ReceiptArtifact],
    runtime_dependencies: &'a [ReceiptArtifact],
    symbols: &'a [ReceiptArtifact],
    sbom: Option<&'a ReceiptArtifact>,
    mut visit: impl FnMut(&'a ReceiptArtifact),
) -> Result<(), ProductReceiptError> {
    let sbom_partition = sbom.map(std::slice::from_ref).unwrap_or(&[]);
    let partitions = [
        build_products,
        runtime_dependencies,
        symbols,
        sbom_partition,
    ];
    let mut offsets = [0_usize; 4];
    let mut previous: Option<&str> = None;

    loop {
        let mut selected_partition = None;
        for partition_index in 0..partitions.len() {
            let Some(artifact) = partitions[partition_index].get(offsets[partition_index]) else {
                continue;
            };
            let should_select = selected_partition.is_none_or(|selected_index| {
                artifact.logical_name.as_str()
                    < partitions[selected_index][offsets[selected_index]]
                        .logical_name
                        .as_str()
            });
            if should_select {
                selected_partition = Some(partition_index);
            }
        }

        let Some(partition_index) = selected_partition else {
            break;
        };
        let artifact = &partitions[partition_index][offsets[partition_index]];
        if previous == Some(artifact.logical_name.as_str()) {
            return Err(ProductReceiptError::new(format!(
                "product receipt contains duplicate artifact logical name `{}`",
                artifact.logical_name
            )));
        }
        previous = Some(artifact.logical_name.as_str());
        offsets[partition_index] += 1;
        visit(artifact);
    }
    Ok(())
}

fn validate_relative_path(value: &str) -> Result<(), ProductReceiptError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() || bytes[0] == b'/' || bytes.last() == Some(&b'/') {
        return Err(invalid_relative_path());
    }
    if is_windows_drive_prefix(bytes) {
        return Err(escaping_relative_path());
    }

    let mut component_start = 0;
    for (index, byte) in bytes.iter().copied().enumerate() {
        match byte {
            b'\\' => return Err(invalid_relative_path()),
            b'/' => {
                if index == component_start {
                    return Err(invalid_relative_path());
                }
                if is_dot_component(&bytes[component_start..index]) {
                    return Err(escaping_relative_path());
                }
                component_start = index + 1;
            }
            _ => {}
        }
    }
    if is_dot_component(&bytes[component_start..]) {
        return Err(escaping_relative_path());
    }
    Ok(())
}

fn is_windows_drive_prefix(bytes: &[u8]) -> bool {
    bytes.len() >= 2 && bytes[0].is_ascii_alphabetic() && bytes[1] == b':'
}

fn is_dot_component(component: &[u8]) -> bool {
    component == b"." || component == b".."
}

fn invalid_relative_path() -> ProductReceiptError {
    ProductReceiptError::new("product receipt artifact relative path is invalid")
}

fn escaping_relative_path() -> ProductReceiptError {
    ProductReceiptError::new("product receipt artifact relative path escapes its closure")
}

fn validate_sha256(label: &str, value: &mut String) -> Result<(), ProductReceiptError> {
    if !validate_sha256_if_normalized(label, value)? {
        value.make_ascii_uppercase();
    }
    Ok(())
}

fn validate_sha256_if_normalized(label: &str, value: &str) -> Result<bool, ProductReceiptError> {
    if value.len() != 64 {
        return Err(ProductReceiptError::new(format!(
            "product receipt {label} must be a SHA-256 hex digest"
        )));
    }
    let mut normalized = true;
    for byte in value.bytes() {
        if !byte.is_ascii_hexdigit() {
            return Err(ProductReceiptError::new(format!(
                "product receipt {label} must be a SHA-256 hex digest"
            )));
        }
        normalized &= !byte.is_ascii_lowercase();
    }
    Ok(normalized)
}

#[cfg(test)]
fn artifacts_are_normalized(artifacts: &[ReceiptArtifact]) -> bool {
    artifacts
        .windows(2)
        .all(|pair| pair[0].logical_name.as_str() <= pair[1].logical_name.as_str())
        && artifacts
            .iter()
            .all(|artifact| is_normalized_sha256(&artifact.sha256))
}

#[cfg(test)]
fn strings_are_sorted(values: &[String]) -> bool {
    values
        .windows(2)
        .all(|pair| pair[0].as_str() <= pair[1].as_str())
}

#[cfg(test)]
fn is_normalized_sha256(value: &str) -> bool {
    !value.bytes().any(|byte| byte.is_ascii_lowercase())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod performance_tests;
