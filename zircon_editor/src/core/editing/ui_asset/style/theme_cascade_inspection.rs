use std::collections::BTreeMap;

use zircon_ui::UiAssetDocument;

use super::theme_authoring::can_promote_local_theme_to_external_style_asset;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct UiAssetThemeCascadeInspection {
    pub layer_items: Vec<String>,
    pub token_items: Vec<String>,
    pub rule_items: Vec<String>,
}

#[derive(Clone, Debug)]
struct UiAssetThemeCascadeLayer<'a> {
    kind: UiAssetThemeCascadeLayerKind,
    reference: &'a str,
    document: Option<&'a UiAssetDocument>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiAssetThemeCascadeLayerKind {
    Imported,
    Local,
}

impl UiAssetThemeCascadeLayerKind {
    fn label(self) -> &'static str {
        match self {
            Self::Imported => "Imported",
            Self::Local => "Local",
        }
    }
}

#[derive(Clone, Debug)]
struct UiAssetThemeTokenDefinition {
    source: String,
    value: String,
}

pub(crate) fn build_theme_cascade_inspection(
    document: &UiAssetDocument,
    imported_styles: &BTreeMap<String, UiAssetDocument>,
) -> UiAssetThemeCascadeInspection {
    let layers = cascade_layers(document, imported_styles);
    UiAssetThemeCascadeInspection {
        layer_items: cascade_layer_items(&layers),
        token_items: cascade_token_items(&layers),
        rule_items: cascade_rule_items(&layers),
    }
}

fn cascade_layers<'a>(
    document: &'a UiAssetDocument,
    imported_styles: &'a BTreeMap<String, UiAssetDocument>,
) -> Vec<UiAssetThemeCascadeLayer<'a>> {
    let mut layers = document
        .imports
        .styles
        .iter()
        .map(|reference| UiAssetThemeCascadeLayer {
            kind: UiAssetThemeCascadeLayerKind::Imported,
            reference,
            document: imported_styles.get(reference),
        })
        .collect::<Vec<_>>();
    if can_promote_local_theme_to_external_style_asset(document) {
        layers.push(UiAssetThemeCascadeLayer {
            kind: UiAssetThemeCascadeLayerKind::Local,
            reference: "local",
            document: Some(document),
        });
    }
    layers
}

fn cascade_layer_items(layers: &[UiAssetThemeCascadeLayer<'_>]) -> Vec<String> {
    layers
        .iter()
        .enumerate()
        .map(|(index, layer)| match layer.document {
            Some(document) => format!(
                "{}. {} • {}",
                index + 1,
                layer.kind.label(),
                theme_layer_summary(layer, document),
            ),
            None => format!(
                "{}. {} • {} • missing",
                index + 1,
                layer.kind.label(),
                layer.reference,
            ),
        })
        .collect()
}

fn theme_layer_summary(layer: &UiAssetThemeCascadeLayer<'_>, document: &UiAssetDocument) -> String {
    let token_count = document.tokens.len();
    let rule_count = total_rule_count(document);
    match layer.kind {
        UiAssetThemeCascadeLayerKind::Imported => {
            format!("{reference} • {token_count} tokens • {rule_count} rules", reference = layer.reference)
        }
        UiAssetThemeCascadeLayerKind::Local => format!("{token_count} tokens • {rule_count} rules"),
    }
}

fn cascade_token_items(layers: &[UiAssetThemeCascadeLayer<'_>]) -> Vec<String> {
    let mut tokens_by_name = BTreeMap::<String, Vec<UiAssetThemeTokenDefinition>>::new();
    for layer in layers {
        let Some(document) = layer.document else {
            continue;
        };
        let source = match layer.kind {
            UiAssetThemeCascadeLayerKind::Local => "Local".to_string(),
            UiAssetThemeCascadeLayerKind::Imported => layer.reference.to_string(),
        };
        for (name, value) in &document.tokens {
            tokens_by_name
                .entry(name.clone())
                .or_default()
                .push(UiAssetThemeTokenDefinition {
                    source: source.clone(),
                    value: value.to_string(),
                });
        }
    }

    let mut items = Vec::new();
    for (name, definitions) in tokens_by_name {
        let Some((active, shadowed)) = definitions.split_last() else {
            continue;
        };
        items.push(format!(
            "active • {name} • {} = {}",
            active.source, active.value
        ));
        for definition in shadowed.iter().rev() {
            items.push(format!(
                "shadowed • {name} • {} = {}",
                definition.source, definition.value
            ));
        }
    }
    items
}

fn cascade_rule_items(layers: &[UiAssetThemeCascadeLayer<'_>]) -> Vec<String> {
    let mut items = Vec::new();
    let mut order = 1usize;
    for layer in layers {
        let Some(document) = layer.document else {
            continue;
        };
        for stylesheet in &document.stylesheets {
            let stylesheet_label = if stylesheet.id.is_empty() {
                "<inline>"
            } else {
                stylesheet.id.as_str()
            };
            for rule in &stylesheet.rules {
                let item = match layer.kind {
                    UiAssetThemeCascadeLayerKind::Imported => format!(
                        "{order}. Imported • {} • {stylesheet_label} • {}",
                        layer.reference, rule.selector
                    ),
                    UiAssetThemeCascadeLayerKind::Local => {
                        format!("{order}. Local • {stylesheet_label} • {}", rule.selector)
                    }
                };
                items.push(item);
                order += 1;
            }
        }
    }
    items
}

fn total_rule_count(document: &UiAssetDocument) -> usize {
    document
        .stylesheets
        .iter()
        .map(|stylesheet| stylesheet.rules.len())
        .sum()
}
