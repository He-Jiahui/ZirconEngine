use std::ffi::OsStr;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use image::ImageFormat;
use sha2::{Digest, Sha256};

use crate::core::framework::render::RenderStats;

const CAPTURE_ENV: &str = "ZR_F2_BASIC_SCENE_CAPTURE_PNG";
const TARGET_ENV: &str = "CARGO_TARGET_DIR";
const MANAGED_BUILD_POLICY_ENV: &str = "ZIRCON_MANAGED_BUILD_POLICY";

pub(super) struct ProductFrame {
    pub(super) width: u32,
    pub(super) height: u32,
    pub(super) rgba: Vec<u8>,
    pub(super) stats: RenderStats,
}

pub(super) struct FramePixelCounts {
    pub(super) primitive_pixels: usize,
    pub(super) non_transparent_pixels: usize,
}

#[derive(Clone, Copy)]
pub(super) enum FrameSlot {
    FirstLaunch,
    UnchangedSecondFrame,
    SecondLaunchAfterTeardown,
}

impl FrameSlot {
    const fn label(self) -> &'static str {
        match self {
            Self::FirstLaunch => "first launch",
            Self::UnchangedSecondFrame => "unchanged second frame",
            Self::SecondLaunchAfterTeardown => "second launch after teardown",
        }
    }

    const fn file_name(self) -> &'static str {
        match self {
            Self::FirstLaunch => "first-launch.png",
            Self::UnchangedSecondFrame => "unchanged-second-frame.png",
            Self::SecondLaunchAfterTeardown => "second-launch-after-teardown.png",
        }
    }

    const fn suffix(self) -> &'static str {
        match self {
            Self::FirstLaunch => "first-launch",
            Self::UnchangedSecondFrame => "unchanged-second-frame",
            Self::SecondLaunchAfterTeardown => "second-launch-after-teardown",
        }
    }
}

pub(super) struct FrameEvidence {
    explicit_path: Option<PathBuf>,
    managed_directory: Option<PathBuf>,
}

impl FrameEvidence {
    pub(super) fn from_current_environment() -> io::Result<Self> {
        let (explicit_path, invalid_override_suppresses_capture) =
            match std::env::var_os(CAPTURE_ENV) {
                Some(value) => match value.into_string() {
                    Ok(value) => (Some(PathBuf::from(value)), false),
                    Err(_) => (None, true),
                },
                None => (None, false),
            };
        // Validate the managed runner target even when an explicit PNG path is supplied.
        let managed_target = managed_target_directory()?;
        // The old std::env::var path suppressed output for a present non-Unicode override.
        let managed_directory = if explicit_path.is_none() && !invalid_override_suppresses_capture {
            managed_target
                .map(create_unique_managed_evidence_directory)
                .transpose()?
        } else {
            None
        };
        Ok(Self {
            explicit_path,
            managed_directory,
        })
    }

    pub(super) fn record_frame(
        &self,
        frame: &ProductFrame,
        slot: FrameSlot,
        counts: FramePixelCounts,
    ) {
        let path = self.output_path(slot);
        let png_sha256 = if let Some(path) = path.as_deref() {
            write_f2_capture_png(path, frame.width, frame.height, &frame.rgba);
            assert_f2_capture_png(path, frame.width, frame.height, &frame.rgba);
            sha256_file(path)
        } else {
            "not-written".to_owned()
        };

        let device = frame.stats.device_diagnostics.as_ref();
        let backend = nonempty_or_unknown(&frame.stats.capabilities.backend_name);
        let adapter = device
            .map(|diagnostics| diagnostics.adapter_name.as_str())
            .unwrap_or("unknown");
        let adapter_type = device
            .map(|diagnostics| diagnostics.adapter_device_type.as_str())
            .unwrap_or("unknown");
        let limits = device.map(|diagnostics| &diagnostics.limits);
        println!(
            "f2_basic_scene_evidence label={:?} png_path={:?} png_sha256={:?} width={} height={} primitive_pixels={} non_transparent_pixels={} backend={:?} adapter={:?} adapter_type={:?} driver=unknown max_bind_groups={} max_texture_dimension_2d={} max_texture_array_layers={} max_sampled_textures_per_shader_stage={} max_binding_array_elements_per_shader_stage={} max_binding_array_sampler_elements_per_shader_stage={} max_storage_buffers_per_shader_stage={} max_storage_buffer_binding_size={}",
            slot.label(),
            self.reported_path(path.as_deref(), slot),
            png_sha256,
            frame.width,
            frame.height,
            counts.primitive_pixels,
            counts.non_transparent_pixels,
            backend,
            adapter,
            adapter_type,
            diagnostic_value(limits.map(|value| value.max_bind_groups)),
            diagnostic_value(limits.map(|value| value.max_texture_dimension_2d)),
            diagnostic_value(limits.map(|value| value.max_texture_array_layers)),
            diagnostic_value(limits.map(|value| value.max_sampled_textures_per_shader_stage)),
            diagnostic_value(limits.map(|value| value.max_binding_array_elements_per_shader_stage)),
            diagnostic_value(limits.map(|value| value.max_binding_array_sampler_elements_per_shader_stage)),
            diagnostic_value(limits.map(|value| value.max_storage_buffers_per_shader_stage)),
            diagnostic_value(limits.map(|value| value.max_storage_buffer_binding_size)),
        );
    }

    fn output_path(&self, slot: FrameSlot) -> Option<PathBuf> {
        if let Some(explicit_path) = &self.explicit_path {
            return Some(match slot {
                FrameSlot::FirstLaunch => explicit_path.clone(),
                _ => labeled_override_sibling(explicit_path, slot.suffix()),
            });
        }
        self.managed_directory
            .as_ref()
            .map(|directory| directory.join(slot.file_name()))
    }

    fn reported_path(&self, path: Option<&Path>, slot: FrameSlot) -> String {
        let Some(path) = path else {
            return "not-written".to_owned();
        };
        if self.explicit_path.is_some() {
            return format!("explicit-override/{}", slot.file_name());
        }
        path.display().to_string()
    }
}

fn sha256_file(path: &Path) -> String {
    let bytes = fs::read(path).expect("read F2 product capture PNG for SHA-256");
    format!("{:x}", Sha256::digest(bytes))
}

fn nonempty_or_unknown(value: &str) -> &str {
    if value.trim().is_empty() {
        "unknown"
    } else {
        value
    }
}

fn diagnostic_value<T: Display>(value: Option<T>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_owned())
}

fn labeled_override_sibling(path: &Path, suffix: &str) -> PathBuf {
    let stem = path
        .file_stem()
        .or_else(|| path.file_name())
        .unwrap_or_else(|| OsStr::new("f2-basic-scene"));
    let mut file_name = stem.to_os_string();
    file_name.push("-");
    file_name.push(suffix);
    file_name.push(".png");
    path.with_file_name(file_name)
}

fn managed_target_directory() -> io::Result<Option<PathBuf>> {
    if std::env::var_os(MANAGED_BUILD_POLICY_ENV).is_none() {
        return Ok(None);
    }
    let configured = std::env::var_os(TARGET_ENV)
        .map(PathBuf::from)
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "managed F2 capture requires CARGO_TARGET_DIR when ZIRCON_MANAGED_BUILD_POLICY is set",
            )
        })?;
    if !configured.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed F2 CARGO_TARGET_DIR must be an absolute path beneath an approved cargo-targets root",
        ));
    }
    #[cfg(windows)]
    {
        return validate_windows_managed_target(&configured).map(Some);
    }
    #[cfg(target_os = "linux")]
    {
        return validate_linux_managed_target(&configured).map(Some);
    }
    #[cfg(not(any(windows, target_os = "linux")))]
    {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed F2 capture target validation is unsupported on this platform",
        ))
    }
}

#[cfg(windows)]
fn validate_windows_managed_target(configured: &Path) -> io::Result<PathBuf> {
    let approved_roots = [
        r"D:\cargo-targets",
        r"E:\cargo-targets",
        r"F:\cargo-targets",
    ];
    let approved_root = approved_roots
        .iter()
        .find(|root| windows_path_is_within(configured, Path::new(*root)))
        .copied()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "managed F2 CARGO_TARGET_DIR must be lexically beneath D:\\cargo-targets, E:\\cargo-targets, or F:\\cargo-targets",
            )
        })?;
    let physical_root = Path::new(approved_root).canonicalize().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot canonicalize approved managed root {approved_root}: {error}"),
        )
    })?;
    if !windows_path_matches_literal_root(&physical_root, approved_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "approved managed root {approved_root} resolves to {}, outside its literal physical drive root",
                physical_root.display()
            ),
        ));
    }
    let physical_target = configured.canonicalize().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "cannot canonicalize managed CARGO_TARGET_DIR {}: {error}",
                configured.display()
            ),
        )
    })?;
    if !windows_path_is_within(&physical_target, &physical_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed CARGO_TARGET_DIR {} resolves outside approved physical root {}",
                configured.display(),
                physical_root.display()
            ),
        ));
    }
    if normalized_windows_path(configured) != normalized_windows_path(&physical_target) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed CARGO_TARGET_DIR {} resolves through a path alias to {}; aliases are not accepted",
                configured.display(),
                physical_target.display()
            ),
        ));
    }
    Ok(physical_target)
}

#[cfg(windows)]
fn normalized_windows_path(path: &Path) -> String {
    let normalized = path.to_string_lossy().replace('/', "\\");
    normalized
        .strip_prefix(r"\\?\")
        .unwrap_or(&normalized)
        .to_ascii_lowercase()
}

#[cfg(windows)]
fn windows_path_matches_literal_root(path: &Path, root: &str) -> bool {
    normalized_windows_path(path) == normalized_windows_path(Path::new(root))
}

#[cfg(windows)]
fn windows_path_is_within(path: &Path, root: &Path) -> bool {
    let path = normalized_windows_path(path);
    let root = normalized_windows_path(root);
    path == root || path.starts_with(&format!("{}\\", root.trim_end_matches('\\')))
}

#[cfg(target_os = "linux")]
fn validate_linux_managed_target(configured: &Path) -> io::Result<PathBuf> {
    let approved_roots = [
        "/mnt/d/cargo-targets",
        "/mnt/e/cargo-targets",
        "/mnt/f/cargo-targets",
    ];
    let approved_root = approved_roots
        .iter()
        .find(|root| {
            configured == Path::new(*root) || configured.starts_with(Path::new(*root))
        })
        .copied()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "managed F2 CARGO_TARGET_DIR must be lexically beneath /mnt/d/cargo-targets, /mnt/e/cargo-targets, or /mnt/f/cargo-targets",
            )
        })?;
    let physical_root = Path::new(approved_root).canonicalize().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("cannot canonicalize approved managed root {approved_root}: {error}"),
        )
    })?;
    if physical_root != Path::new(approved_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "approved managed root {approved_root} resolves to {}, outside its literal physical mount root",
                physical_root.display()
            ),
        ));
    }
    let physical_target = configured.canonicalize().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "cannot canonicalize managed CARGO_TARGET_DIR {}: {error}",
                configured.display()
            ),
        )
    })?;
    if physical_target != physical_root && !physical_target.starts_with(&physical_root) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed CARGO_TARGET_DIR {} resolves outside approved physical root {}",
                configured.display(),
                physical_root.display()
            ),
        ));
    }
    if physical_target != configured {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "managed CARGO_TARGET_DIR {} resolves through a path alias to {}; aliases are not accepted",
                configured.display(),
                physical_target.display()
            ),
        ));
    }
    Ok(physical_target)
}

fn create_unique_managed_evidence_directory(target: PathBuf) -> io::Result<PathBuf> {
    let evidence_root = target.join("f2-evidence");
    match fs::create_dir(&evidence_root) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error),
    }
    let metadata = fs::symlink_metadata(&evidence_root)?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed F2 evidence root must be a real directory",
        ));
    }
    let evidence_root = evidence_root.canonicalize()?;
    if !evidence_root.starts_with(&target) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed F2 evidence path escaped its target directory",
        ));
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    for attempt in 0..16_u8 {
        let path = evidence_root.join(format!(
            "foundation-render-{}-{timestamp}-{attempt}",
            std::process::id()
        ));
        match fs::create_dir(&path) {
            Ok(()) => {
                let path = path.canonicalize()?;
                if path.starts_with(&evidence_root) {
                    return Ok(path);
                }
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "managed F2 evidence run path escaped its target directory",
                ));
            }
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(error) => return Err(error),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "could not allocate a unique managed F2 evidence directory",
    ))
}

pub(super) fn write_f2_capture_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create F2 capture output directory");
    }
    let image = image::RgbaImage::from_raw(width, height, rgba.to_vec())
        .expect("F2 capture buffer must match dimensions");
    image
        .save_with_format(path, ImageFormat::Png)
        .expect("write F2 product capture PNG");
}

pub(super) fn assert_f2_capture_png(path: &Path, width: u32, height: u32, rgba: &[u8]) {
    let captured = image::open(path)
        .expect("read F2 product capture PNG")
        .to_rgba8();
    assert_eq!(
        captured.dimensions(),
        (width, height),
        "F2 product capture must preserve frame dimensions"
    );
    assert_eq!(
        captured.as_raw(),
        rgba,
        "F2 product capture must preserve RGBA pixels, including alpha and visible primitive output"
    );
}
