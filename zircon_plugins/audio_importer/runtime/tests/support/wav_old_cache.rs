use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::UNIX_EPOCH;

use zircon_plugin_audio_importer_runtime::plugin_registration;
use zircon_runtime::asset::artifact::ArtifactStore;
use zircon_runtime::asset::importer::{AssetImportBuildContext, AssetImportRecipe};
use zircon_runtime::asset::project::{
    AssetMetaDocument, PreviewState, ProjectManager, ProjectManifest, ProjectPaths,
};
use zircon_runtime::asset::{
    AssetId, AssetImportContext, AssetImportError, AssetImportOutcome,
    AssetImporterCapabilityStatus, AssetImporterDescriptor, AssetImporterHandler, AssetKind,
    AssetUri,
};
use zircon_runtime::core::resource::{ResourceScheme, ResourceState};

use super::wav_fixture::{create_project, pcm_bits, FIXTURES};
use super::{setup, V1};

const WAV: &str = "audio_importer.wav";
const PRODUCER: &str = "wav156-producer.txt";

#[derive(Debug)]
pub(super) struct HandlerCall {
    uri: AssetUri,
    source_path: PathBuf,
    source_bytes: Vec<u8>,
    recipe: AssetImportRecipe,
    build: Option<AssetImportBuildContext>,
    key: Option<String>,
}

pub(super) type CallTrace = Arc<Mutex<Vec<HandlerCall>>>;

#[derive(Debug)]
struct TracedWav {
    inner: Arc<dyn AssetImporterHandler>,
    descriptor: AssetImporterDescriptor,
    calls: CallTrace,
}

impl AssetImporterHandler for TracedWav {
    fn descriptor(&self) -> &AssetImporterDescriptor {
        &self.descriptor
    }

    fn capability_status(&self) -> AssetImporterCapabilityStatus {
        self.inner.capability_status()
    }

    fn import(&self, context: &AssetImportContext) -> Result<AssetImportOutcome, AssetImportError> {
        self.calls
            .lock()
            .expect("WAV trace lock")
            .push(HandlerCall {
                uri: context.uri.clone(),
                source_path: context.source_path.clone(),
                source_bytes: context.source_bytes.clone(),
                recipe: context.import_recipe().clone(),
                build: context.build_context().cloned(),
                key: context.build_action_key().map(str::to_owned),
            });
        self.inner.import(context)
    }
}

pub(super) fn open(root: &Path, version: u32) -> (ProjectManager, CallTrace) {
    let mut pm = setup(ProjectManager::open(root), "open ProjectManager");
    assert_eq!(pm.paths().root(), root, "setup project root alias");
    let report = plugin_registration();
    assert!(
        report.is_success(),
        "setup audio plug-in registration: {:?}",
        report.diagnostics
    );
    let real = report.extensions.asset_importers();
    let calls = CallTrace::default();
    for inner in real.importers() {
        if inner.descriptor().id == WAV {
            let mut descriptor = inner.descriptor().clone();
            assert_eq!(
                descriptor.importer_version, V1,
                "setup requires the real v1 registration"
            );
            // The control overrides only the public descriptor field on this test delegate.
            descriptor.importer_version = version;
            setup(
                pm.register_asset_importer(TracedWav {
                    inner,
                    descriptor,
                    calls: calls.clone(),
                }),
                "attach WAV delegate",
            );
        } else {
            setup(
                pm.register_asset_importer_arc(inner),
                "attach unmodified sibling handler",
            );
        }
    }

    let assets = setup(
        pm.manifest().primary_asset_root_path(pm.paths()),
        "PM asset root",
    );
    for fixture in &FIXTURES {
        let path = assets.join(fixture.path);
        assert!(
            pm.registry().get_by_locator(&fixture.uri()).is_none(),
            "setup expected an unscanned ProjectManager"
        );
        let original = setup(
            real.capability_report_for_source(&path),
            "select original WAV",
        );
        assert_eq!(original.descriptor.id, WAV, "setup original registration");
        assert_eq!(
            original.descriptor.importer_version, V1,
            "setup original registration"
        );
        assert!(original.status.is_available(), "setup WAV capability");
        let actual = setup(
            pm.importer().capability_report_for_source(&path),
            "select PM WAV",
        );
        eprintln!(
            "wav156 register {} original={} selected={}@{}",
            fixture.path,
            original.descriptor.importer_version,
            actual.descriptor.id,
            actual.descriptor.importer_version
        );
        let mut expected = original;
        expected.descriptor.importer_version = version;
        assert_eq!(
            actual, expected,
            "setup PM selection preserves the rest of the real contract"
        );
    }
    (pm, calls)
}

#[derive(Debug, PartialEq)]
pub(super) struct DiskPair {
    project_manifest: Vec<u8>,
    assets: [DiskSound; 2],
}

#[derive(Debug, PartialEq)]
struct DiskSound {
    source: Vec<u8>,
    mtime_ns: u128,
    meta_path: PathBuf,
    meta_bytes: Vec<u8>,
    meta: AssetMetaDocument,
    artifact_path: PathBuf,
    manifest_bytes: Vec<u8>,
    payload_blake3: String,
    pcm: Vec<u32>,
}

impl DiskPair {
    pub(super) fn assert_distinct_ids(&self, other: &Self) {
        for (a, b) in self.assets.iter().zip(&other.assets) {
            assert_ne!(a.meta.uuid, b.meta.uuid, "independent producer UUIDs");
            assert_ne!(
                a.artifact_path, b.artifact_path,
                "independent physical artifacts"
            );
        }
    }

    fn provenance(&self) -> String {
        use std::fmt::Write;

        let mut text = format!("project_manifest_tag64={}\n", tag(&self.project_manifest));
        for (row, fixture) in self.assets.iter().zip(&FIXTURES) {
            writeln!(text, "uri={} id={} uuid={} importer={}@{} action_key={} settings={:?} fixture_sha256={} asset_source_digest={} mtime_unix_ns={} sidecar={:?} sidecar_tag64={} lib_alias={} manifest={:?} manifest_tag64={} payload_blake3={} pcm={:08x?}",
                row.meta.url, AssetId::from_asset_uuid(row.meta.uuid), row.meta.uuid,
                row.meta.importer_id, row.meta.importer_version, row.meta.config_hash, row.meta.import_settings,
                fixture.sha256, row.meta.source_digest, row.mtime_ns, row.meta_path, tag(&row.meta_bytes),
                row.locator(), row.artifact_path, tag(&row.manifest_bytes), row.payload_blake3, row.pcm,
            ).unwrap();
        }
        text
    }
}

impl DiskSound {
    fn locator(&self) -> &AssetUri {
        self.meta
            .artifact_locator
            .as_ref()
            .expect("Ready Sound artifact locator")
    }
}

pub(super) fn produce_v1(root: &Path, warm: bool) -> DiskPair {
    create_project(root);

    let (mut pm, calls) = open(root, V1);
    let (old, build) = scan(&mut pm, &calls, "first-v1-import", V1, 1);
    let build = build.expect("producer must have called both handlers");
    let receipt = format!("{build}\n{}", old.provenance());
    setup(
        fs::write(root.join(PRODUCER), receipt),
        "write producer witness",
    );
    if warm {
        let (after, _) = scan(&mut pm, &calls, "same-service-v1-full-scan", V1, 0);
        assert_eq!(
            old, after,
            "sampled warm hit must retain old identity and bytes"
        );
    }
    old
}

pub(super) fn producer_basis(root: &Path, before: &DiskPair) -> String {
    let receipt = setup(
        fs::read_to_string(root.join(PRODUCER)),
        "read producer witness",
    );
    let (build, disk) = receipt
        .split_once('\n')
        .expect("setup producer witness format");
    assert_eq!(
        disk,
        before.provenance(),
        "setup child must read the actual producer generation"
    );
    build.into()
}

pub(super) fn capture(root: &Path, stage: &str, version: u32) -> DiskPair {
    let paths = setup(ProjectPaths::from_root(root), "resolve disk project");
    assert_eq!(paths.root(), root, "setup disk project root alias");
    let project = setup(
        ProjectManifest::load(paths.manifest_path()),
        "load disk project manifest",
    );
    let assets = setup(project.primary_asset_root_path(&paths), "disk asset root");
    let store = ArtifactStore::default();
    let pair = DiskPair {
        project_manifest: setup(
            fs::read(paths.manifest_path()),
            "read project manifest bytes",
        ),
        assets: std::array::from_fn(|index| {
            let fixture = &FIXTURES[index];
            let source_path = assets.join(fixture.path);
            let source = setup(fs::read(&source_path), "read WAV bytes");
            assert_eq!(
                source,
                fixture.wav(),
                "setup fixture bytes for {source_path:?}"
            );
            let modified = setup(
                setup(fs::metadata(&source_path), "WAV metadata").modified(),
                "WAV mtime",
            );
            let mtime_ns = setup(modified.duration_since(UNIX_EPOCH), "WAV Unix mtime").as_nanos();

            let meta_path = source_path.with_extension("wav.zmeta");
            let meta_bytes = setup(fs::read(&meta_path), "read .zmeta bytes");
            let meta = setup(AssetMetaDocument::load(&meta_path), "parse producer .zmeta");
            assert_eq!(meta.url, fixture.uri(), "discovered source URI");
            assert_eq!(
                meta.preview_state,
                PreviewState::Ready,
                "{stage} published metadata for {}",
                meta.url
            );
            assert_eq!(meta.asset_kind, AssetKind::Sound);
            assert_eq!(meta.importer_id, WAV);
            assert_eq!(meta.importer_version, version);
            assert!(!meta.source_digest.is_empty());
            assert!(meta.config_hash.starts_with("blake3:"));
            assert_eq!(u128::from(meta.source_mtime_unix_ms), mtime_ns / 1_000_000);
            let locator = meta
                .artifact_locator
                .as_ref()
                .expect("persisted Sound lib:// locator");
            assert_eq!(locator.scheme(), ResourceScheme::Library);
            assert_eq!(locator.label(), None);
            assert!(locator.path().ends_with(".zasset"));
            assert_eq!(meta.entries.len(), 1);
            let entry = &meta.entries[0];
            assert_eq!(entry.url, meta.url);
            assert_eq!(entry.uuid, meta.uuid);
            assert_eq!(entry.asset_kind, AssetKind::Sound);
            assert_eq!(entry.artifact_locator.as_ref(), Some(locator));

            let artifact_path = paths.asset_artifact_root().join(locator.path());
            let manifest_bytes = setup(
                fs::read(&artifact_path),
                "read actual .zasset manifest bytes",
            );
            let inventory = setup(
                store.open_chunk_inventory(&paths, locator),
                "open actual .zasset inventory",
            );
            assert_eq!(inventory.kind(), AssetKind::Sound);
            assert!(!inventory.is_empty());
            eprintln!(
                "wav156 {stage} ArtifactStore::read {locator} for {} at {artifact_path:?}",
                meta.url
            );
            let asset = setup(store.read(&paths, locator), "decode actual disk artifact");
            let pcm = pcm_bits(asset, &meta.url);
            DiskSound {
                source,
                mtime_ns,
                meta_path,
                meta_bytes,
                meta,
                artifact_path,
                manifest_bytes,
                payload_blake3: inventory.content_hash().into(),
                pcm,
            }
        }),
    };
    eprint!(
        "wav156 sampled disk {stage} project={root:?}\n{}",
        pair.provenance()
    );
    for (row, fixture) in pair.assets.iter().zip(&FIXTURES) {
        assert_eq!(
            row.pcm, fixture.bits,
            "{stage} old-build PCM for {}",
            row.meta.url
        );
    }
    assert_ne!(
        pair.assets[0].meta.uuid, pair.assets[1].meta.uuid,
        "two actual resource IDs"
    );
    assert_ne!(
        pair.assets[0].locator(),
        pair.assets[1].locator(),
        "two actual cache locators"
    );
    assert_ne!(pair.assets[0].payload_blake3, pair.assets[1].payload_blake3);
    pair
}

pub(super) fn scan(
    pm: &mut ProjectManager,
    trace: &CallTrace,
    stage: &str,
    version: u32,
    expected_per_uri: usize,
) -> (DiskPair, Option<String>) {
    let result = pm.scan_and_import();
    let calls = std::mem::take(&mut *trace.lock().expect("WAV trace lock"));
    eprintln!("wav156 {stage} scan WAV handler calls={}", calls.len());
    for call in &calls {
        eprintln!(
            "wav156 {stage} handler {} key={:?} recipe={:?} build={:?}",
            call.uri, call.key, call.recipe, call.build
        );
    }
    let records = result.unwrap_or_else(|error| panic!("{stage} scan/commit failed: {error:?}"));
    assert_eq!(
        records.len(),
        FIXTURES.len(),
        "{stage} records: {records:?}"
    );
    for record in &records {
        eprintln!("wav156 {stage} scan record {record:?}");
        assert_eq!(
            record.state,
            ResourceState::Ready,
            "{stage} import or restore outcome"
        );
    }

    let disk = capture(pm.paths().root(), stage, version);
    let mut build = None;
    for row in &disk.assets {
        let uri = &row.meta.url;
        let record = pm
            .registry()
            .get_by_locator(uri)
            .expect("published Sound registry record");
        assert!(records
            .iter()
            .any(|scanned| scanned.id() == record.id() && scanned.primary_locator() == uri));
        assert_eq!(record.state, ResourceState::Ready);
        assert_eq!(record.kind, AssetKind::Sound);
        assert_eq!(record.id(), AssetId::from_asset_uuid(row.meta.uuid));
        assert_eq!(record.artifact_locator(), Some(row.locator()));
        assert_eq!(record.importer_id, row.meta.importer_id);
        assert_eq!(record.importer_version, row.meta.importer_version);
        assert_eq!(record.config_hash, row.meta.config_hash);
        assert_eq!(record.source_hash, row.meta.source_digest);
        let loaded = setup(pm.load_artifact(uri), "load published res:// artifact");
        let pcm = pcm_bits(loaded, uri);
        eprintln!(
            "wav156 {stage} ProjectManager::load_artifact {uri} -> {} pcm={pcm:08x?}",
            row.locator()
        );
        assert_eq!(pcm, row.pcm, "{stage} PM read must match disk read");

        let matching: Vec<_> = calls.iter().filter(|call| &call.uri == uri).collect();
        assert_eq!(
            matching.len(),
            expected_per_uri,
            "{stage} actual delegate calls for {uri}"
        );
        if let [call] = matching.as_slice() {
            assert_eq!(call.source_path.with_extension("wav.zmeta"), row.meta_path);
            assert_eq!(call.source_bytes, row.source);
            assert_eq!(
                call.recipe,
                AssetImportRecipe::from_legacy_settings(row.meta.import_settings.clone())
            );
            assert_eq!(call.key.as_deref(), Some(row.meta.config_hash.as_str()));
            let observed = build_basis(
                call.build
                    .as_ref()
                    .expect("real project import build context"),
                version,
            );
            if let Some(previous) = &build {
                assert_eq!(&observed, previous);
            }
            build = Some(observed);
        }
    }
    assert_eq!(
        calls.len(),
        FIXTURES.len() * expected_per_uri,
        "{stage} total delegate calls"
    );
    (disk, build)
}

pub(super) fn check_version_control(before: &DiskPair, after: &DiskPair) {
    assert_eq!(before.project_manifest, after.project_manifest);
    for (old, new) in before.assets.iter().zip(&after.assets) {
        assert_eq!(old.source, new.source);
        assert_eq!(old.mtime_ns, new.mtime_ns);
        assert_eq!(old.locator(), new.locator());
        assert_eq!(old.artifact_path, new.artifact_path);
        assert_eq!(
            old.pcm, new.pcm,
            "the delegate still runs the old WAV parser"
        );
        assert_ne!(
            old.meta.config_hash, new.meta.config_hash,
            "distinct action keys"
        );
        let mut expected = old.meta.clone();
        expected.importer_version = new.meta.importer_version;
        expected.config_hash.clone_from(&new.meta.config_hash);
        assert_eq!(
            expected, new.meta,
            "version control preserves all other .zmeta fields"
        );
    }
}

fn build_basis(context: &AssetImportBuildContext, version: u32) -> String {
    let (toolchain, build_id) = context
        .toolchain_identity()
        .rsplit_once(";build=")
        .expect("project toolchain build identity");
    let base = toolchain
        .strip_suffix(&format!(";{WAV}@{version}"))
        .expect("project toolchain WAV identity/version");
    assert!(!build_id.is_empty());
    format!(
        "target={:?} profile={:?} abi={} toolchain_base={base:?} build={build_id:?}",
        context.target_platform(),
        context.build_profile(),
        context.engine_abi_version()
    )
}

// Human-readable same-build fingerprint. Assertions also compare the complete manifest bytes.
fn tag(bytes: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    format!("{:016x}", hasher.finish())
}
