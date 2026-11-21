use std::{
    env, fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use zircon_runtime::{
    asset::{
        assets::SceneAsset,
        pipeline::manager::ProjectGenerationMatch,
        project::{ProjectManager, ProjectManifest, ProjectPaths},
        AssetUri, ImportedAsset, ProjectAssetManager,
    },
    core::{framework::render::RenderCameraClearColor, math::Vec4, resource::ResourceState},
};

pub(super) const CAMERA: u64 = 1;
pub(super) const CARRIER: u64 = 2;
pub(super) const LABEL: &str = "HUD probe 212";
pub(super) const IMAGE_LABEL: &str = "HUD two-tone swatch 212";
const SCENE_URI: &str = "res://main.scene.toml";
const HUD_URI: &str = "res://hud.zui";
const IMAGE_URI: &str = "res://swatch.ppm";
const IMPORTER: &str = "ui_document_importer.zui_document";
static NEXT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Hud {
    None,
    Text,
    TextAndImage,
}

pub(super) struct Project {
    root: PathBuf,
    keep: bool,
}

impl Project {
    pub fn new(tag: &str, hud: Hud) -> Self {
        let owned = owned_run_root(tag);
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = owned.join(format!(
            "hud-{}-{stamp:x}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).expect("exclusive fixture subdirectory");
        let fixture = Self { root, keep: false };
        fixture.write(hud);
        fixture
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn preserve(&mut self) {
        self.keep = true;
    }

    fn write(&self, hud: Hud) {
        let paths = ProjectPaths::from_root(&self.root).expect("physical fixture project paths");
        let mut manifest = ProjectManifest::new("HUD public fixture", uri(SCENE_URI), 1);
        paths
            .ensure_layout(&manifest.asset_roots)
            .expect("fixture layout");
        let assets = paths.asset_root(&manifest.asset_roots[0]);
        let mut scene =
            SceneAsset::from_project_toml_str(SCENE, |_| unreachable!("no scene dependencies"))
                .expect("valid serialized fixture scene");
        scene.entities[0]
            .camera
            .as_mut()
            .expect("authored camera")
            .clear_color = RenderCameraClearColor::Color(Vec4::new(0.035, 0.09, 0.19, 1.0));
        fs::write(
            assets.join("main.scene.toml"),
            scene
                .to_project_toml_string(|_| unreachable!("no scene dependencies"))
                .expect("serialize fixture scene"),
        )
        .expect("write fixture scene");
        match hud {
            Hud::None => {}
            Hud::Text => {
                manifest.ui_roots.push(uri(HUD_URI));
                fs::write(assets.join("hud.zui"), CPU_ZUI).expect("write CPU .zui");
            }
            Hud::TextAndImage => {
                manifest.ui_roots.push(uri(HUD_URI));
                fs::write(assets.join("swatch.ppm"), swatch_source())
                    .expect("write PPM source, not an artifact");
                fs::write(assets.join("hud.zui"), GPU_ZUI).expect("write GPU .zui");
            }
        }
        manifest
            .save(paths.manifest_path())
            .expect("save ordinary project manifest");
    }
}

impl Drop for Project {
    fn drop(&mut self) {
        if !self.keep && fs::canonicalize(&self.root).ok().as_ref() == Some(&self.root) {
            if let Err(error) = fs::remove_dir_all(&self.root) {
                eprintln!("hud-public fixture cleanup {:?}: {error}", self.root);
            }
        }
    }
}

fn owned_run_root(tag: &str) -> PathBuf {
    let from_env = |name: &str| {
        let path = PathBuf::from(env::var_os(name).unwrap_or_else(|| {
            panic!("{tag} fixture precondition: set {name} in the isolated child process")
        }));
        assert!(
            path.is_absolute(),
            "{tag} fixture precondition: {name} must be absolute"
        );
        path
    };
    let root =
        fs::canonicalize(from_env("ZIRCON_HUD_TEST_ROOT")).expect("resolve owned fixture run root");
    assert!(root.is_dir());
    #[cfg(windows)]
    {
        let display = ProjectPaths::display_path(&root)
            .to_string_lossy()
            .to_ascii_lowercase();
        assert!([r"d:\cargo-targets\", r"e:\cargo-targets\", r"f:\cargo-targets\"]
            .into_iter().any(|prefix| display.starts_with(prefix)), "{tag} fixture precondition: physical run root is not below drive-root cargo-targets: {display}");
    }
    let check_directory = |path: &Path| {
        let actual = fs::canonicalize(path).expect("resolve existing isolated directory");
        assert!(
            actual.is_dir() && actual.starts_with(&root),
            "{tag} fixture precondition: {actual:?} is outside {root:?}"
        );
        actual
    };
    check_directory(&env::temp_dir());
    let log = check_directory(&from_env("ZIRCON_LOG_ROOT"));
    let config = from_env("ZIRCON_CONFIG_PATH");
    check_directory(config.parent().expect("configuration file parent"));
    match fs::symlink_metadata(&config) {
        Ok(meta) => {
            assert!(
                meta.is_file()
                    && fs::canonicalize(&config)
                        .expect("configuration file identity")
                        .starts_with(&root)
            );
        }
        Err(error) => assert_eq!(error.kind(), std::io::ErrorKind::NotFound),
    }
    // Verify the chosen log directory is writable before the engine is allowed to open it;
    // otherwise its platform logger could fall through to the user's log directory.
    let probe = log.join(format!(
        "hud-log-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&probe).expect("create child in owned log root");
    fs::remove_dir(&probe).expect("remove the owned log probe");
    root
}

fn uri(value: &str) -> AssetUri {
    AssetUri::parse(value).expect("canonical fixture URI")
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct ProjectSample {
    root: PathBuf,
    catalog: u64,
    importer_available: bool,
    declared_roots: Vec<String>,
    view: Option<ReadyAsset>,
    texture: Option<ReadyAsset>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ReadyAsset {
    id: String,
    artifact: String,
    revision: u64,
}

impl ProjectSample {
    pub fn read(manager: &ProjectAssetManager) -> Result<Self, String> {
        let (project, token) = manager
            .current_project_generation_snapshot()
            .ok_or("no active project packet")?
            .into_parts();
        let root = project.paths().root().to_path_buf();
        let catalog = project.catalog_input_generation().sequence();
        let source = project
            .primary_project_asset_root()
            .map_err(|e| e.to_string())?;
        let importer = project
            .importer()
            .capability_report_for_source(&source.join("hud.zui"))
            .map_err(|e| e.to_string())?;
        let view = read_asset(
            manager,
            &project,
            HUD_URI,
            IMPORTER,
            |value| matches!(value, ImportedAsset::UiV2View(view) if view.document.asset.id == HUD_URI),
        )?;
        let texture = read_asset(
            manager,
            &project,
            IMAGE_URI,
            "zircon.builtin.texture.image",
            |value| matches!(value, ImportedAsset::Texture(texture) if (texture.width, texture.height) == (32, 16)),
        )?;
        if manager.check_project_generation(&token) != ProjectGenerationMatch::Current {
            return Err("project changed while reading the active artifact packet".into());
        }
        Ok(Self {
            root,
            catalog,
            importer_available: importer.descriptor.id == IMPORTER
                && importer.status.is_available(),
            declared_roots: project
                .manifest()
                .ui_roots
                .iter()
                .map(ToString::to_string)
                .collect(),
            view,
            texture,
        })
    }

    pub fn check(&self, root: &Path, hud: Hud) {
        assert_eq!(
            fs::canonicalize(&self.root).expect("active physical project root"),
            root
        );
        assert!(
            self.catalog != 0 && self.importer_available,
            "linked .zui provider not active: {self:?}"
        );
        if hud == Hud::None {
            assert!(
                self.declared_roots.is_empty() && self.view.is_none(),
                "bad no-root control: {self:?}"
            );
        } else {
            assert_eq!(self.declared_roots.as_slice(), &[HUD_URI.to_owned()]);
            assert!(
                self.view.is_some(),
                "declared root did not import to a Ready view artifact: {self:?}"
            );
        }
        assert_eq!(
            self.texture.is_some(),
            hud == Hud::TextAndImage,
            "wrong texture admission: {self:?}"
        );
    }
}

fn read_asset(
    manager: &ProjectAssetManager,
    project: &ProjectManager,
    locator: &str,
    importer: &str,
    check: impl FnOnce(ImportedAsset) -> bool,
) -> Result<Option<ReadyAsset>, String> {
    let key = uri(locator);
    // The offline project index carries artifact metadata. The live ResourceManager, not that
    // staging index, assigns the published revision. Release the registry guard before I/O.
    let live = manager
        .resource_manager()
        .registry()
        .get_by_locator(&key)
        .cloned();
    let (source, row) = match (project.registry().get_by_locator(&key), live) {
        (None, None) => return Ok(None),
        (Some(source), Some(row)) => (source, row),
        _ => {
            return Err(format!(
                "project and live resource disagree about {locator}"
            ))
        }
    };
    if source.state != ResourceState::Ready
        || row.state != ResourceState::Ready
        || source.importer_id != importer
        || row.importer_id != importer
        || row.revision == 0
        || source.id() != row.id()
        || source.artifact_locator() != row.artifact_locator()
    {
        return Err(format!(
            "wrong Ready registration for {locator}: source={source:?}, live={row:?}"
        ));
    }
    let artifact = source
        .artifact_locator()
        .ok_or_else(|| format!("no persisted artifact for {locator}"))?;
    let loaded = project.load_artifact(&key).map_err(|e| e.to_string())?;
    if !check(loaded) {
        return Err(format!("wrong typed artifact payload for {locator}"));
    }
    Ok(Some(ReadyAsset {
        id: row.id().to_string(),
        artifact: artifact.to_string(),
        revision: row.revision,
    }))
}

fn swatch_source() -> Vec<u8> {
    let mut bytes = b"P6\n32 16\n255\n".to_vec();
    for _ in 0..16 {
        for x in 0..32 {
            bytes.extend_from_slice(if x < 16 {
                &[20, 225, 254]
            } else {
                &[245, 191, 12]
            });
        }
    }
    bytes
}

const SCENE: &str = r#"
[[entities]]
entity = 1
name = "HUD test camera"
mobility = "Dynamic"
[entities.transform]
translation = [0.0, 1.5, -7.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]
[entities.camera]
fov_y_radians = 0.88
z_near = 0.1
z_far = 128.0
hdr = false

[[entities]]
entity = 2
name = "HUD test menu carrier"
[entities.transform]
translation = [0.0, 0.0, 0.0]
rotation = [0.0, 0.0, 0.0, 1.0]
scale = [1.0, 1.0, 1.0]
"#;

const CPU_ZUI: &str = r#"
[asset]
kind = "view"
id = "res://hud.zui"
version = 2
[root]
node = "caption"
[nodes.caption]
component = "Text"
props = { text = "HUD probe 212" }
layout = { input_policy = "Ignore" }
"#;

const GPU_ZUI: &str = r##"
[asset]
kind = "view"
id = "res://hud.zui"
version = 2
[imports]
resources = [{ kind = "image", uri = "res://swatch.ppm" }]
[root]
node = "hud"
[nodes.hud]
component = "Overlay"
layout = { input_policy = "Ignore" }
children = [{ node = "swatch" }, { node = "caption" }]
[nodes.swatch]
component = "Image"
props = { image = "res://swatch.ppm", alt = "HUD two-tone swatch 212" }
layout = { position = { x = 24.0, y = 24.0 }, width = { min = 64.0, preferred = 64.0, max = 64.0, stretch = "Fixed" }, height = { min = 32.0, preferred = 32.0, max = 32.0, stretch = "Fixed" }, input_policy = "Ignore" }
[nodes.caption]
component = "Text"
props = { text = "HUD probe 212", font_size = 28.0, line_height = 35.0, foreground_color = "#00ff30ff" }
layout = { position = { x = 120.0, y = 24.0 }, width = { min = 250.0, preferred = 250.0, max = 250.0, stretch = "Fixed" }, height = { min = 40.0, preferred = 40.0, max = 40.0, stretch = "Fixed" }, input_policy = "Ignore" }
"##;
