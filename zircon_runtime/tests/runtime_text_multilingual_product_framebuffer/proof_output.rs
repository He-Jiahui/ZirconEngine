use std::path::Path;

use image::{ExtendedColorType, ImageEncoder, codecs::png::PngEncoder};
use zircon_runtime::core::resource::io::atomic_write;

pub(super) fn write_product_framebuffer_png(
    output: &Path,
    rgba: &[u8],
    width: u32,
    height: u32,
) -> Result<(), String> {
    let mut encoded = Vec::new();
    PngEncoder::new(&mut encoded)
        .write_image(rgba, width, height, ExtendedColorType::Rgba8)
        .map_err(|error| {
            format!(
                "encode runtime multilingual text framebuffer {}: {error}",
                output.display()
            )
        })?;
    atomic_write(output, &encoded).map_err(|error| {
        format!(
            "atomically write runtime multilingual text framebuffer {}: {error}",
            output.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::super::proof_path::product_proof_work_path;
    use super::write_product_framebuffer_png;

    fn proof_output_test_root(case_name: &str) -> std::path::PathBuf {
        product_proof_work_path(case_name)
    }

    #[test]
    fn product_proof_writer_preserves_existing_evidence_when_encoding_fails() {
        let root = proof_output_test_root("encode-failure");
        let output = root.join("runtime-text.png");
        std::fs::create_dir_all(&root).expect("create proof output test directory");
        std::fs::write(&output, b"previous accepted evidence").expect("seed previous proof");

        let error = write_product_framebuffer_png(&output, &[], 0, 1)
            .expect_err("an invalid framebuffer must not replace existing evidence");

        assert!(error.contains("encode runtime multilingual text framebuffer"));
        assert_eq!(
            std::fs::read(&output).expect("previous proof remains readable"),
            b"previous accepted evidence"
        );
        std::fs::remove_dir_all(root).expect("remove proof output test directory");
    }

    #[test]
    fn product_proof_writer_replaces_existing_evidence_after_complete_png_encoding() {
        let root = proof_output_test_root("replace-existing");
        let output = root.join("runtime-text.png");
        std::fs::create_dir_all(&root).expect("create proof output test directory");
        std::fs::write(&output, b"stale evidence").expect("seed stale proof");

        write_product_framebuffer_png(&output, &[1, 2, 3, 255, 4, 5, 6, 255], 2, 1)
            .expect("complete framebuffer PNG atomically replaces stale evidence");
        let decoded = image::open(&output)
            .expect("replaced proof must decode")
            .to_rgba8();

        assert_eq!(decoded.dimensions(), (2, 1));
        assert_eq!(decoded.as_raw(), &[1, 2, 3, 255, 4, 5, 6, 255]);
        std::fs::remove_dir_all(root).expect("remove proof output test directory");
    }
}
