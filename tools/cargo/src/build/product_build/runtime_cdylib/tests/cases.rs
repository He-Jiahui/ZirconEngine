use super::*;
use crate::build::product_build::CargoRuntimeDependencyDeclaration;

fn dependency() -> ResolvedRuntimeDependency {
    ResolvedRuntimeDependency {
        declaration: CargoRuntimeDependencyDeclaration {
            logical_name: "zircon-runtime".into(),
            relative_path: "bin/zircon_runtime.dll".into(),
            package: "zircon_runtime".into(),
            target: "zircon_runtime".into(),
            artifact_file_name: "zircon_runtime.dll".into(),
        },
        package_id: "runtime-id".into(),
    }
}

#[test]
fn runtime_cdylib_arguments_keep_crate_type_on_cargo_side() {
    let request = super::super::tests::test_request();
    let args = build_arguments(
        &request,
        Path::new("Cargo.toml"),
        Path::new("output"),
        &["target-server".into()],
    );
    let args: Vec<_> = args.iter().map(|arg| arg.to_string_lossy()).collect();
    assert_eq!(args[0], "rustc");
    assert_eq!(
        &args[3..9],
        [
            "--package",
            "zircon_runtime",
            "--lib",
            "--crate-type",
            "cdylib",
            "--no-default-features"
        ]
    );
    assert!(!args.iter().any(|arg| arg == "--"));
    assert_eq!(args.last().unwrap(), "target-server");
}

#[test]
fn selects_only_declared_package_cdylib_after_successful_finish() {
    let artifact = r#"{"reason":"compiler-artifact","package_id":"runtime-id","target":{"name":"zircon_runtime","crate_types":["cdylib"]},"filenames":["output/zircon_runtime.dll"]}"#;
    let finish = r#"{"reason":"build-finished","success":true}"#;
    let messages = format!("{artifact}\n{finish}\n");
    assert_eq!(
        select_runtime_artifact(messages.as_bytes(), &dependency()).unwrap(),
        PathBuf::from("output/zircon_runtime.dll")
    );
    for invalid in [
        messages.replace("runtime-id", "other-id"),
        messages.replace("cdylib", "rlib"),
        messages.replace("true", "false"),
        format!("{artifact}\n{artifact}\n{finish}\n"),
        format!("{artifact}\n{finish}\n{finish}\n"),
        format!("{artifact}\n"),
    ] {
        assert!(select_runtime_artifact(invalid.as_bytes(), &dependency()).is_err());
    }
}

#[test]
fn rejects_built_development_feature_before_spawning_cdylib_build() {
    let mut runtime = DeclaredRuntime {
        dependency: dependency(),
        features: None,
    };
    let error = runtime
        .observe_artifact(
            "runtime-id",
            "zircon_runtime",
            false,
            br#"{"features":["target-client","dev-dynamic-linking"]}"#,
        )
        .unwrap_err();
    assert!(error.to_string().contains("development DLL features"));
    assert!(runtime.features.is_none());
}

#[test]
fn runtime_feature_capture_accepts_fresh_artifacts_and_rejects_ambiguity() {
    let mut runtime = DeclaredRuntime {
        dependency: dependency(),
        features: None,
    };
    let message = br#"{"features":["dynamic-api","target-client"],"fresh":true}"#;
    runtime
        .observe_artifact("other-id", "zircon_runtime", false, message)
        .unwrap();
    assert!(runtime.features.is_none());
    runtime
        .observe_artifact("runtime-id", "zircon_runtime", false, message)
        .unwrap();
    assert_eq!(
        runtime.features.as_deref().unwrap(),
        ["dynamic-api", "target-client"]
    );
    assert!(runtime
        .observe_artifact("runtime-id", "zircon_runtime", false, message)
        .is_err());
}

#[test]
fn rejects_runtime_artifacts_without_the_dynamic_api_feature() {
    for features in ["[]", "[\"core-min\"]", "[\"target-server\"]"] {
        let mut runtime = DeclaredRuntime {
            dependency: dependency(),
            features: None,
        };
        let message = format!(r#"{{"features":{features}}}"#);
        let error = runtime
            .observe_artifact("runtime-id", "zircon_runtime", false, message.as_bytes())
            .unwrap_err();
        assert!(error.to_string().contains("dynamic-api"));
        assert!(runtime.features.is_none());
    }
}

#[test]
fn missing_built_features_fail_before_spawning_cargo() {
    let runtime = DeclaredRuntime {
        dependency: dependency(),
        features: None,
    };
    let error = runtime
        .build(
            &super::super::tests::test_request(),
            Path::new("nonexistent-cargo"),
            Path::new("snapshot"),
            Path::new("Cargo.toml"),
            Path::new("output"),
            &[],
        )
        .err()
        .unwrap();
    assert!(error
        .to_string()
        .contains("did not report Runtime features"));
}
