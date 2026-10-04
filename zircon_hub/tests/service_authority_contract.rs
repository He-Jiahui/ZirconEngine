//! Static contracts for the feature-gated native Hub service and broker.

use std::{collections::BTreeSet, fs, path::PathBuf};

fn repo_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_hub should have a repository parent")
        .to_path_buf()
}

fn read(path: &str) -> String {
    fs::read_to_string(repo_dir().join(path))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

#[test]
fn cargo_exposes_independent_desktop_broker_and_local_service_features() {
    let manifest: toml::Table = toml::from_str(&read("zircon_hub/Cargo.toml")).unwrap();
    for (feature, expected) in [
        ("default", &["desktop"][..]),
        (
            "desktop",
            &[
                "dep:tauri",
                "dep:tauri-build",
                "dep:zircon_runtime_interface",
                "account-broker",
            ][..],
        ),
        (
            "account-broker",
            &[
                "dep:openidconnect",
                "dep:tokio",
                "dep:axum",
                "dep:keyring",
                "dep:webbrowser",
                "dep:sha2",
                "dep:libc",
            ][..],
        ),
        (
            "local-service",
            &[
                "dep:axum",
                "dep:tokio",
                "dep:rusqlite",
                "dep:openidconnect",
                "dep:jsonwebtoken",
                "dep:uuid",
                "dep:sha2",
                "dep:aes-gcm",
                "dep:libc",
            ][..],
        ),
    ] {
        let actual: BTreeSet<_> = manifest["features"][feature]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect();
        assert_eq!(
            actual,
            expected.iter().copied().collect::<BTreeSet<_>>(),
            "{feature}"
        );
    }
    let service_binary = manifest["bin"]
        .as_array()
        .unwrap()
        .iter()
        .find(|binary| binary["name"].as_str() == Some("zircon_hub_service"))
        .expect("local service binary must be declared");
    assert_eq!(
        service_binary["required-features"],
        toml::Value::Array(vec![toml::Value::String("local-service".into())])
    );

    let lib = read("zircon_hub/src/lib.rs");
    assert!(
        lib.contains("#[cfg(feature = \"account-broker\")]"),
        "account broker module must remain feature-gated"
    );
    assert!(
        lib.contains("#[cfg(feature = \"local-service\")]"),
        "local service module must remain feature-gated"
    );

    let service_bin = read("zircon_hub/src/bin/zircon_hub_service.rs");
    assert!(
        service_bin.contains("use zircon_hub::service;"),
        "local service binary must consume the library service module"
    );
    assert!(
        !service_bin.contains("#[path = \"../service/mod.rs\"]"),
        "local service binary must not compile a duplicate private service tree"
    );
    assert!(
        service_bin.contains("runtime.block_on(service::run())"),
        "local service binary must delegate through its explicit runtime"
    );
    assert!(
        service_bin.contains("runtime.shutdown_timeout"),
        "local service binary must bound runtime teardown"
    );
    let lifecycle = read("zircon_hub/src/service/lifecycle.rs");
    assert!(
        lifecycle.contains("StartupSupervisor::install_ctrl_c().await?")
            && lifecycle.contains("spawn_blocking"),
        "service startup must install cancellation before synchronous initialization"
    );
}

#[test]
fn service_keeps_origin_rejection_bearer_auth_and_bounded_routes() {
    let router = read("zircon_hub/src/service/http/mod.rs");
    for snippet in [
        "request.headers().contains_key(header::ORIGIN)",
        "strip_prefix(\"Bearer \")",
        "DefaultBodyLimit::max(65536)",
        "Semaphore::new(32)",
        "\"/v1/organizations/{organization}/mutations\"",
        "\"/v1/operations/{operation}\"",
    ] {
        assert!(router.contains(snippet), "service router missing {snippet}");
    }
}

#[test]
fn native_pkce_and_credential_contract_never_projects_token_fields() {
    let oidc = read("zircon_hub/src/account/oidc.rs");
    let callback = read("zircon_hub/src/account/callback.rs");
    let account = read("zircon_hub/src/account/mod.rs");
    for snippet in [
        "PkceCodeChallenge::new_random_sha256()",
        "CsrfToken::new_random",
        "Nonce::new_random",
        "set_pkce_verifier(verifier)",
        "headers.contains_key(header::ORIGIN)",
        "pub struct AccountView",
        "pub generation: String",
    ] {
        assert!(
            oidc.contains(snippet) || callback.contains(snippet) || account.contains(snippet),
            "native broker contract missing {snippet}"
        );
    }
    assert!(!account.contains("refresh_token: String"));
    assert!(!account.contains("access_token: String"));
}
