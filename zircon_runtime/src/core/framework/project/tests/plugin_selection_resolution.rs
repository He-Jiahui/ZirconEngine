use super::*;

fn selection(id: &str, required: bool) -> ProjectPluginSelection {
    ProjectPluginSelection {
        id: id.to_owned(),
        enabled: true,
        required,
        target_modes: vec![RuntimeTargetMode::ClientRuntime],
        packaging: super::super::ExportPackagingStrategy::LibraryEmbed,
        runtime_crate: None,
        editor_crate: None,
        features: Vec::new(),
    }
}

#[test]
fn resolver_preserves_one_typed_outcome_per_enabled_selection() {
    let manifest = ProjectPluginManifest {
        selections: vec![
            selection("sound", true),
            selection("sound", true),
            selection("unknown-provider", false),
            selection("", true),
        ],
    };

    let report = resolve_plugin_selections(RuntimeTargetMode::ClientRuntime, &manifest, |id| {
        (id == &RuntimePluginId::Sound).then(|| id.key().to_owned())
    });

    assert_eq!(report.len(), 1);
    assert_eq!(report.outcomes().len(), 4);
    assert_eq!(
        report
            .outcomes()
            .iter()
            .map(|outcome| outcome.status)
            .collect::<Vec<_>>(),
        vec![
            PluginSelectionResolutionStatus::Resolved,
            PluginSelectionResolutionStatus::Duplicate,
            PluginSelectionResolutionStatus::Unsupported,
            PluginSelectionResolutionStatus::InvalidId,
        ]
    );
}

#[test]
fn optional_unsupported_selection_does_not_block_registrations() {
    let manifest = ProjectPluginManifest {
        selections: vec![selection("unknown-provider", false)],
    };
    let report =
        resolve_plugin_selections::<()>(RuntimeTargetMode::ClientRuntime, &manifest, |_| None);

    let checked = report.into_registrations_if_required_resolved().unwrap();
    assert_eq!(
        checked.outcomes()[0].status,
        PluginSelectionResolutionStatus::Unsupported
    );
}

#[test]
fn required_inapplicable_selections_remain_visible_without_blocking_ready() {
    let mut disabled = selection("disabled", true);
    disabled.enabled = false;
    let mut other_target = selection("server-only", true);
    other_target.target_modes = vec![RuntimeTargetMode::ServerRuntime];
    let manifest = ProjectPluginManifest {
        selections: vec![disabled, other_target],
    };
    let checked =
        resolve_plugin_selections::<()>(RuntimeTargetMode::ClientRuntime, &manifest, |_| {
            panic!("inapplicable provider must not run")
        })
        .into_registrations_if_required_resolved()
        .unwrap();
    assert_eq!(checked.outcomes().len(), manifest.selections.len());
    assert_eq!(
        checked.outcomes()[0].status,
        PluginSelectionResolutionStatus::SkippedDisabled
    );
    assert_eq!(
        checked.outcomes()[1].status,
        PluginSelectionResolutionStatus::SkippedTargetMismatch
    );
}

#[test]
fn required_duplicate_invalid_and_unsupported_selections_fail_closed() {
    let manifest = ProjectPluginManifest {
        selections: vec![
            selection("sound", false),
            selection("sound", true),
            selection("unknown-provider", true),
            selection("", true),
        ],
    };
    let error = resolve_plugin_selections(RuntimeTargetMode::ClientRuntime, &manifest, |id| {
        (id == &RuntimePluginId::Sound).then_some(())
    })
    .into_registrations_if_required_resolved()
    .expect_err("unresolved required selections must fail closed");

    assert_eq!(error.failures().len(), 3);
}

#[test]
fn scoped_required_check_ignores_failures_outside_the_consumer_domain() {
    let manifest = ProjectPluginManifest {
        selections: vec![selection("unknown-provider", true)],
    };
    let report =
        resolve_plugin_selections::<()>(RuntimeTargetMode::ClientRuntime, &manifest, |_| None);

    assert!(report
        .into_registrations_if_required_resolved_where(|selection| {
            selection.editor_crate.is_some()
        })
        .is_ok());
}
