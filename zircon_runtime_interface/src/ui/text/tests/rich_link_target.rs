use super::UiRichLinkTarget;

#[test]
fn accepts_only_canonical_engine_local_resource_schemes() {
    let target = UiRichLinkTarget::parse(" docs/./guide.zui#intro ").unwrap();
    assert!(target.matches_display("res://docs/guide.zui#intro"));

    for value in [
        "res://docs/guide.zui",
        "lib://docs/guide.zui",
        "package://com.zircon.docs/guide.zui",
        "builtin://docs/guide.zui",
    ] {
        assert!(UiRichLinkTarget::parse(value).is_ok(), "{value}");
    }
    for value in [
        "",
        "mem://transient/guide.zui",
        "https://example.com/guide",
        "res://../guide.zui",
        "res://docs/guide.zui#",
    ] {
        assert!(UiRichLinkTarget::parse(value).is_err(), "{value}");
    }
}

#[test]
fn serde_revalidates_and_keeps_the_canonical_string_wire_shape() {
    let target = UiRichLinkTarget::parse("res://docs/./guide.zui").unwrap();
    assert_eq!(
        serde_json::to_string(&target).unwrap(),
        r#""res://docs/guide.zui""#
    );
    assert!(serde_json::from_str::<UiRichLinkTarget>(r#""mem://guide.zui""#).is_err());
}
