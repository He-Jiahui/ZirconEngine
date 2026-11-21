//! 用真实持久化扫描区分 compound 的生成写回、作者修改与成员修改。

use std::fs::{self, File, FileTimes};
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use zircon_runtime_interface::project::RelPath;

use super::write_include_shader_package;
use crate::asset::project::{
    AssetMetaDocument, AssetSourceUnit, PreviewState, ProjectCatalogInputDelta,
    ProjectCatalogInputGeneration, ProjectCatalogInputRecord, ProjectManager, ProjectManifest,
    ProjectPaths,
};
use crate::asset::safe_project_path::is_link_or_reparse;
use crate::asset::tests::project::unique_temp_project_root;
use crate::asset::{AssetId, AssetUri, AssetUuid, ImportedAsset};
use crate::core::framework::render::ShaderAssetKind;
use crate::core::resource::{io::SourceRevision, ResourceRecord, ResourceState};

const PACKAGE: &str = "stamp_probe";
const IMPORT_PATH: &str = "asset_stamp_fixture::probe";
const SEEDED_MS: u64 = 946_684_800_000;
const OWNER_MARKER: &str = ".compound-source-time-owner";

#[path = "clock_retry.rs"]
mod clock_retry;

struct Fixture {
    root: PathBuf,
    owner: String,
    descriptor: PathBuf,
    members: [PathBuf; 2],
    locators: [AssetUri; 3],
}

impl Fixture {
    fn new(label: &str) -> Self {
        let suggested = unique_temp_project_root(label);
        let output_hint = suggested.parent().unwrap();
        let output = fs::canonicalize(output_hint.parent().unwrap())
            .expect("resolve existing Cargo target directory (set CARGO_TARGET_DIR)")
            .join(output_hint.file_name().unwrap());
        match fs::create_dir(&output) {
            Ok(()) => {}
            Err(error) if error.kind() == ErrorKind::AlreadyExists => {}
            Err(error) => panic!("create test output {}: {error}", output.display()),
        }
        assert!(fs::symlink_metadata(&output).is_ok_and(|m| m.is_dir() && !is_link_or_reparse(&m)));
        assert_eq!(fs::canonicalize(&output).unwrap(), output);

        let root = output.join(suggested.file_name().unwrap());
        // Never adopt or clean an existing case, even if the name happens to match.
        fs::create_dir(&root)
            .unwrap_or_else(|error| panic!("claim test root {}: {error}", root.display()));
        eprintln!("COMPOUND_SOURCE_TIME_FIXTURE root={}", root.display());
        assert_eq!(fs::canonicalize(&root).unwrap(), root);
        let owner = AssetUuid::new().to_string();
        File::create_new(root.join(OWNER_MARKER))
            .unwrap()
            .write_all(owner.as_bytes())
            .unwrap();

        let paths = ProjectPaths::from_root(&root).unwrap();
        let asset_root = RelPath::project_assets();
        paths
            .ensure_layout(std::slice::from_ref(&asset_root))
            .unwrap();
        let uri = write_include_shader_package(&paths, PACKAGE, IMPORT_PATH);
        ProjectManifest::new("Compound Source Time", uri.clone(), 1)
            .save(paths.manifest_path())
            .unwrap();
        let shaders = paths.asset_root(&asset_root).join("shaders");
        let package = shaders.join(PACKAGE);
        let fixture = Self {
            root,
            owner,
            descriptor: shaders.join(format!("{PACKAGE}.zmeta")),
            members: [
                package.join(format!("{PACKAGE}.zshader")),
                package.join(format!("{PACKAGE}.wgsl")),
            ],
            locators: [
                uri.clone(),
                AssetUri::parse(&format!("{uri}#zshader:{PACKAGE}.zshader")).unwrap(),
                AssetUri::parse(&format!("{uri}#wgsl:{PACKAGE}.wgsl")).unwrap(),
            ],
        };
        for path in std::iter::once(&fixture.descriptor).chain(&fixture.members) {
            set_mtime(path, UNIX_EPOCH + Duration::from_millis(SEEDED_MS));
            assert_eq!(
                millis(modified(path)),
                SEEDED_MS,
                "seed: {}",
                path.display()
            );
        }
        fixture
    }

    fn payloads(&self, manager: &ProjectManager) -> [ImportedAsset; 3] {
        self.locators.each_ref().map(|uri| {
            manager
                .load_artifact(uri)
                .unwrap_or_else(|error| panic!("decode {uri}: {error}"))
        })
    }

    fn open_ready(&self) -> (ProjectManager, Scan, [ResourceRecord; 3]) {
        let uuid = AssetMetaDocument::load(&self.descriptor).unwrap().uuid;
        let mut manager = ProjectManager::open(&self.root).unwrap();
        let (scan, records) = self.scan_ready(&mut manager, uuid);
        (manager, scan, records)
    }

    fn scan_ready(
        &self,
        manager: &mut ProjectManager,
        uuid: AssetUuid,
    ) -> (Scan, [ResourceRecord; 3]) {
        manager.scan_and_import().unwrap();
        let scan = Scan::read(self, manager);
        let records = scan.ready(self, manager, uuid);
        (scan, records)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let marker = self.root.join(OWNER_MARKER);
        if std::thread::panicking()
            || fs::canonicalize(&self.root).ok().as_deref() != Some(self.root.as_path())
            || !fs::symlink_metadata(&marker).is_ok_and(|m| m.is_file() && !is_link_or_reparse(&m))
            || fs::read(&marker).ok().as_deref() != Some(self.owner.as_bytes())
        {
            eprintln!(
                "COMPOUND_SOURCE_TIME_FIXTURE retained={}",
                self.root.display()
            );
            return;
        }
        if let Err(error) = fs::remove_dir_all(&self.root) {
            eprintln!("cleanup test root {}: {error}", self.root.display());
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MemberInput {
    modified: SystemTime,
    bytes: Vec<u8>,
}

struct Scan {
    catalog: Arc<ProjectCatalogInputGeneration>,
    meta: AssetMetaDocument,
    descriptor_text: String,
    descriptor_ms: u64,
    members: [MemberInput; 2],
}

impl Scan {
    fn read(fixture: &Fixture, manager: &ProjectManager) -> Self {
        // read_to_string checks UTF-8 but does not normalize the persisted bytes.
        let descriptor_text = fs::read_to_string(&fixture.descriptor).unwrap();
        Self {
            catalog: manager.catalog_input_generation(),
            meta: AssetMetaDocument::from_toml_str(&descriptor_text).unwrap(),
            descriptor_text,
            descriptor_ms: millis(modified(&fixture.descriptor)),
            members: fixture.members.each_ref().map(|path| MemberInput {
                modified: modified(path),
                bytes: fs::read(path).unwrap(),
            }),
        }
    }

    fn row(&self) -> &ProjectCatalogInputRecord {
        self.catalog
            .record(AssetId::from_asset_uuid(self.meta.uuid))
            .expect("root catalog row")
    }

    fn ready(
        &self,
        fixture: &Fixture,
        manager: &ProjectManager,
        uuid: AssetUuid,
    ) -> [ResourceRecord; 3] {
        assert_eq!(
            self.meta.preview_state,
            PreviewState::Ready,
            "{:?}",
            manager.registry()
        );
        assert_eq!(self.meta.uuid, uuid);
        assert_eq!(self.meta.url, fixture.locators[0]);
        assert_eq!(self.meta.unit, AssetSourceUnit::Compound);
        assert_eq!(
            self.meta.importer_id,
            "zircon.builtin.shader.zshader_package"
        );
        assert!(!self.meta.source_digest.is_empty());
        assert!(self.meta.config_hash.starts_with("blake3:"));
        assert_eq!(self.meta.included_files.len(), fixture.members.len());
        for extension in ["zshader", "wgsl"] {
            let member =
                AssetUri::parse(&format!("{}/{PACKAGE}.{extension}", self.meta.url)).unwrap();
            assert!(
                self.meta.included_files.contains(&member),
                "missing member {member}"
            );
        }
        assert_eq!(self.meta.entries.len(), fixture.locators.len());
        assert_eq!(manager.registry().values().count(), fixture.locators.len());
        let records = fixture.locators.each_ref().map(|uri| {
            let record = manager
                .registry()
                .get_by_locator(uri)
                .expect("resource by locator");
            let entry = self
                .meta
                .entries
                .iter()
                .find(|entry| &entry.url == uri)
                .expect("sidecar entry by locator");
            assert_eq!(record.state, ResourceState::Ready, "{record:?}");
            assert!(record.diagnostics.is_empty(), "{record:?}");
            assert_eq!(record.id(), AssetId::from_asset_uuid(entry.uuid), "{uri}");
            let artifact = record.artifact_locator().expect("durable artifact locator");
            assert_eq!(entry.artifact_locator.as_ref(), Some(artifact), "{uri}");
            record.clone()
        });
        let row = self.row();
        assert_eq!(self.catalog.records().count(), 1, "roots only");
        assert_eq!(row.resource(), &records[0]);
        assert_eq!(
            row.resource().artifact_locator(),
            self.meta.artifact_locator.as_ref()
        );
        assert_eq!(row.resource().source_hash, self.meta.source_digest);
        assert_eq!(row.resource().config_hash, self.meta.config_hash);
        assert_eq!(row.source_path(), fixture.descriptor.as_path());
        assert_eq!(row.source_path(), row.meta_path());
        assert_eq!(row.meta(), &self.meta);
        assert_eq!(row.source_mtime_unix_ms(), self.meta.source_mtime_unix_ms);
        records
    }

    fn assert_only_modified_since(&self, previous: &Self) {
        assert_eq!(
            self.catalog.delta_since(&previous.catalog),
            ProjectCatalogInputDelta {
                modified: vec![self.row().clone()],
                ..Default::default()
            }
        );
    }

    fn assert_same_meta_except_mtime(&self, previous: &Self) {
        let mut meta = self.meta.clone();
        meta.source_mtime_unix_ms = previous.meta.source_mtime_unix_ms;
        assert_eq!(meta, previous.meta, "non-mtime descriptor change");
    }
}

fn set_mtime(path: &Path, time: SystemTime) {
    File::options()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(FileTimes::new().set_modified(time))
        .unwrap_or_else(|error| panic!("set mtime {}: {error}", path.display()));
}

fn modified(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .unwrap_or_else(|error| panic!("read mtime {}: {error}", path.display()))
}

fn millis(time: SystemTime) -> u64 {
    u64::try_from(time.duration_since(UNIX_EPOCH).unwrap().as_millis()).unwrap()
}

fn artifact_revision(manager: &ProjectManager, record: &ResourceRecord) -> SourceRevision {
    let locator = record.artifact_locator().expect("root artifact locator");
    let path = manager.paths().asset_artifact_root().join(locator.path());
    let revision = SourceRevision::observe(&path).expect("observe durable artifact manifest");
    assert!(matches!(&revision, SourceRevision::Present(_)), "{locator}");
    revision
}

fn assert_shader_payloads(payloads: &[ImportedAsset; 3], inputs: &[MemberInput; 2]) {
    let [ImportedAsset::Shader(shader), ImportedAsset::Data(zshader), ImportedAsset::Data(wgsl)] =
        payloads
    else {
        panic!("expected a decoded shader and its two data payloads");
    };
    assert_eq!(shader.kind, ShaderAssetKind::Include);
    assert_eq!(shader.import_path.as_deref(), Some(IMPORT_PATH));
    assert!(shader.validation_diagnostics.is_empty(), "{shader:?}");
    // The package importer inserts a separator only *between* WGSL files. There is one here.
    assert_eq!(shader.wgsl_source.as_bytes(), inputs[1].bytes.as_slice());
    assert_eq!(zshader.text.as_bytes(), inputs[0].bytes.as_slice());
    assert_eq!(wgsl.text.as_bytes(), inputs[1].bytes.as_slice());
}

#[test]
fn compound_source_time_second_full_scan_keeps_ready_source_stable() {
    let fixture = Fixture::new("compound_time_stable");
    let mut manager = ProjectManager::open(&fixture.root).expect("open watcher-free project");
    let seed = Scan::read(&fixture, &manager);
    assert_eq!(seed.meta.preview_state, PreviewState::Dirty);
    assert_eq!(seed.descriptor_ms, SEEDED_MS);
    assert!(seed.meta.artifact_locator.is_none() && seed.meta.entries.is_empty());
    assert_eq!(seed.catalog.records().count(), 0);
    let uuid = seed.meta.uuid;

    manager
        .scan_and_import()
        .expect("first full scan must commit");
    let first = Scan::read(&fixture, &manager);
    let records = first.ready(&fixture, &manager, uuid);
    let payloads = fixture.payloads(&manager);
    assert_shader_payloads(&payloads, &seed.members);
    // Pin identity and bytes across scan 2 without denying file replacement on Windows.
    let artifact = artifact_revision(&manager, &records[0]);
    assert_eq!(first.meta.source_mtime_unix_ms, SEEDED_MS);
    assert_eq!(first.meta.tags, seed.meta.tags);
    assert_eq!(first.meta.import_settings, seed.meta.import_settings);
    assert_eq!(first.members, seed.members);
    assert_eq!(
        first.catalog.delta_since(&seed.catalog),
        ProjectCatalogInputDelta {
            added: vec![first.row().clone()],
            ..Default::default()
        }
    );

    // Nothing writes, touches, or reopens the project between the two full scans.
    manager
        .scan_and_import()
        .expect("second full scan must commit");
    let second = Scan::read(&fixture, &manager);
    let delta = second.catalog.delta_since(&first.catalog);
    let same_arc = Arc::ptr_eq(&first.catalog, &second.catalog);
    let row_ms = |scan: &Scan| {
        scan.catalog
            .record(AssetId::from_asset_uuid(uuid))
            .map(|row| row.source_mtime_unix_ms())
    };
    let member_ms = |scan: &Scan| {
        scan.members
            .each_ref()
            .map(|member| millis(member.modified))
    };
    let sidecar_bytes_equal = first.descriptor_text == second.descriptor_text;
    eprintln!(
        "COMPOUND_SOURCE_TIME_V1 seed_ms={SEEDED_MS} descriptor_fs_ms={}->{} \
         member_fs_ms={:?}->{:?} saved_ms={}->{} row_ms={:?}->{:?} \
         sequence={}->{} arc_equal={same_arc} delta={delta:#?}\n\
         sidecar_bytes_equal={sidecar_bytes_equal}\nsidecar_before={:?}\nsidecar_after={:?}",
        first.descriptor_ms,
        second.descriptor_ms,
        member_ms(&first),
        member_ms(&second),
        first.meta.source_mtime_unix_ms,
        second.meta.source_mtime_unix_ms,
        row_ms(&first),
        row_ms(&second),
        first.catalog.sequence(),
        second.catalog.sequence(),
        first.descriptor_text,
        second.descriptor_text,
    );

    assert_eq!(
        second.ready(&fixture, &manager, uuid),
        records,
        "resource records changed"
    );
    assert_eq!(second.members, first.members, "member inputs changed");
    assert_eq!(
        fixture.payloads(&manager),
        payloads,
        "decoded artifacts changed"
    );
    assert_eq!(
        artifact_revision(&manager, &records[0]),
        artifact,
        "artifact manifest was rewritten"
    );
    assert_eq!(
        first.row().artifact_reference_revision(),
        second.row().artifact_reference_revision()
    );
    assert_eq!(
        first.row().direct_references(),
        second.row().direct_references()
    );
    assert_eq!(
        first.row().reference_repairs(),
        second.row().reference_repairs()
    );
    assert!(
        !delta.project_metadata_changed,
        "unrelated project metadata changed"
    );
    second.assert_same_meta_except_mtime(&first);

    // Physical D1/D2 are evidence for attributing a baseline failure, not pass conditions.
    assert!(
        first.meta.source_mtime_unix_ms == second.meta.source_mtime_unix_ms
            && row_ms(&first) == row_ms(&second)
            && sidecar_bytes_equal
            && same_arc
            && first.catalog.sequence() == second.catalog.sequence()
            && delta.is_unchanged(),
        "COMPOUND_SOURCE_TIME_WRITEBACK: unchanged compound source republished after its own writeback"
    );
}

#[test]
fn compound_source_time_rescan_observes_authored_recipe_and_member_bytes() {
    let fixture = Fixture::new("compound_time_authored");
    let (mut manager, first, records) = fixture.open_ready();
    let uuid = first.meta.uuid;
    let original_ids = records.each_ref().map(ResourceRecord::id);
    let artifact = artifact_revision(&manager, &records[0]);

    let mut authored = first.meta.clone();
    authored.tags.insert("authored-tag".to_owned());
    authored.save(&fixture.descriptor).unwrap();
    let (tagged, tagged_records) = fixture.scan_ready(&mut manager, uuid);
    assert_eq!(tagged_records, records);
    assert_eq!(tagged.members, first.members);
    assert_eq!(tagged.meta.tags, authored.tags);
    assert_eq!(tagged.meta.source_digest, first.meta.source_digest);
    assert_eq!(tagged.meta.config_hash, first.meta.config_hash);
    assert_eq!(tagged.meta.import_settings, first.meta.import_settings);
    let root_entry = tagged
        .row()
        .meta()
        .entries
        .iter()
        .find(|entry| entry.url == fixture.locators[0])
        .unwrap();
    assert_eq!(root_entry.tags, authored.tags);
    assert_eq!(
        artifact_revision(&manager, &records[0]),
        artifact,
        "tags must project through restore"
    );
    drop(artifact);
    tagged.assert_only_modified_since(&first);

    // This authored setting participates in the recipe; it need not change shader semantics.
    let mut authored = tagged.meta.clone();
    authored
        .import_settings
        .insert("audit_recipe_setting".to_owned(), toml::Value::Integer(2));
    authored.save(&fixture.descriptor).unwrap();
    let (recipe, recipe_records) = fixture.scan_ready(&mut manager, uuid);
    assert_eq!(
        recipe_records.each_ref().map(ResourceRecord::id),
        original_ids
    );
    assert_eq!(recipe.members, tagged.members);
    assert_eq!(recipe.meta.source_digest, tagged.meta.source_digest);
    assert_ne!(recipe.meta.config_hash, tagged.meta.config_hash);
    assert_eq!(recipe.meta.import_settings, authored.import_settings);
    assert_eq!(recipe.meta.tags, authored.tags);
    recipe.assert_only_modified_since(&tagged);

    // Change only WGSL bytes at the same path and physical mtime, independently of the sidecar.
    let before = std::str::from_utf8(&recipe.members[1].bytes).unwrap();
    let after = before.replacen("return 1.0;", "return 2.0;", 1);
    assert_ne!(
        before, after,
        "include shader fixture must contain the old constant"
    );
    fs::write(&fixture.members[1], &after).unwrap();
    set_mtime(&fixture.members[1], recipe.members[1].modified);
    assert_eq!(modified(&fixture.members[1]), recipe.members[1].modified);
    let (member, member_records) = fixture.scan_ready(&mut manager, uuid);
    assert_eq!(
        member_records.each_ref().map(ResourceRecord::id),
        original_ids
    );
    assert_eq!(member.members[0], recipe.members[0]);
    assert_eq!(member.members[1].modified, recipe.members[1].modified);
    assert_eq!(member.members[1].bytes, after.as_bytes());
    assert_ne!(member.meta.source_digest, recipe.meta.source_digest);
    assert_ne!(member.meta.config_hash, recipe.meta.config_hash);
    assert_eq!(member.meta.import_settings, authored.import_settings);
    assert_eq!(member.meta.tags, authored.tags);
    assert_shader_payloads(&fixture.payloads(&manager), &member.members);
    member.assert_only_modified_since(&recipe);
}

#[test]
fn compound_source_time_observes_member_touch() {
    assert_explicit_touch(Touch::Member);
}

#[test]
fn compound_source_time_observes_explicit_descriptor_touch() {
    assert_explicit_touch(Touch::Descriptor);
}

enum Touch {
    Member,
    Descriptor,
}

fn assert_explicit_touch(target: Touch) {
    let label = match target {
        Touch::Member => "compound_time_member_touch",
        Touch::Descriptor => "compound_time_descriptor_touch",
    };
    let fixture = Fixture::new(label);
    let (mut manager, first, records) = fixture.open_ready();
    let payloads = fixture.payloads(&manager);
    assert_shader_payloads(&payloads, &first.members);
    let path = match target {
        Touch::Member => &fixture.members[1],
        Touch::Descriptor => &fixture.descriptor,
    };

    // An explicit touch beats even the descriptor's natural post-commit timestamp.
    let touched_ms = first
        .members
        .each_ref()
        .map(|member| millis(member.modified))
        .into_iter()
        .chain([first.descriptor_ms, first.meta.source_mtime_unix_ms])
        .max()
        .unwrap()
        .checked_add(60_000)
        .unwrap();
    let bytes = fs::read(path).unwrap();
    set_mtime(path, UNIX_EPOCH + Duration::from_millis(touched_ms));
    assert_eq!(millis(modified(path)), touched_ms);
    assert_eq!(fs::read(path).unwrap(), bytes);

    let (second, after_records) = fixture.scan_ready(&mut manager, first.meta.uuid);
    assert_eq!(after_records, records);
    assert_eq!(fixture.payloads(&manager), payloads);
    match target {
        Touch::Member => {
            assert_eq!(second.members[0], first.members[0]);
            assert_eq!(second.members[1].bytes, bytes);
            assert_eq!(millis(second.members[1].modified), touched_ms);
        }
        Touch::Descriptor => assert_eq!(second.members, first.members),
    }
    second.assert_same_meta_except_mtime(&first);
    assert_eq!(second.meta.source_mtime_unix_ms, touched_ms);
    second.assert_only_modified_since(&first);
}

#[path = "compound_reopen.rs"]
mod reopen;
