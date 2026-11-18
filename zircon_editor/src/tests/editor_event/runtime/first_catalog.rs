use std::{
    fs,
    path::{Path, PathBuf},
};

use zircon_runtime::asset::artifact::ArtifactStore;
use zircon_runtime::asset::pipeline::manager::ProjectAssetGenerationToken;
use zircon_runtime::asset::project::{
    AssetMetaDocument, ProjectCatalogInputGeneration, ProjectCatalogInputRecord,
};
use zircon_runtime::asset::project::{ProjectManager, ProjectPaths};
use zircon_runtime::asset::watch::AssetChangeKind;
use zircon_runtime::asset::{
    asset_manager_handle, project_asset_manager_handle, AssetUri, AssetUuid, ImportedAsset,
};
use zircon_runtime::core::manager::resolve_manager_service;
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::{write_ui_asset_open_route_asset, UI_ASSET_LOCATOR};
use crate::core::{
    asset::EditorAssetIndex,
    logging::{LogChannel, LogFilter, LogSeverity},
    settings::SETTINGS_USER_ROOT_ENV,
};
use crate::tests::editor_event::support::{
    env_lock, EnvironmentVariableRestoreGuard, EventRuntimeHarness,
};
use crate::ui::host::editor_asset_manager::{
    editor_asset_manager_handle, EditorAssetCatalogGeneration, EditorAssetChangeKind,
};
use crate::ui::host::{
    module::EDITOR_MANAGER_NAME, project_access::percent_encode_diagnostic_token, EditorManager,
};

#[cfg(feature = "profiling")]
#[path = "catalog_profile.rs"]
mod catalog_profile;

const STARTER: &str = "res://scenes/main.scene.toml";
const OPEN_MARKER: &str = "editor_project_open ";

#[cfg(feature = "profiling")]
#[test]
#[ignore = "requires an isolated exact single-thread profile run"]
fn editor202_profile_first_catalog_phases() {
    assert!(
        catalog_profile::dedicated_test_process(),
        "[profile not collected] run only the exact ignored editor202_profile_first_catalog_phases test with --exact --ignored --test-threads 1 --show-output"
    );
    editor202_project_open_publishes_initial_catalog_and_log();
}

#[test]
fn editor202_project_open_publishes_initial_catalog_and_log() {
    let _env = env_lock().lock().unwrap();
    let _profile = IsolatedProfile::new();
    let mut harness = EventRuntimeHarness::new("z202");
    let core = harness.core.handle();
    let r = project_asset_manager_handle(&core).unwrap();
    let e = editor_asset_manager_handle(&core).unwrap();
    let runtime = || resolve_manager_service(&core, r.clone()).unwrap();
    let editor = || resolve_manager_service(&core, e.clone()).unwrap();
    let changes = editor().subscribe_editor_asset_changes();
    let drain_editor = || {
        std::iter::from_fn(|| changes.try_recv())
            .take(64)
            .filter(|event| event.change.kind == EditorAssetChangeKind::CatalogChanged)
            .map(|event| event.change.catalog_revision)
            .collect::<Vec<_>>()
    };
    let runtime_changes = resolve_manager_service(&core, asset_manager_handle(&core).unwrap())
        .unwrap()
        .subscribe_asset_changes();
    let logs = harness
        .core
        .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
        .unwrap()
        .context()
        .logs_handle();
    let filter = LogFilter::from_channel(Some(LogChannel::Editor), LogSeverity::Info);
    let floor = logs
        .snapshot_tail(&filter, 1)
        .last()
        .map_or(0, |r| r.sequence());
    let dropped_before = logs.diagnostics().dropped_records;
    assert!(runtime().current_project_generation_snapshot().is_none());
    assert_eq!(editor().catalog_snapshot().catalog_revision, 0);
    let watch_closed = runtime().asset_watch_diagnostics();

    #[cfg(feature = "profiling")]
    let mut phase_capture = None;
    #[cfg(feature = "profiling")]
    let mut open_phase = None;
    let mut ui_source = None;
    let initial = harness.open_project_with_assets("z202p", |assets| {
        write_ui_asset_open_route_asset(assets);
        ui_source = Some(assets.source_path(UI_ASSET_LOCATOR));
        #[cfg(feature = "profiling")]
        {
            // The template and .zui are in place; the registered Editor open follows synchronously.
            phase_capture = catalog_profile::Capture::start(&_profile.root);
            open_phase = phase_capture.as_ref().and_then(|c| c.enter("open"));
        }
    });
    #[cfg(feature = "profiling")]
    drop(open_phase);
    // The opener stores the line before installing the project file sink. Freeze it before refreshing.
    let open_logs = logs
        .snapshot(&filter)
        .into_iter()
        .filter(|r| r.sequence() > floor && r.entry().message().starts_with(OPEN_MARKER))
        .collect::<Vec<_>>();
    let dropped_at_open = logs.diagnostics().dropped_records;
    let opened = runtime()
        .current_project_generation_snapshot()
        .map(|g| g.into_parts());
    let initial_changes = drain_editor();
    let initial_runtime_events = runtime_changes.try_iter().take(32).collect::<Vec<_>>();
    let runtime_pending = runtime_changes.len();
    let watch_before = runtime().asset_watch_diagnostics();
    let old_token = opened.as_ref().map(|(_, token)| token);
    let current_before = old_token.map(|token| runtime().check_project_generation(token));

    #[cfg(feature = "profiling")]
    let refresh_phase = phase_capture
        .as_ref()
        .and_then(|c| c.enter("explicit_refresh"));
    let refresh_result = editor().refresh_from_runtime_project(); // Exactly one explicit refresh.
    #[cfg(feature = "profiling")]
    drop(refresh_phase);
    #[cfg(feature = "profiling")]
    if let Some(capture) = phase_capture.as_mut() {
        capture.stop();
    }
    let current_after = old_token.map(|token| runtime().check_project_generation(token));
    let refreshed = runtime()
        .current_project_generation_snapshot()
        .map(|g| g.into_parts());
    let watch_after = runtime().asset_watch_diagnostics();
    let after = editor().catalog_snapshot();
    let after_changes = drain_editor();

    let runtime_ids = [opened.as_ref(), refreshed.as_ref()].map(|snapshot| {
        snapshot.map(|(p, t)| {
            (
                ProjectPaths::display_path(t.project_root()),
                p.catalog_input_generation().sequence(),
            )
        })
    });
    let same_activation = old_token
        .zip(refreshed.as_ref())
        .map(|(old, (_, new))| old.activation_identity() == new.activation_identity());
    let watch = [watch_closed, watch_before, watch_after].map(|d| {
        (
            d.batch_count,
            d.reconciliation_count,
            d.committed_generation_count,
            d.failed_batch_count,
        )
    });
    eprintln!("editor202 runtime [open,after]={runtime_ids:?} old_token={current_before:?}->{current_after:?} \
        same_activation={same_activation:?} watch [closed,before,after] (batch,reconcile,commit,failed)={watch:?} \
        refresh={refresh_result:?}");
    let added = initial_runtime_events
        .iter()
        .filter_map(|event| event.as_ref().ok())
        .filter(|event| event.kind == AssetChangeKind::Added)
        .map(|event| event.uri.to_string())
        .collect::<Vec<_>>();
    let error = initial_runtime_events
        .iter()
        .find_map(|event| event.as_ref().err());
    eprintln!("editor202 runtime_stream initial_added={added:?} error={error:?} pending={runtime_pending}");
    for (phase, c, revisions) in [
        ("open", &initial, initial_changes),
        ("refresh", &after, after_changes),
    ] {
        eprintln!("editor202 {phase} editor_root={:?} revision={} epoch={} rows={} starter={} zui={} changes={revisions:?}",
            c.project_root, c.catalog_revision, c.publish_epoch, c.assets.len(),
            c.asset_by_locator(STARTER).is_some(), c.asset_by_locator(UI_ASSET_LOCATOR).is_some());
    }
    for record in &open_logs {
        eprintln!(
            "editor202 opener sequence={} {}",
            record.sequence(),
            record.entry().message()
        );
    }

    #[cfg(feature = "profiling")]
    if let Some(capture) = phase_capture {
        capture.report();
    }

    // Report absence before the original fixture expectations; never reopen a project to sample it.
    if opened.is_none() || refreshed.is_none() || ui_source.is_none() {
        eprintln!(
            "editor202 input_delta attribution=not_attributable reason=missing_endpoint before={} after={} fixture={}",
            opened.is_some(), refreshed.is_some(), ui_source.is_some()
        );
    }

    let (project, token) = opened.expect("[fixture input] no Runtime project after open");
    let (latest, latest_token) =
        refreshed.expect("[fixture input] no Runtime project after refresh");
    let ui_source = operation_path(&ui_source.expect("fixture wrote one physical .zui source"));
    let root = ui_source
        .ancestors()
        .nth(3)
        .expect("<root>/assets/ui/menu.zui");
    // Compare exactly the same operation path as the existing assertions, not its display spelling.
    report_catalog_input_delta((&project, &token), (&latest, &latest_token), root);
    assert!(ui_source.is_file(), "[fixture input] missing physical .zui");
    assert!(
        same_activation == Some(true)
            && [
                project.paths().root(),
                token.project_root(),
                latest.paths().root(),
                latest_token.project_root()
            ]
            .iter()
            .all(|p| *p == root),
        "[fixture input] unexpected project root or activation"
    );
    let ui_uuid = inspect_ui_source(&project, &ui_source);
    let starter = exact_starter(&project, root, true);
    let latest_starter = exact_starter(&latest, root, false);
    eprintln!("editor202 fixture starter_open={starter:?} starter_refresh={latest_starter:?}");
    let starter = starter.unwrap_or_else(|why| panic!("[fixture input] {why}"));

    assert_eq!(
        dropped_before, dropped_at_open,
        "[opener capture] log retention lost records"
    );
    let [record] = open_logs.as_slice() else {
        panic!(
            "[opener capture] expected exactly one editor_project_open, got {}",
            open_logs.len()
        );
    };
    let display_root = ProjectPaths::display_path(root);
    let display_root = display_root.to_string_lossy();
    let message = record.entry().message();
    for (field, expected) in [
        ("project_root", display_root.as_ref()),
        ("scene_uri", STARTER),
    ] {
        let encoded = percent_encode_diagnostic_token(expected);
        assert_eq!(
            log_value(message, field),
            Some(encoded.as_str()),
            "[opener capture] wrong {field}"
        );
    }
    assert_catalog("open", &initial, &display_root, starter);
    for field in [
        "project_generation",
        "project_generation_publish_epoch",
        "catalog_asset_count",
    ] {
        let value = log_value(message, field).and_then(|value| value.parse::<u64>().ok());
        assert!(
            value.is_some_and(|n| n > 0),
            "[opener payload] missing or nonpositive {field}"
        );
    }
    refresh_result.expect("[single refresh] Editor service returned an error");
    let latest_starter = latest_starter.unwrap_or_else(|why| panic!("[single refresh] {why}"));
    assert!(
        starter == latest_starter && after.catalog_revision >= initial.catalog_revision,
        "[single refresh] replaced the source GUID or lost a published revision"
    );
    assert_catalog("refresh", &after, &display_root, latest_starter);
    if let Some(uuid) = ui_uuid {
        let row = initial
            .asset_by_locator(UI_ASSET_LOCATOR)
            .expect("[initial .zui] imported UI row missing");
        assert_eq!(
            (row.kind, row.uuid.as_str()),
            (ResourceKind::UiLayout, uuid.to_string().as_str())
        );
    }
}

fn exact_starter(
    project: &ProjectManager,
    root: &Path,
    verify_artifact: bool,
) -> Result<AssetUuid, String> {
    let uri = AssetUri::parse(STARTER).unwrap();
    let expected = operation_path(&root.join("assets/scenes/main.scene.toml"));
    let rr = project
        .registry()
        .get_by_locator(&uri)
        .ok_or("starter missing in Runtime registry")?;
    let input = project.catalog_input_generation();
    let source = input
        .record(rr.id())
        .ok_or("starter missing in Runtime catalog input")?;
    let locator = rr.artifact_locator().ok_or("starter has no artifact")?;
    let resolved = project
        .source_path_for_uri(&uri)
        .map_err(|error| error.to_string())?;
    if !expected.is_file()
        || input.project_root() != root
        || source.source_path() != expected.as_path()
        || resolved != expected
    {
        return Err("starter source or input root differs from the created project".into());
    }
    for rr in [rr, source.resource()] {
        if rr.primary_locator() != &uri
            || rr.kind != ResourceKind::Scene
            || rr.state != ResourceState::Ready
        {
            return Err(format!(
                "starter is not exact Scene/Ready: {:?}/{:?}",
                rr.kind, rr.state
            ));
        }
    }
    if verify_artifact {
        let inventory = ArtifactStore::default()
            .open_chunk_inventory(project.paths(), locator)
            .map_err(|e| e.to_string())?;
        if inventory.raw_bytes() > 1024 * 1024 {
            return Err("starter artifact exceeds the fixture size budget".into());
        }
        let artifact = project.load_artifact(&uri).map_err(|e| e.to_string())?;
        if !matches!(artifact, ImportedAsset::Scene(_)) {
            return Err("starter payload is not Scene".into());
        }
        eprintln!(
            "editor202 fixture starter_artifact_decoded=Scene raw_bytes={}",
            inventory.raw_bytes()
        );
    }
    // Same immutable Runtime input, no second direct sync or import.
    let direct = EditorAssetIndex::from_runtime_project(project).map_err(|e| e.to_string())?;
    let row = direct
        .row_by_path(&uri)
        .ok_or("starter absent from direct index")?;
    if row.type_marker() != ResourceKind::Scene || row.uuid() != source.meta().uuid {
        return Err("starter direct index disagrees with primary input".into());
    }
    Ok(row.uuid())
}

fn inspect_ui_source(project: &ProjectManager, path: &Path) -> Option<AssetUuid> {
    let uri = AssetUri::parse(UI_ASSET_LOCATOR).unwrap();
    let importer = project.importer().capability_report_for_source(path);
    let available = importer
        .as_ref()
        .is_ok_and(|report| report.status.is_available());
    let record = project.registry().get_by_locator(&uri);
    let input = project.catalog_input_generation();
    let source = record.and_then(|record| input.record(record.id()));
    let resolves = project
        .source_path_for_uri(&uri)
        .is_ok_and(|resolved| resolved.as_path() == path);
    let ready = record.is_some_and(|r| {
        r.primary_locator() == &uri
            && r.kind == ResourceKind::UiLayout
            && r.state == ResourceState::Ready
            && r.artifact_locator().is_some()
    });
    let exact =
        source.filter(|s| s.source_path() == path && s.resource().state == ResourceState::Ready);
    let eligible = available && resolves && ready && exact.is_some();
    let outcome = match (eligible, available) {
        (true, _) => "eligible",
        (false, true) => "import_precondition_failed",
        _ => "not_exercised",
    };
    eprintln!("editor202 zui={path:?} resolves={resolves} importer={:?} registry={:?} input_row={} projection={outcome}",
        importer.as_ref().map(|r| (&r.descriptor.id, &r.status)),
        record.map(|r| (r.kind, r.state, r.artifact_locator().is_some())), source.is_some());
    // This headless composition need not install the .zui plugin. Scene is the independent positive control.
    exact.filter(|_| eligible).map(|s| s.meta().uuid)
}

fn assert_catalog(phase: &str, c: &EditorAssetCatalogGeneration, root: &str, uuid: AssetUuid) {
    assert!(
        c.project_root.as_ref() == root && c.catalog_revision > 0 && c.publish_epoch > 0,
        "[{phase} catalog] no initialized generation for the created project"
    );
    let row = c
        .asset_by_locator(STARTER)
        .unwrap_or_else(|| panic!("[{phase} catalog] missing exact starter"));
    assert_eq!(
        (row.kind, row.uuid.as_str()),
        (ResourceKind::Scene, uuid.to_string().as_str())
    );
}

fn log_value<'a>(message: &'a str, field: &str) -> Option<&'a str> {
    let mut values = message
        .strip_prefix(OPEN_MARKER)?
        .split_ascii_whitespace()
        .filter_map(|word| word.split_once('='))
        .filter_map(|(key, value)| (key == field).then_some(value));
    let value = values.next()?;
    values.next().is_none().then_some(value)
}

fn operation_path(path: &Path) -> PathBuf {
    ProjectPaths::resolve_path(path)
        .expect("resolve owned fixture path")
        .into_operation_path()
}

struct IsolatedProfile {
    _restore: Vec<EnvironmentVariableRestoreGuard>,
    root: PathBuf,
}

impl IsolatedProfile {
    fn new() -> Self {
        let root = fs::canonicalize(std::env::temp_dir())
            .expect("fixture TEMP")
            .join(format!("z202-{}", uuid::Uuid::new_v4().simple()));
        fs::create_dir(&root).expect("create exclusive fixture user profile");
        let mut profile = Self {
            _restore: Vec::new(),
            root,
        };
        // Runtime config, editor user settings, and Hub recent history use distinct selectors.
        for key in [
            "USERPROFILE",
            "HOME",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
            SETTINGS_USER_ROOT_ENV,
            "ZIRCON_CONFIG_PATH",
        ] {
            profile
                ._restore
                .push(EnvironmentVariableRestoreGuard::capture(key));
            if key == "ZIRCON_CONFIG_PATH" {
                std::env::set_var(key, profile.root.join("runtime-config.json"));
            } else {
                std::env::set_var(key, &profile.root);
            }
        }
        profile
    }
}

impl Drop for IsolatedProfile {
    fn drop(&mut self) {
        if fs::canonicalize(&self.root).ok().as_deref() == Some(self.root.as_path()) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }
}

fn report_catalog_input_delta(
    (before_project, before_token): (&ProjectManager, &ProjectAssetGenerationToken),
    (after_project, after_token): (&ProjectManager, &ProjectAssetGenerationToken),
    fixture_root: &Path,
) {
    let previous = before_project.catalog_input_generation();
    let current = after_project.catalog_input_generation();
    let roots = [
        before_project.paths().root(),
        before_token.project_root(),
        previous.project_root(),
        after_project.paths().root(),
        after_token.project_root(),
        current.project_root(),
    ];
    let same_root = roots.iter().all(|root| *root == roots[0]);
    let fixture_root_matches = fixture_root == roots[0];
    let same_activation = before_token.activation_identity() == after_token.activation_identity()
        && before_project.activation_identity() == before_token.activation_identity()
        && after_project.activation_identity() == after_token.activation_identity();
    let sequences = (previous.sequence(), current.sequence());
    if !same_root || !fixture_root_matches || !same_activation {
        eprintln!("editor202 input_delta attribution=not_attributable reason=project_identity seq={sequences:?} \
            same_root={same_root} fixture_root_matches={fixture_root_matches} same_activation={same_activation} \
            roots[before_manager,token,input,after_manager,token,input]={roots:?} fixture_root={fixture_root:?}");
        return;
    }

    let delta = current.delta_since(&previous);
    let manifest_changed = previous.manifest() != current.manifest();
    let package_assets_changed = previous.package_assets() != current.package_assets();
    let counts = [
        delta.added.len(),
        delta.removed.len(),
        delta.modified.len(),
        delta.renamed.len(),
    ];
    let exact = catalog_contents_equal(&previous, &current, |a, b| a == b);
    let [ignore_source, ignore_meta, ignore_both] =
        [MtimeSlots::Row, MtimeSlots::Meta, MtimeSlots::Both].map(|slots| {
            catalog_contents_equal(&previous, &current, |a, b| {
                input_equal_except_mtime(a, b, slots)
            })
        });
    // If dropping only timestamps makes the packets equal, the remaining raw differences
    // must be modifications of retained IDs at the same locators, not topology changes.
    let mtime_shape_agrees = !ignore_both
        || counts
            == [
                0,
                0,
                previous
                    .records()
                    .filter(|old| current.record(old.resource().id()) != Some(*old))
                    .count(),
                0,
            ];
    let summary_agrees = exact == delta.is_unchanged()
        && delta.project_metadata_changed == (manifest_changed || package_assets_changed)
        && mtime_shape_agrees;
    let single_mtime_only = match (summary_agrees, exact, ignore_source, ignore_meta) {
        (false, _, _, _) => "unresolved_delta_summary_disagrees",
        (true, true, _, _) => "no_net_change",
        (true, false, true, false) => "source_mtime_unix_ms",
        (true, false, false, true) => "meta.source_mtime_unix_ms",
        _ => "not_a_single_mtime_field",
    };
    let both_mtime_slots_explain_net_diff =
        summary_agrees && !exact && !ignore_source && !ignore_meta && ignore_both;
    eprintln!("editor202 input_delta endpoints=before_after_explicit_refresh_attempt root={:?} \
        same_activation=true seq={sequences:?} counts[added,removed,modified,renamed]={counts:?} \
        project_metadata_changed={} manifest_changed={manifest_changed} package_assets_changed={package_assets_changed}",
        roots[0], delta.project_metadata_changed);
    eprintln!(
        "editor202 input_delta mtime_equal[exact,ignore_row,ignore_meta,ignore_both]={:?} \
        delta_summary_agrees={summary_agrees} single_mtime_only={single_mtime_only} \
        both_mtime_slots_explain_net_diff={both_mtime_slots_explain_net_diff}",
        [exact, ignore_source, ignore_meta, ignore_both]
    );
    report_changed_field("manifest", previous.manifest(), current.manifest());
    report_changed_field(
        "package_assets",
        previous.package_assets(),
        current.package_assets(),
    );
    for (kind, records) in [("added", &delta.added), ("removed", &delta.removed)] {
        for row in records {
            let resource = row.resource();
            eprintln!(
                "editor202 input_delta {kind} id={} locator={} origin={:?} row={row:?}",
                resource.id(),
                resource.primary_locator(),
                input_origin(row)
            );
        }
    }
    for row in &delta.modified {
        if let Some(old) = previous.record(row.resource().id()) {
            report_changed_input("modified", old, row);
        } else {
            eprintln!("editor202 input_delta attribution=not_attributable reason=missing_modified_predecessor id={} locator={}",
                row.resource().id(), row.resource().primary_locator());
        }
    }
    for rename in &delta.renamed {
        report_changed_input("renamed", &rename.previous, &rename.current);
    }
}

// Compare the complete endpoint content by ID, not the rows selected by delta_since.
fn catalog_contents_equal(
    previous: &ProjectCatalogInputGeneration,
    current: &ProjectCatalogInputGeneration,
    same_record: impl Fn(&ProjectCatalogInputRecord, &ProjectCatalogInputRecord) -> bool,
) -> bool {
    previous.project_root() == current.project_root()
        && previous.manifest() == current.manifest()
        && previous.package_assets() == current.package_assets()
        && previous.records().count() == current.records().count()
        && previous.records().all(|old| {
            current
                .record(old.resource().id())
                .is_some_and(|new| same_record(old, new))
        })
}

#[derive(Clone, Copy)]
enum MtimeSlots {
    Row,
    Meta,
    Both,
}

fn input_equal_except_mtime(
    a: &ProjectCatalogInputRecord,
    b: &ProjectCatalogInputRecord,
    ignore: MtimeSlots,
) -> bool {
    // Account for every stored field at the pinned revision except the requested slot(s).
    a.resource() == b.resource()
        && a.source_path() == b.source_path()
        && a.meta_path() == b.meta_path()
        && a.artifact_reference_revision() == b.artifact_reference_revision()
        && a.direct_references() == b.direct_references()
        && a.reference_repairs() == b.reference_repairs()
        && (matches!(ignore, MtimeSlots::Row | MtimeSlots::Both)
            || a.source_mtime_unix_ms() == b.source_mtime_unix_ms())
        && match ignore {
            MtimeSlots::Row => a.meta() == b.meta(),
            MtimeSlots::Meta | MtimeSlots::Both => {
                meta_without_source_mtime(a) == meta_without_source_mtime(b)
            }
        }
}

fn meta_without_source_mtime(row: &ProjectCatalogInputRecord) -> AssetMetaDocument {
    let mut meta = row.meta().clone();
    meta.source_mtime_unix_ms = 0;
    meta
}

fn input_origin(
    row: &ProjectCatalogInputRecord,
) -> (
    zircon_runtime_interface::resource::ResourceScheme,
    Option<&str>,
    &Path,
    &Path,
) {
    let locator = row.resource().primary_locator();
    (
        locator.scheme(),
        locator.package_id(),
        row.source_path(),
        row.meta_path(),
    )
}

fn report_changed_input(
    kind: &str,
    old: &ProjectCatalogInputRecord,
    new: &ProjectCatalogInputRecord,
) {
    let mtime_equal = [MtimeSlots::Row, MtimeSlots::Meta, MtimeSlots::Both]
        .map(|slots| input_equal_except_mtime(old, new, slots));
    eprintln!("editor202 input_delta {kind} id={} locator={} -> {} mtime_equal[ignore_row,ignore_meta,ignore_both]={mtime_equal:?}",
        new.resource().id(), old.resource().primary_locator(), new.resource().primary_locator());
    report_changed_field(
        "source_mtime_unix_ms",
        &old.source_mtime_unix_ms(),
        &new.source_mtime_unix_ms(),
    );
    report_changed_field(
        "meta.source_mtime_unix_ms",
        &old.meta().source_mtime_unix_ms,
        &new.meta().source_mtime_unix_ms,
    );
    report_changed_field("resource", old.resource(), new.resource());
    report_changed_field(
        "reference.direct",
        old.direct_references(),
        new.direct_references(),
    );
    report_changed_field(
        "reference.repairs",
        old.reference_repairs(),
        new.reference_repairs(),
    );
    report_changed_field(
        "reference.artifact_revision",
        &old.artifact_reference_revision(),
        &new.artifact_reference_revision(),
    );
    report_changed_field(
        "origin(scheme,pkg,source_path,meta_path)",
        &input_origin(old),
        &input_origin(new),
    );
    if old.meta() != new.meta() {
        report_changed_field(
            "meta(other; mtime elided)",
            &meta_without_source_mtime(old),
            &meta_without_source_mtime(new),
        );
    }
}

fn report_changed_field<T: std::fmt::Debug + PartialEq + ?Sized>(field: &str, old: &T, new: &T) {
    if old != new {
        eprintln!("editor202 input_delta field={field} before={old:?} after={new:?}");
    }
}
