use super::*;

#[test]
fn base_project_locator_borrows_the_common_unlabeled_key() {
    let locator = AssetUri::parse("res://textures/albedo.ztexture").unwrap();
    let base = base_project_locator(&locator).unwrap();

    assert!(matches!(base, Cow::Borrowed(value) if std::ptr::eq(value, &locator)));
    assert_eq!(into_base_project_locator(locator.clone()).unwrap(), locator);
}

#[test]
fn base_project_locator_owns_only_the_label_stripped_key() {
    let locator = AssetUri::parse("res://models/hero.glb#Mesh0").unwrap();
    let expected = AssetUri::parse("res://models/hero.glb").unwrap();

    assert_eq!(base_project_locator(&locator).unwrap().as_ref(), &expected);
    assert_eq!(into_base_project_locator(locator).unwrap(), expected);
}
