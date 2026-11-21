use std::fs::{self, FileTimes, OpenOptions};
use std::path::Path;
use std::time::{Duration, UNIX_EPOCH};

use zircon_runtime::asset::project::{
    AssetMetaDocument, ProjectCatalogInputGeneration, ProjectCatalogInputRecord, ProjectManager,
};
use zircon_runtime::asset::AssetUri;
use zircon_runtime::core::diagnostics::profiling as p;
use zircon_runtime::core::framework::render::ShaderIdePreviewVariant;
use zircon_runtime::core::resource::io::SourceSnapshot;
use zircon_runtime::graphics::write_shader_ide_env_for_project;
use zircon_runtime_interface::resource::{ResourceKind, ResourceState};

use super::{
    catalog_contents_equal, catalog_profile, input_equal_except_mtime, IsolatedProfile, MtimeSlots,
};
use crate::core::project::{NewProjectDraft, ProjectAuthority, ProjectTemplateId};
use crate::tests::{editor_event::support::env_lock, support::test_project_creation_provenance};

const TEST: &str = concat!(
    "tests::editor_event::runtime::animation_assets::first_catalog::catalog_writer::",
    "editor202_compound_writer_control"
);
const STREAM: &str = "editor202_writer_control";
const CALLS: [&str; 3] = ["first_scan", "shader_ide", "no_edit_full_reconcile"];
const FILES: [&str; 3] = [
    "shaders/pbr_shader.zmeta",
    "shaders/pbr_shader/pbr.zshader",
    "shaders/pbr_shader/pbr.wgsl",
];

#[test]
#[ignore = "owned watcher-free diagnostic; run exact, ignored, single thread with output"]
fn editor202_compound_writer_control() {
    assert!(
        catalog_profile::exact_test_process(TEST),
        "run {TEST} --exact --ignored --test-threads 1 --show-output"
    );
    let _env = env_lock().lock().unwrap();
    let fixture = IsolatedProfile::new();
    let created = ProjectAuthority::default()
        .create_project(
            &NewProjectDraft {
                project_name: "EventFixture".to_owned(),
                location: fixture.root.join("w").to_string_lossy().into_owned(),
                template: ProjectTemplateId::RenderableEmpty,
            },
            &test_project_creation_provenance(),
        )
        .expect("create owned RenderableEmpty fixture");
    assert!(
        created.root.starts_with(&fixture.root),
        "fixture outside owned profile"
    );

    // This is the file-generation pipeline, not an active ProjectAssetManager/Editor.
    // Do not start a watcher or mutate a source after establishing the baseline.
    let mut project = ProjectManager::open(&created.root).expect("open fixture ProjectManager");
    let activation = project.activation_identity().clone();
    let assets = project.primary_project_asset_root().unwrap().to_owned();
    let descriptor = assets.join(FILES[0]);
    assert!(assets.starts_with(&created.root));
    for (index, relative) in FILES.iter().enumerate() {
        // Put the setup clocks below the expected replacement clock, without sleeps or retries.
        let clock = UNIX_EPOCH + Duration::from_secs(946_684_800 + 60 * index as u64);
        let path = assets.join(relative);
        OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_times(FileTimes::new().set_modified(clock))
            .unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), clock);
    }
    let initial = project.catalog_input_generation();
    let before = observe_files(&assets, "before_first");
    assert!(p::feature_enabled() && !p::capture_active());
    let session = format!("editor202-writer-control-{}", std::process::id());
    let status = p::start_capture(p::ProfileCaptureConfig {
        session_id: session.clone(),
        output_root: fixture.root.join("profile").to_string_lossy().into_owned(),
        max_spans: p::PROFILE_CAPTURE_MAX_SPANS,
        max_counters: p::PROFILE_CAPTURE_MAX_COUNTERS,
        include_perfetto: false,
        ..Default::default()
    });
    let capture = CaptureGuard;
    assert!(status.active && status.feature_enabled);

    observe_call(CALLS[0], || project.scan_and_import()).expect("first real full generation");
    let first = observe_files(&assets, "after_first");
    let original_generation = project.catalog_input_generation();
    let first_row = shader_row(&original_generation, &descriptor, &first[0]);
    report_journal_endpoint(&project, CALLS[0]);

    let ide = observe_call(CALLS[1], || {
        write_shader_ide_env_for_project(
            &project,
            None,
            &[ShaderIdePreviewVariant::default_forward()],
        )
    })
    .expect("run the same production Shader IDE generator as Editor");
    let cache = Path::new(&ide.module_map);
    eprintln!("{STREAM} ide_report={ide:?}");
    assert!(cache.starts_with(project.paths().cache_root()) && cache.is_file());
    assert!(
        ide.shader_count > 0 && ide.written_file_count > 0,
        "missing IDE positive control"
    );
    let after_ide = observe_files(&assets, "after_ide");
    let after_ide_generation = project.catalog_input_generation();

    // Empty input selects the real full file-generation path. This is not a replay of any #202
    // batch. No manual edit occurs between the two scans.
    observe_call(CALLS[2], || project.scan_and_import_watch_changes(&[]))
        .expect("one real empty-input full reconciliation");
    let reconciled = observe_files(&assets, "after_reconcile");
    let final_generation = project.catalog_input_generation();
    let final_row = shader_row(&final_generation, &descriptor, &reconciled[0]);
    drop(capture);
    report_profile(&session);
    report_journal_endpoint(&project, CALLS[2]);

    assert_eq!(project.activation_identity(), &activation);
    assert!([
        &initial,
        &original_generation,
        &after_ide_generation,
        &final_generation
    ]
    .iter()
    .all(|generation| generation.project_root() == created.root.as_path()));
    assert_eq!(first_row.resource().id(), final_row.resource().id());
    let before_max = before.iter().map(|file| file.mtime_ms).max().unwrap();
    let same =
        |a: &[ObservedFile], b: &[ObservedFile]| a.iter().zip(b).all(|(a, b)| a.equivalent(b));
    let first_writes_descriptor = before[0].source.bytes() != first[0].source.bytes();
    let ide_same_endpoints = same(&first, &after_ide)
        && original_generation.sequence() == after_ide_generation.sequence();
    let members_same = same(&before[1..], &first[1..])
        && same(&first[1..], &after_ide[1..])
        && same(&after_ide[1..], &reconciled[1..]);
    let descriptor_supplies_unique_max = after_ide[0].mtime_ms > before_max
        && after_ide[1..]
            .iter()
            .all(|member| member.mtime_ms < after_ide[0].mtime_ms);
    let reconciled_reads_descriptor_clock =
        descriptor_supplies_unique_max && final_row.source_mtime_unix_ms() == after_ide[0].mtime_ms;
    let reconcile_rewrites_descriptor = after_ide[0].source.bytes() != reconciled[0].source.bytes();
    let delta = final_generation.delta_since(&original_generation);
    let only_shader_double_mtime = !delta.project_metadata_changed
        && delta.added.is_empty()
        && delta.removed.is_empty()
        && delta.renamed.is_empty()
        && matches!(delta.modified.as_slice(), [row] if row.resource().id() == final_row.resource().id())
        && [MtimeSlots::Row, MtimeSlots::Meta, MtimeSlots::Both].map(|slots| {
            catalog_contents_equal(&original_generation, &final_generation, |a, b| {
                input_equal_except_mtime(a, b, slots)
            })
        }) == [false, false, true];
    let sequences = [
        initial.sequence(),
        original_generation.sequence(),
        after_ide_generation.sequence(),
        final_generation.sequence(),
    ];
    let feedback_observed = first_writes_descriptor
        && ide_same_endpoints
        && members_same
        && first_row.source_mtime_unix_ms() == before_max
        && reconciled_reads_descriptor_clock
        && reconcile_rewrites_descriptor
        && only_shader_double_mtime
        && sequences[3] > sequences[2];
    eprintln!("{STREAM} same_activation=true root={:?} shader_id={} seq[before,first,ide,reconcile]={sequences:?} row_clock_ms[first,reconcile]={:?} before_max_ms={before_max} first_writes_descriptor={first_writes_descriptor} ide_same_endpoints={ide_same_endpoints} members_same={members_same} descriptor_supplies_unique_max={descriptor_supplies_unique_max} reconciled_reads_descriptor_clock={reconciled_reads_descriptor_clock} reconcile_rewrites_descriptor={reconcile_rewrites_descriptor} only_shader_double_mtime={only_shader_double_mtime} feedback_observed={feedback_observed}",
        created.root, first_row.resource().id(), [first_row.source_mtime_unix_ms(), final_row.source_mtime_unix_ms()]);
}

struct ObservedFile {
    source: SourceSnapshot,
    modified: std::time::SystemTime,
    mtime_ms: u64,
}

impl ObservedFile {
    fn equivalent(&self, other: &Self) -> bool {
        self.modified == other.modified
            && self.source.revision() == other.source.revision()
            && self.source.bytes() == other.source.bytes()
    }
}

fn observe_files(assets: &Path, phase: &str) -> [ObservedFile; FILES.len()] {
    FILES.map(|relative| {
        let path = assets.join(relative);
        let modified = fs::metadata(&path).unwrap().modified().unwrap();
        // The production revision pins the identity using metadata-only access on Windows.
        let source = SourceSnapshot::read_bounded(&path, 1024 * 1024).unwrap();
        assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified, "unstable endpoint: {path:?}");
        let mtime_ms = u64::try_from(modified.duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap();
        eprintln!("{STREAM} phase={phase} path={relative} modified={modified:?} mtime_ms={mtime_ms} revision={:?}", source.revision());
        ObservedFile { source, modified, mtime_ms }
    })
}

fn shader_row<'a>(
    generation: &'a ProjectCatalogInputGeneration,
    descriptor: &Path,
    disk: &ObservedFile,
) -> &'a ProjectCatalogInputRecord {
    let uri = AssetUri::parse("res://shaders/pbr_shader").unwrap();
    let mut matching = generation
        .records()
        .filter(|row| row.resource().primary_locator() == &uri);
    let row = matching.next().expect("exact compound PBR input row");
    assert!(matching.next().is_none());
    assert_eq!(
        (row.resource().kind, row.resource().state),
        (ResourceKind::Shader, ResourceState::Ready)
    );
    assert_eq!(
        (row.source_path(), row.meta_path()),
        (descriptor, descriptor)
    );
    let disk_meta =
        AssetMetaDocument::from_toml_str(std::str::from_utf8(disk.source.bytes()).unwrap())
            .unwrap();
    assert_eq!(row.meta(), &disk_meta);
    assert_eq!(row.source_mtime_unix_ms(), disk_meta.source_mtime_unix_ms);
    row
}

fn report_journal_endpoint(project: &ProjectManager, phase: &str) {
    let dir = project.paths().derived_root().join("project-generation");
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries
            .take(9)
            .map(|entry| entry.map(|e| e.file_name()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap(),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => panic!("read owned journal endpoint {dir:?}: {error}"),
    };
    eprintln!(
        "{STREAM} phase={phase} journal_endpoint_only={entries:?} may_be_truncated={}",
        entries.len() == 9
    );
}

struct CaptureGuard;

impl Drop for CaptureGuard {
    fn drop(&mut self) {
        p::stop_capture();
    }
}

fn observe_call<T>(name: &'static str, f: impl FnOnce() -> T) -> T {
    let _scope = p::ProfileScope::enter(STREAM, "operation", name);
    f()
}

fn report_profile(session: &str) {
    let s = p::snapshot();
    assert!(!s.active && s.feature_enabled && s.session_id == session);
    let [retention] = s.recorder_retention.as_slice() else {
        panic!("unexpected profile recorder retention")
    };
    let r = &retention.spans;
    assert!(
        r.overwritten == 0
            && r.written == r.retained
            && r.retained == s.spans.len() as u64
            && r.retained < r.capacity
    );
    for (name, expected) in CALLS.into_iter().zip([1, 0, 1]) {
        let mut caller = s.spans.iter().filter(|span| {
            span.stream == STREAM && span.category == "operation" && span.name == name
        });
        let anchor = caller.next().expect("missing scoped operation");
        assert!(anchor.parent_id.is_none() && caller.next().is_none());
        let file_commits = s
            .spans
            .iter()
            .filter(|span| {
                span.parent_id == Some(anchor.id)
                    && span.stream == "asset"
                    && span.category == "project_generation.phase"
                    && span.name == "file_commit"
            })
            .map(|span| (span.id, span.start_us, span.duration_us))
            .take(4)
            .collect::<Vec<_>>();
        eprintln!("{STREAM} phase={name} caller_span={} caller_start_us={} caller_duration_us={} direct_file_commits[id,start_us,duration_us]={file_commits:?}", anchor.id, anchor.start_us, anchor.duration_us);
        assert_eq!(
            file_commits.len(),
            expected,
            "wrong production commit scope for {name}"
        );
    }
}
