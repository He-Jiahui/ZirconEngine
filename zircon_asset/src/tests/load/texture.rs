use crate::load::texture::generate_checker_texture;

#[test]
fn builtin_checker_texture_has_rgba_payload() {
    let payload = generate_checker_texture();

    assert_eq!(
        payload.rgba.len(),
        payload.width as usize * payload.height as usize * 4
    );
}
