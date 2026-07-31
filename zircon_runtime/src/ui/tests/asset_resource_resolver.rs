use crate::core::resource::{
    ResourceId, ResourceKind, ResourceLocator, ResourceManager, ResourceScheme,
};
use crate::ui::template::{
    UiResolvedUiResource, UiResourceResolveDiagnosticCode, UiResourceResolver,
    UiResourceResolverSchemeMap,
};
use zircon_runtime_interface::ui::template::{
    UiResourceDependency, UiResourceDependencySource, UiResourceDiagnosticSeverity,
    UiResourceFallbackMode, UiResourceFallbackPolicy, UiResourceKind, UiResourceRef,
};

#[test]
fn ui_resource_resolver_returns_existing_runtime_resource_handle() {
    let manager = ResourceManager::new();
    let locator = locator("res://ui/icons/scene.icon.toml");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator.clone(),
    ));

    let mut resolver = UiResourceResolver::new(manager);
    let resolved = resolver.resolve(&resource_ref(UiResourceKind::Image, locator.to_string()));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Handle {
            handle: crate::core::resource::UntypedResourceHandle::new(id, ResourceKind::Texture),
            uri: locator.to_string(),
        }
    );
    assert!(resolver.diagnostics().is_empty());
}

#[test]
fn ui_resource_resolver_uses_placeholder_fallback_when_primary_is_missing() {
    let manager = ResourceManager::new();
    let fallback_locator = locator("res://ui/icons/missing-placeholder.icon.toml");
    let fallback_id = ResourceId::from_locator(&fallback_locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        fallback_id,
        ResourceKind::Texture,
        fallback_locator.clone(),
    ));

    let mut resolver = UiResourceResolver::new(manager);
    let resolved = resolver.resolve(&UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "res://ui/icons/missing.icon.toml".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some(fallback_locator.to_string()),
        },
    });

    assert_eq!(
        resolved,
        UiResolvedUiResource::Placeholder {
            handle: Some(crate::core::resource::UntypedResourceHandle::new(
                fallback_id,
                ResourceKind::Texture,
            )),
            diagnostic_index: 0,
        }
    );
    assert_eq!(
        resolver.diagnostics()[0].code,
        UiResourceResolveDiagnosticCode::MissingPrimary
    );
    assert_eq!(
        resolver.diagnostics()[0].severity,
        UiResourceDiagnosticSeverity::Warning
    );
}

#[test]
fn ui_resource_resolver_treats_ui_asset_scheme_as_missing_not_invalid() {
    let manager = ResourceManager::new();
    let mut resolver = UiResourceResolver::new(manager);

    let resolved = resolver.resolve(&resource_ref(
        UiResourceKind::Image,
        "asset://ui/icons/scene.svg".to_string(),
    ));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Placeholder {
            handle: None,
            diagnostic_index: 0,
        }
    );
    assert_eq!(
        resolver.diagnostics()[0].code,
        UiResourceResolveDiagnosticCode::MissingPrimary
    );
    assert_eq!(
        resolver.diagnostics()[0].severity,
        UiResourceDiagnosticSeverity::Warning
    );
}

#[test]
fn ui_resource_resolver_maps_asset_scheme_to_runtime_locator_when_configured() {
    let manager = ResourceManager::new();
    let locator = locator("res://ui/icons/scene.svg");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator.clone(),
    ));
    let mut resolver = UiResourceResolver::new(manager)
        .with_scheme_map(UiResourceResolverSchemeMap::default().asset_to(ResourceScheme::Res));

    let resolved = resolver.resolve(&resource_ref(
        UiResourceKind::Image,
        "asset://ui/icons/scene.svg".to_string(),
    ));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Handle {
            handle: crate::core::resource::UntypedResourceHandle::new(id, ResourceKind::Texture),
            uri: "asset://ui/icons/scene.svg".to_string(),
        }
    );
    assert!(resolver.diagnostics().is_empty());
}

#[test]
fn ui_resource_resolver_maps_project_scheme_to_package_locator_when_configured() {
    let manager = ResourceManager::new();
    let locator = locator("package://demo/ui/icons/scene.svg");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator,
    ));
    let mut resolver = UiResourceResolver::new(manager)
        .with_scheme_map(UiResourceResolverSchemeMap::default().project_to_package("demo"));

    let resolved = resolver.resolve(&resource_ref(
        UiResourceKind::Image,
        "project://ui/icons/scene.svg".to_string(),
    ));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Handle {
            handle: crate::core::resource::UntypedResourceHandle::new(id, ResourceKind::Texture),
            uri: "project://ui/icons/scene.svg".to_string(),
        }
    );
    assert!(resolver.diagnostics().is_empty());
}

#[test]
fn ui_resource_resolver_preserves_ui_scheme_labels_when_mapping_to_runtime_locator() {
    let manager = ResourceManager::new();
    let locator = locator("res://ui/icons/sheet.svg#run");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator,
    ));
    let mut resolver = UiResourceResolver::new(manager)
        .with_scheme_map(UiResourceResolverSchemeMap::default().asset_to(ResourceScheme::Res));

    let resolved = resolver.resolve(&resource_ref(
        UiResourceKind::Image,
        "asset://ui/icons/sheet.svg#run".to_string(),
    ));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Handle {
            handle: crate::core::resource::UntypedResourceHandle::new(id, ResourceKind::Texture),
            uri: "asset://ui/icons/sheet.svg#run".to_string(),
        }
    );
    assert!(resolver.diagnostics().is_empty());
}

#[test]
fn ui_resource_resolver_reports_invalid_mapped_ui_scheme_empty_label() {
    let manager = ResourceManager::new();
    let mut resolver = UiResourceResolver::new(manager)
        .with_scheme_map(UiResourceResolverSchemeMap::default().asset_to(ResourceScheme::Res));

    let resolved = resolver.resolve(&resource_ref(
        UiResourceKind::Image,
        "asset://ui/icons/sheet.svg#".to_string(),
    ));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Placeholder {
            handle: None,
            diagnostic_index: 0,
        }
    );
    assert_eq!(
        resolver.diagnostics()[0].code,
        UiResourceResolveDiagnosticCode::InvalidUri
    );
    assert_eq!(
        resolver.diagnostics()[0].severity,
        UiResourceDiagnosticSeverity::Error
    );
    assert_eq!(
        resolver.diagnostics()[0].message,
        "resource uri is invalid: resource locator label cannot be empty"
    );
}

#[test]
fn ui_resource_resolver_reports_missing_placeholder_fallback() {
    let manager = ResourceManager::new();
    let mut resolver = UiResourceResolver::new(manager);

    let resolved = resolver.resolve(&UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "res://ui/icons/missing.icon.toml".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some("res://ui/icons/missing-placeholder.icon.toml".to_string()),
        },
    });

    assert_eq!(
        resolved,
        UiResolvedUiResource::Placeholder {
            handle: None,
            diagnostic_index: 0,
        }
    );
    assert_eq!(
        resolver.diagnostics()[0].code,
        UiResourceResolveDiagnosticCode::MissingPrimary
    );
    assert_eq!(
        resolver.diagnostics()[1].code,
        UiResourceResolveDiagnosticCode::MissingFallback
    );
    assert_eq!(
        resolver.diagnostics()[1].severity,
        UiResourceDiagnosticSeverity::Error
    );
}

#[test]
fn ui_resource_resolver_reports_kind_mismatch_without_using_wrong_handle() {
    let manager = ResourceManager::new();
    let locator = locator("res://fonts/inter.font.toml");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator.clone(),
    ));

    let mut resolver = UiResourceResolver::new(manager);
    let resolved = resolver.resolve(&resource_ref(UiResourceKind::Font, locator.to_string()));

    assert_eq!(
        resolved,
        UiResolvedUiResource::Placeholder {
            handle: None,
            diagnostic_index: 0,
        }
    );
    assert_eq!(
        resolver.diagnostics()[0].code,
        UiResourceResolveDiagnosticCode::KindMismatch
    );
    assert_eq!(
        resolver.diagnostics()[0].severity,
        UiResourceDiagnosticSeverity::Error
    );
}

#[test]
fn ui_resource_resolver_caches_resolution_by_reference() {
    let manager = ResourceManager::new();
    let locator = locator("res://textures/checker.png");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator.clone(),
    ));

    let mut resolver = UiResourceResolver::new(manager);
    let reference = resource_ref(UiResourceKind::Image, locator.to_string());

    let first = resolver.resolve(&reference);
    let second = resolver.resolve(&reference);

    assert_eq!(first, second);
    assert_eq!(resolver.cache_len(), 1);
    assert!(resolver.diagnostics().is_empty());
}

#[test]
fn ui_resource_resolver_invalidates_cached_primary_and_fallback_uri_references() {
    let manager = ResourceManager::new();
    let primary_locator = locator("res://textures/checker.png");
    let primary_id = ResourceId::from_locator(&primary_locator);
    let fallback_locator = locator("res://textures/fallback.png");
    let fallback_id = ResourceId::from_locator(&fallback_locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        primary_id,
        ResourceKind::Texture,
        primary_locator.clone(),
    ));
    manager.register_record(crate::core::resource::ResourceRecord::new(
        fallback_id,
        ResourceKind::Texture,
        fallback_locator.clone(),
    ));
    let mut resolver = UiResourceResolver::new(manager);
    let primary = resource_ref(UiResourceKind::Image, primary_locator.to_string());
    let fallback = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "res://textures/missing.png".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some(fallback_locator.to_string()),
        },
    };

    resolver.resolve(&primary);
    resolver.resolve(&fallback);
    assert_eq!(resolver.cache_len(), 2);

    let report = resolver.invalidate_uris([
        fallback_locator.to_string(),
        fallback_locator.to_string(),
        "  ".to_string(),
    ]);

    assert_eq!(report.requested_uris, vec![fallback_locator.to_string()]);
    assert_eq!(report.references_removed, 1);
    assert_eq!(report.diagnostics_retained, resolver.diagnostics().len());
    assert_eq!(resolver.cache_len(), 1);

    let report = resolver.invalidate_uris([primary_locator.to_string()]);

    assert_eq!(report.references_removed, 1);
    assert_eq!(resolver.cache_len(), 0);
}

#[test]
fn ui_resource_resolver_invalidates_mapped_ui_scheme_primary_and_fallback_uris() {
    let manager = ResourceManager::new();
    let primary_locator = locator("res://textures/checker.png");
    let primary_id = ResourceId::from_locator(&primary_locator);
    let fallback_locator = locator("res://textures/fallback.png");
    let fallback_id = ResourceId::from_locator(&fallback_locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        primary_id,
        ResourceKind::Texture,
        primary_locator.clone(),
    ));
    manager.register_record(crate::core::resource::ResourceRecord::new(
        fallback_id,
        ResourceKind::Texture,
        fallback_locator.clone(),
    ));
    let mut resolver = UiResourceResolver::new(manager)
        .with_scheme_map(UiResourceResolverSchemeMap::default().asset_to(ResourceScheme::Res));
    let primary = resource_ref(
        UiResourceKind::Image,
        "asset://textures/checker.png".to_string(),
    );
    let fallback = UiResourceRef {
        kind: UiResourceKind::Image,
        uri: "asset://textures/missing.png".to_string(),
        fallback: UiResourceFallbackPolicy {
            mode: UiResourceFallbackMode::Placeholder,
            uri: Some("asset://textures/fallback.png".to_string()),
        },
    };

    resolver.resolve(&primary);
    resolver.resolve(&fallback);
    assert_eq!(resolver.cache_len(), 2);

    let report = resolver.invalidate_uris([fallback_locator.to_string()]);

    assert_eq!(report.requested_uris, vec![fallback_locator.to_string()]);
    assert_eq!(report.references_removed, 1);
    assert_eq!(resolver.cache_len(), 1);

    let report = resolver.invalidate_uris([primary_locator.to_string()]);

    assert_eq!(report.requested_uris, vec![primary_locator.to_string()]);
    assert_eq!(report.references_removed, 1);
    assert_eq!(resolver.cache_len(), 0);
}

#[test]
fn ui_resource_resolver_builds_dependency_resolution_report() {
    let manager = ResourceManager::new();
    let locator = locator("res://ui/textures/checker.png");
    let id = ResourceId::from_locator(&locator);
    manager.register_record(crate::core::resource::ResourceRecord::new(
        id,
        ResourceKind::Texture,
        locator.clone(),
    ));

    let missing = resource_ref(
        UiResourceKind::Image,
        "res://ui/textures/missing.png".to_string(),
    );
    let dependencies = vec![
        dependency(
            resource_ref(UiResourceKind::Image, locator.to_string()),
            "root.props.image",
        ),
        dependency(missing.clone(), "root.props.missing_image"),
        dependency(missing, "root.props.reused_missing_image"),
    ];

    let mut resolver = UiResourceResolver::new(manager);
    let report = resolver.resolve_dependencies(&dependencies);

    assert_eq!(report.resources.len(), 3);
    assert_eq!(report.resolved_count(), 1);
    assert_eq!(report.placeholder_count(), 2);
    assert!(!report.has_errors());
    assert_eq!(report.diagnostics.len(), 1);
    assert_eq!(
        report.resources[0].resolved,
        UiResolvedUiResource::Handle {
            handle: crate::core::resource::UntypedResourceHandle::new(id, ResourceKind::Texture),
            uri: locator.to_string(),
        }
    );
    assert!(report.resources[0].diagnostic_indices.is_empty());
    assert_eq!(report.resources[1].diagnostic_indices, vec![0]);
    assert_eq!(report.resources[2].diagnostic_indices, vec![0]);
    assert_eq!(
        report.resources[1].dependency.path,
        "root.props.missing_image"
    );
}

fn resource_ref(kind: UiResourceKind, uri: String) -> UiResourceRef {
    UiResourceRef {
        kind,
        uri,
        fallback: UiResourceFallbackPolicy::default(),
    }
}

fn locator(value: &str) -> ResourceLocator {
    ResourceLocator::parse(value).unwrap()
}

fn dependency(reference: UiResourceRef, path: &str) -> UiResourceDependency {
    UiResourceDependency {
        reference,
        source: UiResourceDependencySource::NodeProp,
        path: path.to_string(),
    }
}

#[test]
fn ui_resource_resolver_invalidates_uri_batch_with_one_cache_retain() {
    let source = include_str!("../template/asset/resource_ref/resolver.rs");
    let invalidation = source
        .split_once("pub fn invalidate_uris")
        .expect("resource cache invalidation must remain available")
        .1
        .split_once("fn resolve_uncached")
        .expect("invalidation boundary must remain available")
        .0;

    assert!(invalidation.contains("resource_reference_contains_any_uri("));
    assert_eq!(invalidation.matches("self.cache.retain").count(), 1);
    assert!(!invalidation.contains("resource_reference_contains_uri(reference, uri"));
}
