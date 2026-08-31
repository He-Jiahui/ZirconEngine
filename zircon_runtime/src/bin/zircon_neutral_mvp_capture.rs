use std::error::Error;
use std::path::Path;
use std::process::ExitCode;
use std::time::Duration;

use image::{ImageFormat, RgbaImage};
use zircon_runtime::graphics::NeutralMvpRenderer;

const WIDTH: u32 = 320;
const HEIGHT: u32 = 180;
const CAPTURE_TIMEOUT: Duration = Duration::from_secs(5);
const OUTPUT_FILE_NAME: &str = "plan17_wgpu_neutral_mvp_triangle_current.png";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let renderer = NeutralMvpRenderer::new_offscreen(WIDTH, HEIGHT)?;
    let pixels = match renderer.capture_rgba8(1, CAPTURE_TIMEOUT) {
        Ok(pixels) => pixels,
        Err(error) => {
            let _ = renderer.destroy();
            return Err(Box::new(error));
        }
    };
    renderer.destroy()?;

    let image = RgbaImage::from_raw(WIDTH, HEIGHT, pixels)
        .ok_or("neutral MVP capture had an invalid RGBA8 pixel buffer")?;
    let output = workspace_root()
        .join("docs")
        .join("tests")
        .join("runtime")
        .join("render")
        .join(OUTPUT_FILE_NAME);
    if let Some(parent) = output.parent() {
        std::fs::create_dir_all(parent)?;
    }
    image.save_with_format(&output, ImageFormat::Png)?;
    println!("{}", output.display());
    Ok(())
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("zircon_runtime must be directly below the workspace root")
}
