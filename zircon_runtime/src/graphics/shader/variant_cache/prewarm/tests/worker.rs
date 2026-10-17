use std::cell::Cell;
use std::collections::HashMap;

use crate::core::framework::render::{
    ShaderVariantPrewarmExecutionBudget, ShaderVariantPrewarmManifest, ShaderVariantPrewarmSource,
};

use super::{prewarm_shader_variants_to_disk_inner, validate_source_once};

#[test]
fn shared_source_wgsl_validation_is_cached_once_per_prewarm_batch() {
    let source = ShaderVariantPrewarmSource::new(
        "res://materials/shared.wgsl",
        "fn main() {}",
        Vec::new(),
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    );
    let mut validation_results = HashMap::new();
    let validation_count = Cell::new(0usize);

    validate_source_once(&mut validation_results, &source, |_| {
        validation_count.set(validation_count.get() + 1);
        Ok(())
    })
    .expect("first validation should pass");
    validate_source_once(&mut validation_results, &source, |_| {
        validation_count.set(validation_count.get() + 1);
        Ok(())
    })
    .expect("cached validation should pass");

    assert_eq!(validation_count.get(), 1);
}

fn expect_manifest_preflight_error(
    manifest: ShaderVariantPrewarmManifest,
    budget: ShaderVariantPrewarmExecutionBudget,
    message: &str,
    rejected_count: usize,
) {
    let cache_root = std::env::temp_dir().join(format!(
        "shader-preflight-unused-cache-{}",
        std::process::id()
    ));
    let report = prewarm_shader_variants_to_disk_inner(&manifest, &cache_root, budget, None, None);
    assert_eq!(
        (
            report.requested_count,
            report.written_count,
            report.failed_count
        ),
        (0, 0, 0)
    );
    assert!(report.failures.is_empty());
    assert!(report.written_variants.is_empty());
    assert!(
        report
            .preflight_error
            .as_deref()
            .is_some_and(|error| error.contains(message)),
        "expected preflight error containing {message:?}, got {:?}",
        report.preflight_error
    );
    assert_eq!(
        report.execution_budget.max_in_flight_variants,
        budget.max_in_flight_variants
    );
    assert_eq!(
        report.execution_budget.max_in_flight_source_bytes,
        budget.max_in_flight_source_bytes
    );
    assert_eq!(
        report.execution_budget.max_resident_source_bytes,
        budget.max_resident_source_bytes
    );
    assert_eq!(report.execution_budget.rejected_count, rejected_count);
    assert_eq!(report.execution_budget.peak_in_flight_variants, 0);
    assert_eq!(report.execution_budget.peak_in_flight_source_bytes, 0);
    if budget.validate().is_ok() {
        assert_eq!(
            report.execution_budget.resident_source_bytes,
            manifest.source_table_resident_bytes().unwrap()
        );
    }
    assert!(
        !cache_root.exists(),
        "preflight must not create cache files: {}",
        cache_root.display()
    );
}

fn preflight_fixture_source() -> ShaderVariantPrewarmSource {
    ShaderVariantPrewarmSource::new(
        "res://materials/preflight.wgsl",
        "fn main() {}",
        Vec::new(),
        "template-r1",
        "naga-r1",
        "wgpu-r1",
    )
}

#[test]
fn whole_manifest_preflight_invalid_budget_does_not_invent_variant_zero() {
    for budget in [
        ShaderVariantPrewarmExecutionBudget {
            max_in_flight_variants: 0,
            ..Default::default()
        },
        ShaderVariantPrewarmExecutionBudget {
            max_in_flight_source_bytes: 0,
            ..Default::default()
        },
        ShaderVariantPrewarmExecutionBudget {
            max_resident_source_bytes: 0,
            ..Default::default()
        },
    ] {
        expect_manifest_preflight_error(ShaderVariantPrewarmManifest::empty(), budget, "must", 1);
    }
}

#[test]
fn whole_manifest_preflight_schema_error_does_not_invent_variant_zero() {
    let mut manifest = ShaderVariantPrewarmManifest::empty();
    manifest.schema_version = ShaderVariantPrewarmManifest::SCHEMA_VERSION + 1;
    expect_manifest_preflight_error(manifest, Default::default(), "schema", 0);
}

#[test]
fn whole_manifest_preflight_integrity_error_does_not_invent_variant_zero() {
    let source = preflight_fixture_source();
    let manifest = ShaderVariantPrewarmManifest::new(vec![source.clone(), source], Vec::new());
    expect_manifest_preflight_error(manifest, Default::default(), "more than once", 0);
}

#[test]
fn whole_manifest_preflight_residency_error_preserves_budget_rejection() {
    let manifest = ShaderVariantPrewarmManifest::new(vec![preflight_fixture_source()], Vec::new());
    let budget = ShaderVariantPrewarmExecutionBudget {
        max_resident_source_bytes: 1,
        ..Default::default()
    };
    expect_manifest_preflight_error(manifest, budget, "resident bytes", 1);
}
