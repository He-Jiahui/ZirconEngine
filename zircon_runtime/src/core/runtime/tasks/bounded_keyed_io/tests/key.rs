use std::sync::Arc;

use super::BoundedKeyedIoKey;

#[derive(Clone, PartialEq, Eq)]
struct PhysicalPathIdentity(u64);

#[test]
fn typed_keys_preserve_domain_equality_across_clones() {
    let first = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
    let same = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
    let different = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(8));

    assert_eq!(first, first.clone());
    assert_eq!(first, same);
    assert_ne!(first, different);
}

#[test]
fn different_key_domains_never_collide() {
    let typed = BoundedKeyedIoKey::from_value(PhysicalPathIdentity(7));
    let numeric = BoundedKeyedIoKey::from_value(7_u64);

    assert_ne!(typed, numeric);
}

#[test]
fn existing_string_conversions_share_one_key_domain() {
    let borrowed = BoundedKeyedIoKey::from("archive");
    let owned = BoundedKeyedIoKey::from(String::from("archive"));
    let shared = BoundedKeyedIoKey::from(Arc::<str>::from("archive"));

    assert_eq!(borrowed, owned);
    assert_eq!(owned, shared);
}
