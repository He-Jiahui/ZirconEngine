use super::*;

#[test]
fn opaque_images_preserve_every_byte_and_the_shared_allocation() {
    let bytes: Arc<[u8]> = (0..=u8::MAX)
        .flat_map(|value| [value, u8::MAX - value, value / 2, u8::MAX])
        .collect::<Vec<_>>()
        .into();
    let encoded = encode_linear_premultiplied_srgba8(bytes.clone());
    assert!(Arc::ptr_eq(&bytes, &encoded));
    assert_eq!(&*encoded, &*bytes);
}

#[test]
fn all_channel_alpha_pairs_match_the_original_linear_premultiplication() {
    let bytes = (0..=u8::MAX)
        .flat_map(|alpha| {
            (0..=u8::MAX).flat_map(move |value| [value, u8::MAX - value, value / 2, alpha])
        })
        .collect::<Vec<_>>();
    let mut expected = bytes.clone();
    for pixel in expected.chunks_exact_mut(4) {
        let alpha = f32::from(pixel[3]) / 255.0;
        for channel in &mut pixel[..3] {
            let linear = srgb_to_linear(f32::from(*channel) / 255.0) * alpha;
            *channel = (linear_to_srgb(linear) * 255.0).round() as u8;
        }
    }
    let original: Arc<[u8]> = bytes.into();
    let encoded = encode_linear_premultiplied_srgba8(original.clone());
    assert_eq!(&*encoded, expected);
    assert_ne!(&*encoded, &*original);
}

#[test]
fn transparent_pixels_are_zero_and_an_incomplete_tail_is_preserved() {
    let encoded = encode_linear_premultiplied_srgba8(vec![17, 99, 255, 0, 3, 5, 7].into());
    assert_eq!(&*encoded, &[0, 0, 0, 0, 3, 5, 7]);
}
