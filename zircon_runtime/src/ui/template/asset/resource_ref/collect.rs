use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

use toml::Value;

use zircon_runtime_interface::ui::template::{
    UiAssetDocument, UiAssetError, UiChildMount, UiNodeDefinition, UiResourceCollectionReport,
    UiResourceDependency, UiResourceDependencySource, UiResourceDiagnostic, UiResourceFallbackMode,
    UiResourceFallbackPolicy, UiResourceKind, UiResourceRef, UiStyleDeclarationBlock,
};

pub fn collect_document_resource_dependencies(
    document: &UiAssetDocument,
    widget_imports: &BTreeMap<String, UiAssetDocument>,
    style_imports: &BTreeMap<String, UiAssetDocument>,
) -> Result<UiResourceCollectionReport, UiAssetError> {
    let mut collector = ResourceDependencyCollector::default();
    let mut path = String::new();
    collector.collect_document(document, None, &mut path)?;
    for (reference, import) in widget_imports {
        let mut path = format!("imported_widget:{reference}");
        collector.collect_document(
            import,
            Some(UiResourceDependencySource::ImportedWidget),
            &mut path,
        )?;
    }
    for (reference, import) in style_imports {
        let mut path = format!("imported_style:{reference}");
        collector.collect_document(
            import,
            Some(UiResourceDependencySource::ImportedStyle),
            &mut path,
        )?;
    }
    Ok(collector.finish())
}

pub fn unique_resource_references(
    dependencies: &[UiResourceDependency],
) -> BTreeSet<UiResourceRef> {
    dependencies
        .iter()
        .map(|dependency| dependency.reference.clone())
        .collect()
}

#[derive(Default)]
struct ResourceDependencyCollector {
    dependencies: BTreeMap<UiResourceRef, UiResourceDependency>,
    diagnostics: Vec<UiResourceDiagnostic>,
}

impl ResourceDependencyCollector {
    fn collect_document(
        &mut self,
        document: &UiAssetDocument,
        import_source: Option<UiResourceDependencySource>,
        path: &mut String,
    ) -> Result<(), UiAssetError> {
        let source_for = |default_source| import_source.unwrap_or(default_source);

        let prefix_len = push_path_segment(path, "imports.resources");
        for (index, reference) in document.imports.resources.iter().enumerate() {
            let index_prefix_len = path.len();
            let _ = write!(path, "[{index}]");
            self.insert_validated(
                reference.clone(),
                source_for(UiResourceDependencySource::DocumentImport),
                path.clone(),
            )?;
            path.truncate(index_prefix_len);
        }
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "tokens");
        for (token, value) in &document.tokens {
            let token_prefix_len = push_path_segment(path, token);
            self.collect_value(
                value,
                source_for(UiResourceDependencySource::TokenValue),
                path,
            )?;
            path.truncate(token_prefix_len);
        }
        path.truncate(prefix_len);

        if let Some(root) = &document.root {
            let prefix_len = push_path_segment(path, "root");
            self.collect_node(root, source_for, path)?;
            path.truncate(prefix_len);
        }

        let prefix_len = push_path_segment(path, "components");
        for (component_name, component) in &document.components {
            let component_prefix_len = push_path_segment(path, component_name);
            let root_prefix_len = push_path_segment(path, "root");
            self.collect_node(&component.root, source_for, path)?;
            path.truncate(root_prefix_len);
            path.truncate(component_prefix_len);
        }
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "stylesheets");
        for stylesheet in &document.stylesheets {
            let stylesheet_prefix_len = push_path_segment(path, &stylesheet.id);
            let rules_prefix_len = push_path_segment(path, "rules");
            for (rule_index, rule) in stylesheet.rules.iter().enumerate() {
                let rule_prefix_len = match rule.id.as_deref() {
                    Some(rule_id) => push_path_segment(path, rule_id),
                    None => {
                        let prefix_len = path.len();
                        let _ = write!(path, "[{rule_index}]");
                        prefix_len
                    }
                };
                let set_prefix_len = push_path_segment(path, "set");
                self.collect_declaration_block(
                    &rule.set,
                    source_for(UiResourceDependencySource::StyleRuleDeclaration),
                    path,
                )?;
                path.truncate(set_prefix_len);
                path.truncate(rule_prefix_len);
            }
            path.truncate(rules_prefix_len);
            path.truncate(stylesheet_prefix_len);
        }
        path.truncate(prefix_len);
        Ok(())
    }

    fn collect_node<F>(
        &mut self,
        node: &UiNodeDefinition,
        source_for: F,
        path: &mut String,
    ) -> Result<(), UiAssetError>
    where
        F: Fn(UiResourceDependencySource) -> UiResourceDependencySource + Copy,
    {
        let prefix_len = push_path_segment(path, "props");
        self.collect_map(
            &node.props,
            source_for(UiResourceDependencySource::NodeProp),
            path,
        )?;
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "params");
        self.collect_map(
            &node.params,
            source_for(UiResourceDependencySource::NodeProp),
            path,
        )?;
        path.truncate(prefix_len);

        if let Some(layout) = &node.layout {
            let prefix_len = push_path_segment(path, "layout");
            self.collect_map(
                layout,
                source_for(UiResourceDependencySource::NodeLayout),
                path,
            )?;
            path.truncate(prefix_len);
        }

        let prefix_len = push_path_segment(path, "style_overrides");
        self.collect_declaration_block(
            &node.style_overrides,
            source_for(UiResourceDependencySource::NodeStyleOverride),
            path,
        )?;
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "children");
        for (index, child) in node.children.iter().enumerate() {
            let index_prefix_len = path.len();
            let _ = write!(path, "[{index}]");
            self.collect_child_mount(child, source_for, path)?;
            path.truncate(index_prefix_len);
        }
        path.truncate(prefix_len);
        Ok(())
    }

    fn collect_child_mount<F>(
        &mut self,
        child: &UiChildMount,
        source_for: F,
        path: &mut String,
    ) -> Result<(), UiAssetError>
    where
        F: Fn(UiResourceDependencySource) -> UiResourceDependencySource + Copy,
    {
        let prefix_len = push_path_segment(path, "slot");
        self.collect_map(
            &child.slot,
            source_for(UiResourceDependencySource::ChildMountSlot),
            path,
        )?;
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "node");
        self.collect_node(&child.node, source_for, path)?;
        path.truncate(prefix_len);
        Ok(())
    }

    fn collect_declaration_block(
        &mut self,
        block: &UiStyleDeclarationBlock,
        source: UiResourceDependencySource,
        path: &mut String,
    ) -> Result<(), UiAssetError> {
        let prefix_len = push_path_segment(path, "self");
        self.collect_map(&block.self_values, source, path)?;
        path.truncate(prefix_len);

        let prefix_len = push_path_segment(path, "slot");
        self.collect_map(&block.slot, source, path)?;
        path.truncate(prefix_len);
        Ok(())
    }

    fn collect_map(
        &mut self,
        values: &BTreeMap<String, Value>,
        source: UiResourceDependencySource,
        path: &mut String,
    ) -> Result<(), UiAssetError> {
        for (key, value) in values {
            let prefix_len = push_path_segment(path, key);
            self.collect_value(value, source, path)?;
            path.truncate(prefix_len);
        }
        Ok(())
    }

    fn collect_value(
        &mut self,
        value: &Value,
        source: UiResourceDependencySource,
        path: &mut String,
    ) -> Result<(), UiAssetError> {
        match value {
            Value::String(uri) if has_supported_scheme(uri) => {
                let reference = UiResourceRef {
                    kind: UiResourceKind::infer_from_path_and_uri(path, uri),
                    uri: uri.clone(),
                    fallback: UiResourceFallbackPolicy::default(),
                };
                self.insert_validated(reference, source, path.clone())?;
            }
            Value::Array(values) => {
                for (index, value) in values.iter().enumerate() {
                    let prefix_len = path.len();
                    let _ = write!(path, "[{index}]");
                    self.collect_value(value, source, path)?;
                    path.truncate(prefix_len);
                }
            }
            Value::Table(table) if is_resource_table(table) => {
                let reference = parse_resource_table(table, path)?;
                self.insert_validated(reference, source, path.clone())?;
            }
            Value::Table(table) => {
                for (key, value) in table {
                    let prefix_len = push_path_segment(path, key);
                    self.collect_value(value, source, path)?;
                    path.truncate(prefix_len);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn insert_validated(
        &mut self,
        reference: UiResourceRef,
        source: UiResourceDependencySource,
        path: String,
    ) -> Result<(), UiAssetError> {
        if let Err(diagnostic) = reference.validate(&path) {
            self.diagnostics.push(diagnostic.clone());
            return Err(UiAssetError::InvalidDocument {
                asset_id: "ui-resource-dependency".to_string(),
                detail: format!("{} at {}", diagnostic.message, diagnostic.path),
            });
        }
        let dependency = UiResourceDependency {
            reference: reference.clone(),
            source,
            path,
        };
        let _ = self.dependencies.entry(reference).or_insert(dependency);
        Ok(())
    }

    fn finish(self) -> UiResourceCollectionReport {
        UiResourceCollectionReport {
            dependencies: self.dependencies.into_values().collect(),
            diagnostics: self.diagnostics,
        }
    }
}

fn push_path_segment(path: &mut String, segment: &str) -> usize {
    let prefix_len = path.len();
    if !path.is_empty() {
        path.push('.');
    }
    path.push_str(segment);
    prefix_len
}

fn is_resource_table(table: &toml::map::Map<String, Value>) -> bool {
    table.get("kind").is_some_and(
        |kind| matches!(kind, Value::String(kind) if resource_kind_from_name(kind).is_some()),
    ) || table.contains_key("uri")
        || matches!(table.get("fallback"), Some(Value::Table(_)))
}

fn parse_resource_table(
    table: &toml::map::Map<String, Value>,
    path: &str,
) -> Result<UiResourceRef, UiAssetError> {
    let kind = required_string(table, "kind", path).and_then(|kind| parse_kind(kind, path))?;
    let uri = required_string(table, "uri", path)?.to_string();
    let fallback = match table.get("fallback") {
        Some(Value::Table(fallback)) => parse_fallback(fallback, path)?,
        Some(_) => return Err(invalid_resource(path, "resource fallback must be a table")),
        None => UiResourceFallbackPolicy::default(),
    };
    Ok(UiResourceRef {
        kind,
        uri,
        fallback,
    })
}

fn parse_fallback(
    table: &toml::map::Map<String, Value>,
    path: &str,
) -> Result<UiResourceFallbackPolicy, UiAssetError> {
    let mode = match table.get("mode") {
        Some(Value::String(mode)) => parse_fallback_mode(mode, path)?,
        Some(_) => {
            return Err(invalid_resource(
                path,
                "resource fallback mode must be a string",
            ))
        }
        None => UiResourceFallbackMode::None,
    };
    let uri = match table.get("uri") {
        Some(Value::String(uri)) => Some(uri.clone()),
        Some(_) => {
            return Err(invalid_resource(
                path,
                "resource fallback uri must be a string",
            ))
        }
        None => None,
    };
    Ok(UiResourceFallbackPolicy { mode, uri })
}

fn required_string<'a>(
    table: &'a toml::map::Map<String, Value>,
    key: &str,
    path: &str,
) -> Result<&'a str, UiAssetError> {
    match table.get(key) {
        Some(Value::String(value)) => Ok(value),
        Some(_) => Err(invalid_resource(
            path,
            &format!("resource {key} must be a string"),
        )),
        None => Err(invalid_resource(
            path,
            &format!("resource table requires {key}"),
        )),
    }
}

fn parse_kind(value: &str, path: &str) -> Result<UiResourceKind, UiAssetError> {
    resource_kind_from_name(value)
        .ok_or_else(|| invalid_resource(path, "resource kind is unsupported"))
}

fn resource_kind_from_name(value: &str) -> Option<UiResourceKind> {
    match value {
        "font" => Some(UiResourceKind::Font),
        "image" => Some(UiResourceKind::Image),
        "media" => Some(UiResourceKind::Media),
        "generic_asset" => Some(UiResourceKind::GenericAsset),
        _ => None,
    }
}

fn parse_fallback_mode(value: &str, path: &str) -> Result<UiResourceFallbackMode, UiAssetError> {
    match value {
        "none" => Ok(UiResourceFallbackMode::None),
        "placeholder" => Ok(UiResourceFallbackMode::Placeholder),
        "optional" => Ok(UiResourceFallbackMode::Optional),
        _ => Err(invalid_resource(
            path,
            "resource fallback mode is unsupported",
        )),
    }
}

fn invalid_resource(path: &str, detail: &str) -> UiAssetError {
    UiAssetError::InvalidDocument {
        asset_id: "ui-resource-dependency".to_string(),
        detail: format!("{detail} at {path}"),
    }
}

fn has_supported_scheme(uri: &str) -> bool {
    uri.starts_with("res://") || uri.starts_with("asset://") || uri.starts_with("project://")
}

#[cfg(test)]
#[path = "collect/path_buffer_tests.rs"]
mod path_buffer_tests;
