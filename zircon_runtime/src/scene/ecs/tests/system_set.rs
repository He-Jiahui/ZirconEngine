use super::SystemSetRegistry;

#[test]
fn runtime60_batch_borrowed_system_set_intern_reuses_dense_id() {
    let mut registry = SystemSetRegistry::default();

    let first = registry.intern("physics.main").unwrap();
    let repeated = registry.intern("physics.main").unwrap();

    assert_eq!(repeated, first);
    assert_eq!(registry.names.len(), 1);
    assert_eq!(registry.ids_by_name.len(), 1);
}

#[test]
fn runtime60_batch_owned_system_set_intern_preserves_name() {
    let mut registry = SystemSetRegistry::default();
    let name = String::from("render.main");

    let borrowed_id = registry.intern(&name).unwrap();
    let owned_id = registry.intern(name).unwrap();

    assert_eq!(owned_id, borrowed_id);
    assert_eq!(registry.name(owned_id), Some("render.main"));
}

#[test]
fn runtime60_batch_invalid_borrowed_system_set_does_not_mutate_registry() {
    let mut registry = SystemSetRegistry::default();

    assert!(registry.intern("Physics.main").is_err());

    assert!(registry.names.is_empty());
    assert!(registry.ids_by_name.is_empty());
}
