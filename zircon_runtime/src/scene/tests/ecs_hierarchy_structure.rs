fn section_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .and_then(|text| text.split(end).next())
        .unwrap_or_else(|| panic!("read section from {start} to {end}"))
}

#[test]
fn remove_entity_uses_direct_entity_and_active_camera_scans() {
    let source = include_str!("../world/hierarchy.rs");
    let remove_entity = section_between(
        source,
        "pub fn remove_entity",
        "pub fn remove_entity_recursive",
    );

    assert!(
        remove_entity.contains("let mut index = 0_usize;")
            && remove_entity.contains("while index < self.entities.len()")
            && remove_entity.contains("if self.entities[index] == entity")
            && remove_entity.contains("if index == self.entities.len()")
            && remove_entity.contains("self.entities.remove(index);")
            && !remove_entity.contains(".position(|current| *current == entity)"),
        "entity removal must locate the stable entity with a direct index scan before removal"
    );
    assert!(
        remove_entity.contains("if self.active_camera == entity")
            && remove_entity.contains("self.active_camera = 0;")
            && remove_entity.contains("for camera in self.cameras.keys().copied()")
            && remove_entity.contains("if camera != entity")
            && remove_entity.contains("self.active_camera = camera;")
            && !remove_entity.contains(".find(|camera| *camera != entity)")
            && !remove_entity.contains(".unwrap_or(0)"),
        "active-camera fallback after entity removal must use a direct camera scan"
    );
}

#[test]
fn mobility_validation_uses_direct_child_and_parent_scans() {
    let source = include_str!("../world/hierarchy.rs");
    let validate_mobility_change = section_between(
        source,
        "pub(super) fn validate_mobility_change",
        "fn ensure_transform_mutable",
    );

    assert!(
        validate_mobility_change.contains("for child in self.entities.iter().copied()")
            && validate_mobility_change.contains("if self.parent_of(child) != Some(entity)")
            && validate_mobility_change.contains("continue;")
            && validate_mobility_change
                .contains("if self.mobility(child) == Some(Mobility::Static)")
            && validate_mobility_change
                .contains("\"cannot make node {entity} Dynamic while it owns Static children\"")
            && !validate_mobility_change
                .contains(".filter(|child| self.parent_of(*child) == Some(entity))")
            && !validate_mobility_change.contains(".any(|child|"),
        "Dynamic mobility validation must scan children directly and stop at the first Static child"
    );
    assert!(
        validate_mobility_change.contains("if let Some(parent) = self.parent_of(entity)")
            && validate_mobility_change
                .contains("if self.mobility(parent) == Some(Mobility::Dynamic)")
            && validate_mobility_change
                .contains("\"cannot make node {entity} Static under Dynamic parent\"")
            && !validate_mobility_change.contains(".is_some_and("),
        "Static mobility validation must use a direct parent branch"
    );
}
