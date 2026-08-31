from pathlib import Path
import unittest


REPO_ROOT = Path(__file__).resolve().parents[2]
SOURCE = REPO_ROOT / "zircon_runtime/src/asset/assets/texture/cube_asset.rs"


def _source() -> str:
    return SOURCE.read_text(encoding="utf-8")


def _function_body(source: str, signature: str) -> str:
    start = source.index(signature)
    opening_brace = source.index("{", start)
    depth = 0
    for index in range(opening_brace, len(source)):
        if source[index] == "{":
            depth += 1
        elif source[index] == "}":
            depth -= 1
            if depth == 0:
                return source[opening_brace + 1 : index]
    raise AssertionError(f"unclosed function body for {signature}")


def test_cubemap_assembly_consumes_owned_face_descriptors() -> None:
    source = _source()
    body = _function_body(source, "pub fn texture_asset_from_cubemap_faces(")

    assert "mut faces: Vec<TextureAsset>" in source
    assert "take_render_image_descriptor(&mut faces[0])" in body
    assert ".iter_mut()" in body
    assert ".skip(1)" in body
    assert "take_render_image_descriptor(texture)" in body


def test_owned_face_descriptor_projection_uses_option_take() -> None:
    body = _function_body(_source(), "fn take_render_image_descriptor(")
    compact_body = "".join(body.split())

    assert "texture.descriptor.take()" in compact_body
    assert "TextureAssetDescriptor::from_payload(&texture.payload)" in compact_body
    assert ".into_render_image_descriptor(texture.width,texture.height)" in compact_body


def test_cubemap_assembly_does_not_clone_render_descriptors() -> None:
    body = _function_body(_source(), "pub fn texture_asset_from_cubemap_faces(")

    assert ".render_image_descriptor()" not in body
    assert body.count("take_render_image_descriptor(") == 2


def test_first_face_is_validated_once_before_remaining_faces() -> None:
    body = _function_body(_source(), "pub fn texture_asset_from_cubemap_faces(")
    remaining_faces_loop = body.index(
        "for (face, texture) in faces.iter_mut().enumerate().skip(1)"
    )

    assert body.count("validate_face_descriptor(") == 2
    assert "validate_face_descriptor(" in body[:remaining_faces_loop]
    assert "&faces[0]" in body[:remaining_faces_loop]


class Runtime92OwnedCubemapFaceDescriptorsPerformanceContractTests(unittest.TestCase):
    def test_cubemap_assembly_consumes_owned_face_descriptors(self) -> None:
        test_cubemap_assembly_consumes_owned_face_descriptors()

    def test_owned_face_descriptor_projection_uses_option_take(self) -> None:
        test_owned_face_descriptor_projection_uses_option_take()

    def test_cubemap_assembly_does_not_clone_render_descriptors(self) -> None:
        test_cubemap_assembly_does_not_clone_render_descriptors()

    def test_first_face_is_validated_once_before_remaining_faces(self) -> None:
        test_first_face_is_validated_once_before_remaining_faces()


if __name__ == "__main__":
    unittest.main()
