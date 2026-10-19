use std::path::PathBuf;
use std::sync::Arc;

use super::NativePluginDiscoveryRefreshInput;

#[test]
fn load_manifest_input_clones_share_export_root() {
    let input =
        NativePluginDiscoveryRefreshInput::load_manifest(PathBuf::from("export/plugins/native"));
    let cloned = input.clone();

    let (
        NativePluginDiscoveryRefreshInput::LoadManifest {
            export_root: original,
        },
        NativePluginDiscoveryRefreshInput::LoadManifest {
            export_root: cloned,
        },
    ) = (&input, &cloned)
    else {
        panic!("load-manifest constructor preserves the input variant");
    };

    assert!(Arc::ptr_eq(original, cloned));
}

fn bounded_request(
    path: &std::path::Path,
    budget: super::NativePluginDiscoveryRefreshBudget,
) -> (
    super::NativePluginDiscoveryRefreshRequest,
    super::NativePluginDiscoveryRefreshSink,
) {
    let cancellation = super::super::ticket::NativePluginDiscoveryRefreshCancellation::new(
        std::time::Instant::now() + std::time::Duration::from_secs(30),
    );
    let request = super::NativePluginDiscoveryRefreshRequest::new(
        super::NativePluginDiscoveryRoot::from_canonical_path(path),
        NativePluginDiscoveryRefreshInput::root_scan(),
        super::NativePluginDiscoveryRefreshWork::FullRootScan,
        None,
        1,
        budget.clone(),
        cancellation,
    );
    (
        request,
        super::NativePluginDiscoveryRefreshSink::new(budget),
    )
}

fn typed_source_fixture(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "zircon-typed-manifest-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock after epoch")
            .as_nanos()
    ))
}

#[test]
fn bounded_reader_preserves_real_missing_file_source_and_clone_identity() {
    use crate::plugin::native_plugin_loader::candidate_from_manifest::read_bounded_utf8_file;
    let path = typed_source_fixture("missing");
    let (request, mut sink) =
        bounded_request(&path, super::NativePluginDiscoveryRefreshBudget::default());
    let error = read_bounded_utf8_file(&request, &mut sink, &path, "native plugin manifest")
        .expect_err("real missing path must fail");
    assert!(matches!(
        &error,
        super::NativePluginDiscoveryRefreshError::ReadManifest {
            manifest_path, file_kind, ..
        } if manifest_path == &path && file_kind.as_ref() == "native plugin manifest"
    ));
    let source = std::error::Error::source(&error)
        .and_then(|source| source.downcast_ref::<std::io::Error>())
        .expect("actual IO source");
    assert_eq!(source.kind(), std::io::ErrorKind::NotFound);
    assert_eq!(error, error.clone());
    let independent = read_bounded_utf8_file(&request, &mut sink, &path, "native plugin manifest")
        .expect_err("second real read fails independently");
    assert_ne!(
        error, independent,
        "display text must not conflate source identity"
    );
}

#[test]
fn metered_candidate_preserves_real_toml_source_span_and_clone_identity() {
    use crate::plugin::native_plugin_loader::candidate_from_manifest::metered_candidate_from_manifest_path;
    let path = typed_source_fixture("toml");
    std::fs::write(&path, "id = [").expect("write malformed manifest in registered TEMP");
    let (request, mut sink) =
        bounded_request(&path, super::NativePluginDiscoveryRefreshBudget::default());
    let error = metered_candidate_from_manifest_path(&request, &mut sink, path.clone())
        .err()
        .expect("malformed TOML must fail");
    std::fs::remove_file(&path).expect("remove owned malformed manifest");
    assert!(matches!(
        &error,
        super::NativePluginDiscoveryRefreshError::ParseManifest { manifest_path, .. }
            if manifest_path == &path
    ));
    let source = std::error::Error::source(&error)
        .and_then(|source| source.downcast_ref::<toml::de::Error>())
        .expect("actual TOML source");
    assert!(source.span().is_some(), "parser location survives refresh");
    assert_eq!(error, error.clone());
    if let super::NativePluginDiscoveryRefreshError::ParseManifest {
        source: original, ..
    } = &error
    {
        let super::NativePluginDiscoveryRefreshError::ParseManifest { source: cloned, .. } =
            error.clone()
        else {
            panic!("clone preserves typed variant");
        };
        assert!(Arc::ptr_eq(original, &cloned));
    }
}

#[test]
fn metered_candidate_preserves_cancellation_and_budget_precedence_over_parse() {
    use super::{
        NativePluginDiscoveryRefreshBudget as Budget,
        NativePluginDiscoveryRefreshBudgetKind as Kind, NativePluginDiscoveryRefreshError as Error,
    };
    use crate::plugin::native_plugin_loader::candidate_from_manifest::metered_candidate_from_manifest_path;
    let path = typed_source_fixture("admission");
    std::fs::write(&path, "id = [").expect("write malformed manifest in registered TEMP");
    let (request, mut sink) = bounded_request(&path, Budget::default());
    request.cancellation.cancel();
    assert!(matches!(
        metered_candidate_from_manifest_path(&request, &mut sink, path.clone()),
        Err(Error::Cancelled)
    ));
    for (kind, budget) in [
        (
            Kind::CandidateCount,
            Budget {
                max_candidates: 0,
                ..Budget::default()
            },
        ),
        (
            Kind::ReadBytes,
            Budget {
                max_read_bytes: 0,
                ..Budget::default()
            },
        ),
        (
            Kind::ScratchBytes,
            Budget {
                max_scratch_bytes: 0,
                ..Budget::default()
            },
        ),
    ] {
        let (request, mut sink) = bounded_request(&path, budget);
        assert!(matches!(
            metered_candidate_from_manifest_path(&request, &mut sink, path.clone()),
            Err(Error::BudgetExceeded { kind: actual, .. }) if actual == kind
        ));
    }
    std::fs::remove_file(&path).expect("remove owned malformed manifest");
}
