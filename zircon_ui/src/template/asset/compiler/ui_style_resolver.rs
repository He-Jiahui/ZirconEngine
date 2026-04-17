use crate::{UiAssetDocument, UiAssetError, UiTemplateNode};

use super::style_apply::{apply_styles_to_tree, build_style_plan};
use super::ui_document_compiler::{CompilationArtifacts, ResolvedStyleSheet, UiDocumentCompiler};
use super::value_normalizer::compose_tokens;

#[derive(Default)]
pub struct UiStyleResolver;

impl UiStyleResolver {
    pub(super) fn apply(
        document: &UiAssetDocument,
        compiler: &UiDocumentCompiler,
        root: &mut UiTemplateNode,
        artifacts: &CompilationArtifacts,
    ) -> Result<(), UiAssetError> {
        let mut sheets = artifacts.widget_styles().to_vec();
        for reference in &document.imports.styles {
            let imported = compiler.style_imports.get(reference).ok_or_else(|| {
                UiAssetError::UnknownImport {
                    reference: reference.clone(),
                }
            })?;
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
