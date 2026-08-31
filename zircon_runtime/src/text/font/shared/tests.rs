use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

use glyphon::{Attrs, Buffer, Metrics, Shaping};

use super::*;
use crate::asset::{AssetUri, FontAsset, FontAssetSourceFormat, FontBlobArtifact};
use crate::text::font::source_manifest::cooked_font_asset_source_key;
use crate::text::font::test_font_fixtures::unique_font_fixture_path;
use crate::text::{CompositeFontDescriptor, FontFamilyName, FontQuery};

mod asset_claims;

#[cfg(target_os = "windows")]
#[test]
fn runtime_shared_font_database_defers_system_font_discovery_to_its_consumer() {
    let mut database = runtime_default_font_database();

    assert!(
        database.apply_system_font_policy(SystemFontPolicy::Discover) > 0,
        "the shared bootstrap must not enumerate system fonts before a consumer explicitly opts in"
    );
}

#[test]
fn runtime_shared_font_database_bootstrap_cannot_reopen_checkout_font_sources() {
    let source = include_str!("../shared.rs");
    let bootstrap = source
        .split_once("fn runtime_default_font_database() -> FontDatabase {")
        .expect("shared font bootstrap must remain present")
        .1
        .split_once("\nfn process_font_collection_service()")
        .expect("shared font bootstrap must remain bounded by the process collection owner")
        .0;

    assert!(bootstrap.contains("include_str!"));
    assert!(bootstrap.contains("include_bytes!"));
    assert!(bootstrap.contains("replace_font_asset_blob"));
    assert!(bootstrap.contains("cooked_font_asset_source_key"));
    for forbidden in [
        "std::fs::",
        "File::open",
        "OpenOptions",
        "read_to_string",
        "read_to_end",
        "replace_font_asset(",
    ] {
        assert!(
            !bootstrap.contains(forbidden),
            "the packaged bootstrap must not reopen checkout font sources through {forbidden}"
        );
    }
}

#[test]
fn runtime_shared_font_database_bootstraps_embedded_cooked_default_and_private_alias() {
    let mut database = runtime_default_font_database();
    let face = database
        .font_asset_primary_face(PACKAGED_DEFAULT_FONT_OWNER)
        .expect("checked-in packaged default font face");
    assert_eq!(
        database
            .match_face(&FontQuery::single_family(""))
            .map(|matched| matched.face),
        Some(face),
        "a clean runtime must resolve an unspecified family without system fonts"
    );
    assert_eq!(
        database.runtime_last_resort_face(),
        Some(face),
        "the packaged face must remain the engine-owned last-resort raster source"
    );
    assert_eq!(
        database.default_ui_family_for_test(),
        Some(PACKAGED_DEFAULT_FONT_FAMILY),
        "headless text must start from the packaged UI family before a project is attached"
    );
    let packaged_han = database
        .fallback_candidates_for_codepoint(
            '界',
            &FontQuery::single_family(PACKAGED_DEFAULT_FONT_FAMILY),
            None,
            Some("zh-Hans-CN"),
        )
        .first()
        .copied()
        .expect("the packaged composite must select its embedded CJK face");
    assert_eq!(
        database
            .face_family_name(packaged_han)
            .as_ref()
            .map(FontFamilyName::as_str),
        Some("Zircon Noto Sans CJK SC Proof")
    );
    let asset =
        FontAsset::from_toml_str(include_str!("../../../../assets/fonts/default.font.toml"))
            .expect("parse embedded runtime default font manifest");
    let blob = FontBlobArtifact::from_decoded_bytes(
        FontAssetSourceFormat::TrueTypeCollection,
        include_bytes!("../../../../assets/fonts/ZirconDefaultComposite-subset.ttc").to_vec(),
    );
    assert_eq!(
        blob.source_format(),
        FontAssetSourceFormat::TrueTypeCollection
    );
    assert_eq!(blob.content_hash(), *blake3::hash(blob.bytes()).as_bytes());
    assert!(blob.has_valid_content_hash());
    let source_uri = AssetUri::parse(PACKAGED_DEFAULT_FONT_ASSET_URI)
        .expect("the packaged default font URI must remain valid");
    let source_path = cooked_font_asset_source_key(&source_uri);
    assert_eq!(
        source_path,
        Path::new("cooked-font").join("fonts/default.font.toml"),
        "the bootstrap and project asset loader must share the same cooked source identity"
    );
    let before_attach = database.face_count();
    assert_eq!(
        before_attach, 2,
        "the Runtime bootstrap must preload exactly the manifest's Fira Mono and CJK faces"
    );
    let attached = database
        .replace_font_asset_blob("res://fonts/default.font.toml", &asset, &source_path, &blob)
        .expect("attach default UI asset owner to packaged faces");

    assert_eq!(
        database
            .face_family_name(face)
            .as_ref()
            .map(FontFamilyName::as_str),
        Some(PACKAGED_DEFAULT_FONT_FAMILY)
    );
    assert!(attached.asset_mapping_changed);
    assert!(
        attached.database_changed,
        "a newly selectable UI font object must publish a new render-input generation"
    );
    assert_eq!(attached.faces.len(), 2);
    assert_eq!(attached.faces[0], face);
    assert_eq!(
        database
            .face_family_name(attached.faces[0])
            .as_ref()
            .map(FontFamilyName::as_str),
        Some(PACKAGED_DEFAULT_FONT_FAMILY)
    );
    assert_eq!(
        database
            .face_family_name(attached.faces[1])
            .as_ref()
            .map(FontFamilyName::as_str),
        Some("Zircon Noto Sans CJK SC Proof")
    );
    assert_eq!(
        database.font_asset_primary_face("res://fonts/default.font.toml"),
        Some(face)
    );
    assert_eq!(database.face_count(), before_attach);

    let removed = database.remove_font_asset("res://fonts/default.font.toml");

    assert!(removed.asset_mapping_changed);
    assert!(
        removed.database_changed,
        "removing a selectable UI font object must publish a new render-input generation"
    );
    assert!(removed.retired_faces.is_empty());
    assert_eq!(database.face_count(), before_attach);
    assert_eq!(
        database.font_asset_primary_face(PACKAGED_DEFAULT_FONT_OWNER),
        Some(face)
    );
    assert_eq!(
        database
            .match_face(&FontQuery::single_family(PACKAGED_RUNTIME_FALLBACK_FAMILY))
            .map(|matched| matched.face),
        Some(face),
        "the retained fallback identity must resolve the packaged TTC face, not a same-named system face"
    );
    let backend = database.backend_database_snapshot();
    let families = [glyphon::fontdb::Family::Name(
        PACKAGED_RUNTIME_FALLBACK_FAMILY,
    )];
    let backend_face = backend
        .query(&glyphon::fontdb::Query {
            families: &families,
            ..glyphon::fontdb::Query::default()
        })
        .expect("the runtime fallback identity must be queryable by glyphon");
    assert_eq!(database.font_face_id(backend_face), Some(face));

    let mut font_system = glyphon::FontSystem::new_with_locale_and_db(
        "en-us".to_string(),
        database.backend_database_snapshot(),
    );
    let mut buffer = Buffer::new(&mut font_system, Metrics::new(16.0, 20.0));
    let attrs = Attrs::new().family(glyphon::Family::Name(PACKAGED_RUNTIME_FALLBACK_FAMILY));
    buffer.set_text(
        &mut font_system,
        "Fallback",
        &attrs,
        Shaping::Advanced,
        None,
    );
    buffer.shape_until_scroll(&mut font_system, false);
    let shaped_backend_face = buffer
        .layout_runs()
        .flat_map(|run| run.glyphs.iter())
        .next()
        .expect("ASCII fallback text must shape a glyph")
        .font_id;
    assert_eq!(database.font_face_id(shaped_backend_face), Some(face));

    assert!(database.set_default_ui_family("Project UI"));
    assert_eq!(database.default_ui_family_for_test(), Some("Project UI"));
    assert!(database.clear_default_ui_family());
    assert_eq!(
        database.default_ui_family_for_test(),
        Some(PACKAGED_DEFAULT_FONT_FAMILY),
        "removing the project override must restore the packaged UI family"
    );
}

#[cfg(feature = "profiling")]
#[test]
fn shared_font_database_profiles_snapshot_and_mutation_clone_boundaries() {
    let _capture_guard = crate::core::diagnostics::profiling::test_capture_lock();
    let shared = FontCollectionService::from_database(runtime_default_font_database());
    let mut config = crate::core::diagnostics::profiling::ProfileCaptureConfig::default();
    config.session_id = "text01-shared-font-database-boundaries".to_owned();
    config.max_spans = 4;
    config.max_counters = 8;
    assert!(crate::core::diagnostics::profiling::start_capture(config).active);

    let _ = shared.snapshot();
    let _ = shared.mutate(|_| ());

    let profile = crate::core::diagnostics::profiling::snapshot();
    assert!(!crate::core::diagnostics::profiling::reset_capture().active);
    for name in [
        "shared_snapshot",
        "shared_owned_snapshot_clone",
        "shared_mutation",
        "shared_mutation_outer_database_clone",
        "shared_owned_mutation_result_clone",
    ] {
        assert!(
            profile.spans.iter().any(|span| {
                span.stream == "runtime"
                    && span.category == "text.font_database"
                    && span.name == name
            }),
            "shared font database must expose the {name} publication boundary to profiling"
        );
    }
    for name in [
        "text.font_database.snapshot_face_count",
        "text.font_database.mutation_before_face_count",
        "text.font_database.mutation_after_face_count",
        "text.font_database.mutation_render_inputs_changed",
        "text.font_database.mutation_outer_clone_face_count",
        "text.font_database.mutation_result_clone_face_count",
    ] {
        assert!(
            profile
                .counters
                .iter()
                .any(|counter| counter.stream == "runtime" && counter.name == name),
            "shared font database must emit {name}"
        );
    }
}

#[test]
fn font_collection_snapshot_reuses_the_published_database_arc() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());

    let first = shared.collection_snapshot();
    let second = shared.collection_snapshot();
    let independent = FontCollectionService::from_database(FontDatabase::with_default_fallbacks())
        .collection_snapshot();

    assert_eq!(first.generation(), second.generation());
    assert_ne!(first.collection_id(), independent.collection_id());
    assert!(
        first.shares_database_with(&second),
        "read-only snapshot acquisition must retain the publication Arc instead of cloning the database"
    );
}

#[test]
fn font_collection_snapshot_keeps_the_retired_generation_alive() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());
    let retired = shared.collection_snapshot();
    assert_eq!(retired.database().default_ui_family_for_test(), None);

    let (published_generation, _, changed) =
        shared.mutate(|database| database.set_default_ui_family("Project UI"));
    let published = shared.collection_snapshot();

    assert!(changed);
    assert_eq!(published.generation(), published_generation);
    assert!(published.generation() > retired.generation());
    assert!(!published.shares_database_with(&retired));
    assert_eq!(
        retired.database().default_ui_family_for_test(),
        None,
        "an in-flight shaping lease must keep reading its exact retired database"
    );
    assert_eq!(
        published.database().default_ui_family_for_test(),
        Some("Project UI")
    );
}

#[test]
fn mutation_published_snapshot_reuses_the_published_database_arc() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());

    let (published, changed) = shared.mutate_published_snapshot(|database| {
        database.set_default_ui_family("Published Snapshot Family")
    });
    let current = shared.collection_snapshot();

    assert!(changed);
    assert_eq!(published.generation(), current.generation());
    assert!(
        published.shares_database_with(&current),
        "mutation result must lease the published Arc instead of cloning the complete database"
    );
    assert_eq!(
        published.database().default_ui_family_for_test(),
        Some("Published Snapshot Family")
    );
}

#[test]
fn identical_shared_font_database_publish_preserves_generation() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (first, _, ()) = shared.mutate(|_| ());
    let (second, _, ()) = shared.mutate(|_| ());

    assert_eq!(first, generation);
    assert_eq!(second, generation);
}

#[test]
fn changed_shared_font_database_publish_advances_generation_once() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (changed, _, did_change) = shared.mutate(|database| {
        let changed = database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Project UI"),
            sub_fonts: Vec::new(),
        }));
        changed
    });
    let (unchanged, _, did_change_again) = shared.mutate(|database| {
        let changed = database.set_project_composite_font(Some(CompositeFontDescriptor {
            default_family: FontFamilyName::from("Project UI"),
            sub_fonts: Vec::new(),
        }));
        changed
    });

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
    assert!(did_change);
    assert!(!did_change_again);
}

#[test]
fn changed_default_ui_family_advances_shared_font_generation_once() {
    let shared = FontCollectionService::from_database(FontDatabase::with_default_fallbacks());
    let (generation, _) = shared.snapshot();

    let (changed, _, did_change) = shared.mutate(|database| {
        let changed = database.set_default_ui_family("Project UI");
        changed
    });
    let (unchanged, _, did_change_again) = shared.mutate(|database| {
        let changed = database.set_default_ui_family("Project UI");
        changed
    });

    assert!(changed > generation);
    assert_eq!(unchanged, changed);
    assert!(did_change);
    assert!(!did_change_again);
}

#[test]
fn shared_font_asset_mutations_preserve_independent_owners() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf");
    let destination = unique_font_fixture_path("shared-font-revision", "ttf");
    std::fs::copy(source, &destination).expect("copy shared font fixture");
    let first_owner = "res://fonts/shared-first.font.toml";
    let second_owner = "res://fonts/shared-second.font.toml";
    let shared = FontCollectionService::from_database(FontDatabase::default());
    let (initial_generation, _) = shared.snapshot();
    let (first_generation, _, first) = shared.mutate(|database| {
        let report = database
            .replace_font_source(first_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("register first shared owner");
        report
    });
    let (second_generation, _, attached) = shared.mutate(|database| {
        let report = database
            .replace_font_source(second_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("attach independent shared owner");
        report
    });

    assert!(first.database_changed);
    assert!(first.asset_mapping_changed);
    assert!(first_generation > initial_generation);
    assert!(attached.database_changed);
    assert!(attached.asset_mapping_changed);
    assert!(second_generation > first_generation);

    let (removal_generation, remaining, first_owner_removed) = shared.mutate(|database| {
        let report = database.remove_font_asset(first_owner);
        report
    });

    assert!(first_owner_removed.database_changed);
    assert!(first_owner_removed.asset_mapping_changed);
    assert!(removal_generation > second_generation);
    assert_eq!(remaining.face_count(), 1);

    OpenOptions::new()
        .append(true)
        .open(&destination)
        .expect("open shared fixture for revision")
        .write_all(&[0])
        .expect("append shared revision byte");
    let (replacement_generation, _, replacement) = shared.mutate(|database| {
        let report = database
            .replace_font_source(second_owner, &destination, Some("Shared Reloadable"), 0)
            .expect("replace remaining shared owner revision");
        report
    });
    let (final_generation, final_database, removed) = shared.mutate(|database| {
        let report = database.remove_font_asset(second_owner);
        report
    });

    assert!(replacement.database_changed);
    assert!(replacement.asset_mapping_changed);
    assert!(replacement_generation > removal_generation);
    assert!(removed.database_changed);
    assert!(removed.asset_mapping_changed);
    assert!(final_generation > replacement_generation);
    assert_eq!(final_database.face_count(), 0);

    let _ = std::fs::remove_file(destination);
}
