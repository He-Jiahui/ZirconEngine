use std::error::Error;
use std::path::{Path, PathBuf};

pub(crate) const DEFAULT_FACE_SIZE: u32 = 256;
pub(crate) const MIN_FACE_SIZE: u32 = 64;
pub(crate) const MAX_FACE_SIZE: u32 = 512;

#[derive(Clone, Debug)]
pub(crate) struct ViewerConfig {
    pub(crate) hdri_path: PathBuf,
    pub(crate) face_size: u32,
    pub(crate) renderdoc_capture_once: bool,
    pub(crate) exit_after_capture: bool,
    pub(crate) help_requested: bool,
}

impl ViewerConfig {
    pub(crate) fn from_args(
        args: impl IntoIterator<Item = String>,
    ) -> Result<Self, Box<dyn Error>> {
        let mut hdri_path = default_hdri_path();
        let mut face_size = DEFAULT_FACE_SIZE;
        let mut renderdoc_capture_once = false;
        let mut exit_after_capture = false;
        let mut help_requested = false;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" | "--help" => help_requested = true,
                "--renderdoc-capture-once" => renderdoc_capture_once = true,
                "--exit-after-capture" => exit_after_capture = true,
                "--hdri" => {
                    let Some(path) = args.next() else {
                        return Err("--hdri requires a file path".into());
                    };
                    hdri_path = PathBuf::from(path);
                }
                "--face-size" => {
                    let Some(value) = args.next() else {
                        return Err("--face-size requires a pixel value".into());
                    };
                    face_size = parse_face_size(&value)?;
                }
                _ if arg.starts_with('-') => {
                    return Err(format!("unknown argument `{arg}`").into());
                }
                _ => {
                    hdri_path = PathBuf::from(arg);
                }
            }
        }

        Ok(Self {
            hdri_path,
            face_size,
            renderdoc_capture_once,
            exit_after_capture,
            help_requested,
        })
    }
}

fn parse_face_size(value: &str) -> Result<u32, Box<dyn Error>> {
    let parsed = value.parse::<u32>()?;
    if !(MIN_FACE_SIZE..=MAX_FACE_SIZE).contains(&parsed) || !parsed.is_power_of_two() {
        return Err(format!(
            "--face-size must be a power of two between {MIN_FACE_SIZE} and {MAX_FACE_SIZE}, got {parsed}"
        )
        .into());
    }
    Ok(parsed)
}

pub(crate) fn default_hdri_path() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir);
    workspace_root
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("shader")
        .join("assets")
        .join("polyhaven_lakes_2k.hdr")
}

pub(crate) fn print_help() {
    println!(
        "zircon_shader_pbr_viewer [--hdri <path>]\n\
         Optional: --face-size <64|128|256|512>\n\
         Optional: --renderdoc-capture-once [--exit-after-capture]\n\
         Left mouse drag: orbit camera\n\
         Mouse wheel: zoom\n\
         Default HDRI: {}\n\
         Default face size: {}",
        default_hdri_path().display(),
        DEFAULT_FACE_SIZE
    );
}
