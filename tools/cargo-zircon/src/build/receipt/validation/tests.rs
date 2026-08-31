use super::{validate_created_utc, validate_normalized_artifact_closure, validate_relative_path};
use crate::build::receipt::{ArtifactKind, ReceiptArtifact};

#[test]
fn accepts_receipt_timestamp_boundary_values() {
    for value in [
        "0000-01-01T00:00:00Z",
        "2024-02-29T23:59:60Z",
        "2026-08-29T12:34:56.000001Z",
    ] {
        validate_created_utc(value).unwrap();
    }
}

#[test]
fn rejects_malformed_receipt_timestamps_without_panicking() {
    for value in [
        "2023-02-29T00:00:00Z",
        "2026-08-29T12:34:56.Z",
        "2026-08-29T12:34:56+00:00",
        "2026-08-29T12:34:56ZZ",
        "2026-08-29T24:00:00Z",
    ] {
        assert!(validate_created_utc(value).is_err(), "accepted {value}");
    }

    let unicode = format!("000{}{}Z", '\u{00e9}', "0".repeat(14));
    assert_eq!(unicode.len(), 20);
    assert!(validate_created_utc(&unicode).is_err());
}

#[test]
fn validates_artifact_relative_paths_without_host_path_semantics() {
    for value in [
        "bin/zircon_editor.exe",
        "symbols/editor/zircon_editor.pdb",
        "resources/localized-name.txt",
    ] {
        validate_relative_path(value).unwrap();
    }

    for value in [
        "",
        "/bin/editor.exe",
        "bin/editor.exe/",
        "bin//editor.exe",
        r"bin\editor.exe",
        "./bin/editor.exe",
        "bin/../editor.exe",
        "../editor.exe",
        "C:/bin/editor.exe",
        "C:bin/editor.exe",
    ] {
        assert!(validate_relative_path(value).is_err(), "accepted {value}");
    }
}

#[test]
fn fused_normalized_artifact_closure_validates_fields_and_uniqueness() {
    let build_products = [artifact(
        "editor",
        "bin/editor.exe",
        ArtifactKind::Executable,
    )];
    let runtime_dependencies = [artifact(
        "runtime",
        "runtime/editor.dll",
        ArtifactKind::DynamicLibrary,
    )];
    let symbols = [artifact(
        "symbols",
        "symbols/editor.pdb",
        ArtifactKind::SymbolFile,
    )];
    let sbom = artifact("sbom", "sbom/product.spdx.json", ArtifactKind::Sbom);

    assert!(validate_normalized_artifact_closure(
        &build_products,
        &runtime_dependencies,
        &symbols,
        Some(&sbom),
    )
    .unwrap());
}

#[test]
fn fused_normalized_artifact_closure_rejects_duplicate_paths() {
    let build_products = [artifact(
        "editor",
        "bin/editor.exe",
        ArtifactKind::Executable,
    )];
    let runtime_dependencies = [artifact(
        "runtime",
        "bin/editor.exe",
        ArtifactKind::DynamicLibrary,
    )];

    let error =
        validate_normalized_artifact_closure(&build_products, &runtime_dependencies, &[], None)
            .unwrap_err();
    assert!(error
        .to_string()
        .contains("duplicate artifact relative path `bin/editor.exe`"));
}

#[test]
fn fused_normalized_artifact_closure_falls_back_for_unsorted_partition() {
    let build_products = [
        artifact("z-editor", "bin/z-editor.exe", ArtifactKind::Executable),
        artifact("a-editor", "bin/a-editor.exe", ArtifactKind::Executable),
    ];

    assert!(!validate_normalized_artifact_closure(&build_products, &[], &[], None,).unwrap());
}

fn artifact(logical_name: &str, relative_path: &str, kind: ArtifactKind) -> ReceiptArtifact {
    ReceiptArtifact {
        logical_name: logical_name.to_string(),
        relative_path: relative_path.to_string(),
        kind,
        sha256: "A".repeat(64),
        byte_length: 1,
    }
}
