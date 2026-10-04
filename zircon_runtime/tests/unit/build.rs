use super::{
    render_runtime_profile_assembly_presets, validate_runtime_profile_presets,
    RuntimeFeaturePresetDocument,
};

fn canonical_document() -> RuntimeFeaturePresetDocument {
    toml::from_str(include_str!("../../runtime-feature-presets.toml"))
        .expect("canonical runtime profile preset document")
}

fn profile_mut<'a>(
    document: &'a mut RuntimeFeaturePresetDocument,
    id: &str,
) -> &'a mut super::RuntimeFeaturePresetRow {
    document
        .profiles
        .iter_mut()
        .find(|profile| profile.id == id)
        .expect("profile")
}

fn module_mut<'a>(
    document: &'a mut RuntimeFeaturePresetDocument,
    id: &str,
) -> &'a mut super::BuiltinRuntimeModuleRow {
    document
        .builtin_modules
        .iter_mut()
        .find(|module| module.id == id)
        .expect("builtin module")
}

#[test]
fn canonical_profile_preset_document_validates() {
    validate_runtime_profile_presets(&canonical_document()).expect("valid canonical document");
}

#[test]
fn schema_rejects_unknown_fields() {
    let source = include_str!("../../runtime-feature-presets.toml").replacen(
        "schema_version = 2",
        "schema_version = 2\nunexpected_field = true",
        1,
    );

    let error = toml::from_str::<RuntimeFeaturePresetDocument>(&source)
        .err()
        .expect("unknown field must be rejected");
    assert!(error.to_string().contains("unknown field"));
}

#[test]
fn validation_rejects_unsupported_schema_and_missing_profile() {
    let mut unsupported = canonical_document();
    unsupported.schema_version = 1;
    assert!(validate_runtime_profile_presets(&unsupported)
        .expect_err("schema 1 must be rejected")
        .contains("unsupported"));

    let mut incomplete = canonical_document();
    incomplete.profiles.pop();
    assert!(validate_runtime_profile_presets(&incomplete)
        .expect_err("missing profile must be rejected")
        .contains("every built-in profile"));
}

#[test]
fn validation_rejects_unknown_and_duplicate_modules() {
    let mut unknown = canonical_document();
    profile_mut(&mut unknown, "minimal")
        .builtin_modules
        .push("missing".to_owned());
    assert!(validate_runtime_profile_presets(&unknown)
        .expect_err("unknown module must be rejected")
        .contains("unknown builtin module"));

    let mut duplicate = canonical_document();
    profile_mut(&mut duplicate, "minimal")
        .builtin_modules
        .push("foundation".to_owned());
    assert!(validate_runtime_profile_presets(&duplicate)
        .expect_err("duplicate module must be rejected")
        .contains("duplicate builtin module"));
}

#[test]
fn validation_rejects_missing_wrong_and_nonlocal_module_feature_gates() {
    let mut missing = canonical_document();
    module_mut(&mut missing, "graphics").required_feature = None;
    assert!(validate_runtime_profile_presets(&missing)
        .expect_err("missing graphics gate must be rejected")
        .contains("feature gate drift"));

    let mut wrong = canonical_document();
    module_mut(&mut wrong, "graphics").required_feature = Some("script".to_owned());
    assert!(validate_runtime_profile_presets(&wrong)
        .expect_err("wrong graphics gate must be rejected")
        .contains("feature gate drift"));

    let mut nonlocal = canonical_document();
    module_mut(&mut nonlocal, "graphics").required_feature = Some("dep:naga".to_owned());
    assert!(validate_runtime_profile_presets(&nonlocal)
        .expect_err("dependency token must not become a module cfg gate")
        .contains("feature gate drift"));
}

#[test]
fn generated_assembly_lookup_exhaustively_matches_profile_ids() {
    let generated = render_runtime_profile_assembly_presets(&canonical_document());

    assert!(generated.contains("match id {"));
    for (index, variant) in ["Minimal", "Client2d", "Client3d", "Editor", "Dev", "Server"]
        .into_iter()
        .enumerate()
    {
        assert!(generated.contains(&format!(
            "RuntimeProfileId::{variant} => &RUNTIME_PROFILE_ASSEMBLY_PRESETS[{index}]"
        )));
    }
}

#[test]
fn validation_rejects_bad_target_and_maturity_enums() {
    let mut bad_target = canonical_document();
    profile_mut(&mut bad_target, "server").target_mode = "service".to_owned();
    assert!(validate_runtime_profile_presets(&bad_target)
        .expect_err("bad target must be rejected")
        .contains("unsupported target mode"));

    let mut bad_maturity = canonical_document();
    profile_mut(&mut bad_maturity, "server").minimum_maturity = "preview".to_owned();
    assert!(validate_runtime_profile_presets(&bad_maturity)
        .expect_err("bad maturity must be rejected")
        .contains("unsupported minimum maturity"));
}

#[test]
fn validation_rejects_default_optional_plugin_overlap() {
    let mut document = canonical_document();
    profile_mut(&mut document, "client2d")
        .optional_plugins
        .push("ui".to_owned());

    assert!(validate_runtime_profile_presets(&document)
        .expect_err("plugin overlap must be rejected")
        .contains("both default and optional"));
}

#[test]
fn validation_rejects_duplicate_and_empty_capabilities() {
    let mut duplicate = canonical_document();
    profile_mut(&mut duplicate, "minimal")
        .required_capabilities
        .push("runtime.core.lifecycle".to_owned());
    assert!(validate_runtime_profile_presets(&duplicate)
        .expect_err("duplicate capability must be rejected")
        .contains("duplicate required capability"));

    let mut empty = canonical_document();
    profile_mut(&mut empty, "minimal").required_capabilities[0].clear();
    assert!(validate_runtime_profile_presets(&empty)
        .expect_err("empty capability must be rejected")
        .contains("invalid canonical"));
}

#[test]
fn validation_requires_cfg_module_features() {
    let mut document = canonical_document();
    profile_mut(&mut document, "client2d")
        .runtime_features
        .retain(|feature| feature != "graphics");

    assert!(validate_runtime_profile_presets(&document)
        .expect_err("cfg module feature must be required")
        .contains("without runtime feature graphics"));
}

#[test]
fn validation_accepts_cargo_weak_dependency_feature_tokens() {
    let canonical = canonical_document();
    validate_runtime_profile_presets(&canonical)
        .expect("the canonical weak dependency feature token must be accepted");

    let mut malformed = canonical_document();
    profile_mut(&mut malformed, "server")
        .runtime_features
        .push("zr_dev_deps_dylib?naga".to_owned());
    assert!(validate_runtime_profile_presets(&malformed)
        .expect_err("a weak dependency token without '/feature' must be rejected")
        .contains("invalid server runtime feature"));
}
