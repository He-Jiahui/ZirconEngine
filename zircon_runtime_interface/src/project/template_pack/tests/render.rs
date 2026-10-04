use super::{
    project_template_descriptor, render_entries, validate_manifest_requirements,
    EmbeddedProjectTemplateEntry, ProjectTemplateId, ProjectTemplatePackError,
    RENDERABLE_EMPTY_ENTRIES,
};

#[test]
fn entry_render_rejects_duplicate_canonical_paths() {
    let source = [
        EmbeddedProjectTemplateEntry {
            path: "assets/data.bin",
            bytes: b"first",
        },
        EmbeddedProjectTemplateEntry {
            path: "assets/data.bin",
            bytes: b"second",
        },
    ];

    assert!(matches!(
        render_entries(&source),
        Err(ProjectTemplatePackError::DuplicateEntryPath { path })
            if path == "assets/data.bin"
    ));
}

#[test]
fn entry_render_rejects_a_file_used_as_a_directory() {
    let source = [
        EmbeddedProjectTemplateEntry {
            path: "assets",
            bytes: b"file",
        },
        EmbeddedProjectTemplateEntry {
            path: "assets/data.bin",
            bytes: b"child",
        },
    ];

    assert!(matches!(
        render_entries(&source),
        Err(ProjectTemplatePackError::EntryPathConflictsWithFile { path, ancestor })
            if path == "assets/data.bin" && ancestor == "assets"
    ));
}

#[test]
fn render_rejects_a_manifest_that_weakens_a_required_provider() {
    let manifest = RENDERABLE_EMPTY_ENTRIES
        .iter()
        .find(|entry| entry.path == "zircon-project.toml")
        .expect("embedded template manifest");
    let source = std::str::from_utf8(manifest.bytes).unwrap();
    let weakened = source.replacen("required = true", "required = false", 1);

    assert!(matches!(
        validate_manifest_requirements(
            weakened.as_bytes(),
            project_template_descriptor(ProjectTemplateId::RenderableEmpty),
        ),
        Err(ProjectTemplatePackError::ManifestRequirements { reason })
            if reason.contains("must be enabled, required")
    ));
}

#[test]
fn render_accepts_provider_selection_without_target_modes() {
    let manifest = RENDERABLE_EMPTY_ENTRIES
        .iter()
        .find(|entry| entry.path == "zircon-project.toml")
        .expect("embedded template manifest");
    let source = std::str::from_utf8(manifest.bytes).unwrap();
    let mut value = toml::from_str::<toml::Value>(source).unwrap();
    let selections = value
        .get_mut("plugins")
        .and_then(toml::Value::as_table_mut)
        .and_then(|plugins| plugins.get_mut("selections"))
        .and_then(toml::Value::as_array_mut)
        .expect("template provider selections");
    for selection in selections {
        selection
            .as_table_mut()
            .expect("provider selection table")
            .remove("target_modes");
    }
    let rewritten = toml::to_string(&value).unwrap();

    validate_manifest_requirements(
        rewritten.as_bytes(),
        project_template_descriptor(ProjectTemplateId::RenderableEmpty),
    )
    .expect("missing target_modes uses the runtime all-target default");
}
