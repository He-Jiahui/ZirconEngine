use super::*;

#[test]
fn text_font_variations_hash_stable() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let variations = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
    let same = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 650.0)]);
    let different = VariationCoords(vec![(u32::from_be_bytes(*b"wght"), 700.0)]);

    assert_eq!(
        database.instance(face, &variations).unwrap(),
        database.instance(face, &same).unwrap()
    );
    assert_ne!(
        database.instance(face, &variations).unwrap(),
        database.instance(face, &different).unwrap()
    );
}

#[test]
fn text_font_variations_hash_normalizes_coordinate_order() {
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(
            FontFaceDescriptor::regular("Inter Variable"),
            Arc::from([1_u8].as_slice()),
            None,
        )
        .unwrap();
    let forward = VariationCoords(vec![
        (u32::from_be_bytes(*b"wght"), 650.0),
        (u32::from_be_bytes(*b"wdth"), 90.0),
    ]);
    let reversed = VariationCoords(vec![
        (u32::from_be_bytes(*b"wdth"), 90.0),
        (u32::from_be_bytes(*b"wght"), 650.0),
    ]);

    assert_eq!(
        database.instance(face, &forward).unwrap(),
        database.instance(face, &reversed).unwrap()
    );
}

#[test]
fn text_font_database_registers_descriptor_variations_as_default_instance() {
    let mut database = FontDatabase::default();
    let mut descriptor = FontFaceDescriptor::regular("Inter Variable");
    descriptor.variations = VariationCoords(vec![
        (u32::from_be_bytes(*b"wght"), 650.0),
        (u32::from_be_bytes(*b"wdth"), 90.0),
    ]);
    let face = database
        .register_stored_face(descriptor, Arc::from([1_u8].as_slice()), None)
        .unwrap();

    let instance_id = database.default_instance_id(face).unwrap();
    let instance = database.font_instance(instance_id).unwrap();

    assert_eq!(instance.face, face);
    assert_eq!(
        instance.variations,
        VariationCoords(vec![
            (u32::from_be_bytes(*b"wdth"), 90.0),
            (u32::from_be_bytes(*b"wght"), 650.0),
        ])
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_effective_instance_id_tracks_real_weight_axis() {
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(
            Path::new(r"C:\Windows\Fonts\bahnschrift.ttf"),
            Some("Bahnschrift Variable Test"),
            0,
        )
        .expect("register Windows variable font");

    assert_ne!(
        database.effective_instance_id(face, 300).unwrap(),
        database.effective_instance_id(face, 700).unwrap()
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_effective_variations_merge_descriptor_axes_and_ui_weight() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let weight = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wght"))
        .expect("Bahnschrift weight axis");
    let mut descriptor = FontFaceDescriptor::regular("Bahnschrift Descriptor Variable Test");
    descriptor.variations = VariationCoords(vec![(
        u32::from_be_bytes(width.tag.to_bytes()),
        width.min_value,
    )]);
    let mut database = FontDatabase::default();
    let face = database
        .register_stored_face(descriptor, Arc::from(bytes.into_boxed_slice()), None)
        .expect("register variable descriptor face");

    assert_eq!(
        database.effective_variations(face, 700).unwrap(),
        VariationCoords(vec![
            (u32::from_be_bytes(width.tag.to_bytes()), width.min_value,),
            (
                u32::from_be_bytes(weight.tag.to_bytes()),
                700.0_f32.clamp(weight.min_value, weight.max_value),
            ),
        ])
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_instance_identity_quantizes_real_axis_to_f2dot14_bucket() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let negative_span = width.def_value - width.min_value;
    assert!(negative_span > 0.0);
    let first = width.def_value - negative_span * 0.25;
    let same_bucket = first - negative_span * (0.1 / 16_384.0);
    let tag = u32::from_be_bytes(width.tag.to_bytes());
    let mut database = FontDatabase::default();
    let face = database
        .register_font_file(source, Some("Bahnschrift Quantized Instance Test"), 0)
        .expect("register Windows variable font");

    assert_eq!(
        database
            .instance(face, &VariationCoords(vec![(tag, first)]))
            .unwrap(),
        database
            .instance(face, &VariationCoords(vec![(tag, same_bucket)]))
            .unwrap(),
        "coordinates in one OpenType normalized F2DOT14 bucket must share an instance"
    );
}

#[cfg(target_os = "windows")]
#[test]
fn text_font_database_asset_key_deduplicates_same_f2dot14_instance_bucket() {
    let source = Path::new(r"C:\Windows\Fonts\bahnschrift.ttf");
    let bytes = std::fs::read(source).expect("Windows variable-font fixture");
    let parsed = ttf_parser::Face::parse(&bytes, 0).expect("parse Bahnschrift");
    let width = parsed
        .variation_axes()
        .into_iter()
        .find(|axis| axis.tag == ttf_parser::Tag::from_bytes(b"wdth"))
        .expect("Bahnschrift width axis");
    let span = width.def_value - width.min_value;
    let first = width.def_value - span * 0.25;
    let same_bucket = first - span * (0.1 / 16_384.0);
    let tag = u32::from_be_bytes(width.tag.to_bytes());
    let bytes: Arc<[u8]> = Arc::from(bytes.into_boxed_slice());
    let mut first_descriptor = FontFaceDescriptor::regular("Bahnschrift Asset Bucket Test");
    first_descriptor.variations = VariationCoords(vec![(tag, first)]);
    let mut second_descriptor = first_descriptor.clone();
    second_descriptor.variations = VariationCoords(vec![(tag, same_bucket)]);
    let logical_source = Path::new(r"C:\virtual\bahnschrift-variable.ttf");
    let mut database = FontDatabase::default();

    assert_eq!(
        database
            .register_asset_descriptor(first_descriptor, Arc::clone(&bytes), logical_source)
            .unwrap(),
        database
            .register_asset_descriptor(second_descriptor, bytes, logical_source)
            .unwrap(),
        "asset descriptors in one rendered variation bucket must share a base face"
    );
}
