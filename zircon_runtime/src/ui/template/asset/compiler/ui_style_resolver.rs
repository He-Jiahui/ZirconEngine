use zircon_runtime_interface::ui::template::{UiAssetDocument, UiAssetError, UiTemplateNode};

use super::style_apply::{apply_styles_to_tree, build_style_plan};
use super::ui_document_compiler::{CompilationArtifacts, ResolvedStyleSheet, UiDocumentCompiler};
use super::value_normalizer::compose_tokens;

#[cfg(test)]
#[path = "ui_style_resolver/capacity_tests.rs"]
mod capacity_tests;

#[derive(Default)]
pub struct UiStyleResolver;

impl UiStyleResolver {
    pub(super) fn apply(
        document: &UiAssetDocument,
        compiler: &UiDocumentCompiler,
        root: &mut UiTemplateNode,
        artifacts: &CompilationArtifacts,
    ) -> Result<(), UiAssetError> {
        let imported_styles = document
            .imports
            .styles
            .iter()
            .map(|reference| {
                compiler
                    .style_imports
                    .get(reference)
                    .ok_or_else(|| UiAssetError::UnknownImport {
                        reference: reference.clone(),
                    })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let imported_stylesheet_count = imported_styles
            .iter()
            .map(|imported| imported.stylesheets.len())
            .sum::<usize>();
        let mut sheets = Vec::with_capacity(
            artifacts.widget_styles().len()
                + imported_stylesheet_count
                + document.stylesheets.len(),
        );
        sheets.extend_from_slice(artifacts.widget_styles());
        for imported in imported_styles {
            let tokens = compose_tokens(&document.tokens, &imported.tokens);
            for stylesheet in &imported.stylesheets {
                sheets.push(ResolvedStyleSheet {
                    stylesheet: stylesheet.clone(),
                    tokens: tokens.clone(),
                });
            }
        }
        for stylesheet in &document.stylesheets {
            sheets.push(ResolvedStyleSheet {
                stylesheet: stylesheet.clone(),
                tokens: document.tokens.clone(),
            });
        }

        let parsed = build_style_plan(&sheets)?;
        let mut path = Vec::new();
        apply_styles_to_tree(root, &parsed, &mut path);
        Ok(())
    }
}
