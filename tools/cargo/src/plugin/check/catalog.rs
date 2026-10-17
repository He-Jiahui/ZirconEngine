//! Collect real manifest includes from the catalog's linked Rust module tree.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};

use super::PluginCheckError;

pub(super) fn manifest_includes(root: &Path) -> Result<HashSet<PathBuf>, PluginCheckError> {
    let mut pending = vec![root.to_path_buf()];
    let mut visited = HashSet::new();
    let mut manifests = HashSet::new();
    while let Some(source_path) = pending.pop() {
        let source_path = source_path.canonicalize()?;
        if !visited.insert(source_path.clone()) {
            continue;
        }
        let parent = source_path
            .parent()
            .ok_or_else(|| PluginCheckError::new("catalog source has no parent"))?;
        let module_directory = match source_path.file_name().and_then(|name| name.to_str()) {
            Some("lib.rs" | "main.rs" | "mod.rs") => parent.to_path_buf(),
            _ => parent.join(source_path.file_stem().ok_or_else(|| {
                PluginCheckError::new("catalog source has no module name")
            })?),
        };
        let syntax = syn::parse_file(&fs::read_to_string(&source_path)?)?;
        let mut references = SourceReferences {
            module_directory,
            path_directory: parent.to_path_buf(),
            includes: Vec::new(),
            modules: Vec::new(),
        };
        references.visit_file(&syntax);
        for include in references.includes {
            let path = parent.join(include);
            if path.file_name().and_then(|name| name.to_str()) == Some("plugin.toml") {
                manifests.insert(path.canonicalize()?);
            }
        }
        pending.extend(references.modules);
    }
    Ok(manifests)
}

struct SourceReferences {
    module_directory: PathBuf,
    path_directory: PathBuf,
    includes: Vec<String>,
    modules: Vec<PathBuf>,
}

impl<'ast> Visit<'ast> for SourceReferences {
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        if node.path.is_ident("include_str") {
            if let Ok(argument) = syn::parse2::<syn::LitStr>(node.tokens.clone()) {
                self.includes.push(argument.value());
            }
        }
        visit::visit_macro(self, node);
    }

    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if let Some((_, items)) = &node.content {
            let previous_module = self.module_directory.clone();
            let previous_path = self.path_directory.clone();
            self.module_directory.push(node.ident.to_string());
            self.path_directory = self.module_directory.clone();
            for item in items {
                self.visit_item(item);
            }
            self.module_directory = previous_module;
            self.path_directory = previous_path;
            return;
        }
        let explicit_path = node.attrs.iter().find_map(|attribute| {
            if !attribute.path().is_ident("path") {
                return None;
            }
            let syn::Meta::NameValue(value) = &attribute.meta else {
                return None;
            };
            let syn::Expr::Lit(expression) = &value.value else {
                return None;
            };
            let syn::Lit::Str(path) = &expression.lit else {
                return None;
            };
            Some(path.value())
        });
        if let Some(path) = explicit_path {
            self.modules.push(self.path_directory.join(path));
        } else {
            let name = node.ident.to_string();
            let flat = self.module_directory.join(format!("{name}.rs"));
            self.modules.push(if flat.is_file() {
                flat
            } else {
                self.module_directory.join(name).join("mod.rs")
            });
        }
    }
}
