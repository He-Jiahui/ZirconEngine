use std::collections::BTreeSet;

use super::*;

#[test]
fn gltf_lexical_source_identity_orders_missing_paths_without_io() {
    let base = std::env::temp_dir().join("zircon-lexical-uncreated-source");
    let lower = LexicalProjectPathIdentity::new(&base.join("mesh.bin")).unwrap();
    let upper = LexicalProjectPathIdentity::new(&base.join("MESH.BIN")).unwrap();
    let identities = BTreeSet::from([lower, upper]);
    assert_eq!(identities.len(), if cfg!(windows) { 1 } else { 2 });
}
