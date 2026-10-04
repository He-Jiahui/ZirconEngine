use std::collections::BTreeMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::{
    insert_unique, validate_contribution_id, EditorExtensionRegistration, EditorExtensionRegistry,
    EditorExtensionRegistryError, EditorUiTemplatePaneDataSource,
};

impl EditorExtensionRegistry {
    pub fn register_ui_template(
        &mut self,
        mut descriptor: EditorUiTemplateDescriptor,
    ) -> Result<(), EditorExtensionRegistryError> {
        if descriptor.plugin_root.is_none() {
            descriptor.plugin_root = self.ui_template_root.clone();
        }
        validate_ui_template_document("ui template document", descriptor.ui_document())?;
        insert_unique(
            &mut self.ui_templates,
            descriptor.id.clone(),
            descriptor,
            "ui template",
        )
    }

    pub fn register_ui_template_pane_data_source(
        &mut self,
        template_id: impl Into<String>,
        source: Arc<dyn EditorUiTemplatePaneDataSource>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let template_id = template_id.into();
        validate_contribution_id("ui template pane data source", &template_id)?;
        if !self.ui_templates.contains_key(&template_id) {
            return Err(EditorExtensionRegistryError::MissingUiTemplate { template_id });
        }
        insert_unique(
            &mut self.ui_template_pane_data_sources,
            template_id,
            EditorUiTemplatePaneDataSourceRegistration { source },
            "ui template pane data source",
        )
    }

    /// Replaces the complete template contribution set without exposing a partial reload.
    pub fn replace_ui_template_contributions(
        &mut self,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), EditorExtensionRegistryError> {
        let mut candidate_templates = BTreeMap::new();
        for mut descriptor in templates {
            if descriptor.plugin_root.is_none() {
                descriptor.plugin_root = self
                    .ui_templates
                    .get(&descriptor.id)
                    .and_then(|previous| previous.plugin_root.clone())
                    .or_else(|| self.ui_template_root.clone());
            }
            validate_ui_template_document("ui template document", descriptor.ui_document())?;
            insert_unique(
                &mut candidate_templates,
                descriptor.id.clone(),
                descriptor,
                "ui template",
            )?;
        }
        for view in self.views.values() {
            if let Some(template_id) = view.ui_template_id() {
                if !candidate_templates.contains_key(template_id) {
                    return Err(EditorExtensionRegistryError::MissingUiTemplate {
                        template_id: template_id.to_string(),
                    });
                }
            }
        }

        let mut candidate_pane_data_sources = BTreeMap::new();
        for (template_id, source) in pane_data_sources {
            validate_contribution_id("ui template pane data source", &template_id)?;
            if !candidate_templates.contains_key(&template_id) {
                return Err(EditorExtensionRegistryError::MissingUiTemplate { template_id });
            }
            insert_unique(
                &mut candidate_pane_data_sources,
                template_id,
                EditorUiTemplatePaneDataSourceRegistration { source },
                "ui template pane data source",
            )?;
        }

        self.ui_templates = candidate_templates;
        self.ui_template_pane_data_sources = candidate_pane_data_sources;
        self.bind_matching_ui_templates_to_views();
        Ok(())
    }

    pub fn ui_templates(&self) -> Vec<&EditorUiTemplateDescriptor> {
        self.ui_templates.values().collect()
    }

    pub fn ui_template_pane_data_source(
        &self,
        template_id: &str,
    ) -> Option<Arc<dyn EditorUiTemplatePaneDataSource>> {
        self.ui_template_pane_data_sources
            .get(template_id)
            .map(|registration| registration.source.clone())
    }

    pub fn ui_template_pane_data_sources(
        &self,
    ) -> BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>> {
        self.ui_template_pane_data_sources
            .iter()
            .map(|(template_id, registration)| (template_id.clone(), registration.source.clone()))
            .collect()
    }

    pub(crate) fn bind_ui_template_root(&mut self, root: impl AsRef<Path>) {
        let root = root.as_ref().to_path_buf();
        self.ui_template_root = Some(root.clone());
        for descriptor in self.ui_templates.values_mut() {
            descriptor.plugin_root = Some(root.clone());
        }
    }
}

impl EditorExtensionRegistration {
    pub fn replace_ui_template_contributions(
        &mut self,
        templates: impl IntoIterator<Item = EditorUiTemplateDescriptor>,
        pane_data_sources: BTreeMap<String, Arc<dyn EditorUiTemplatePaneDataSource>>,
    ) -> Result<(), EditorExtensionRegistryError> {
        self.registry
            .replace_ui_template_contributions(templates, pane_data_sources)
    }
}

fn validate_ui_template_document(
    kind: &'static str,
    document: &str,
) -> Result<(), EditorExtensionRegistryError> {
    let trimmed = document.trim();
    if trimmed.is_empty() || trimmed.len() != document.len() || !document.ends_with(".zui") {
        return Err(EditorExtensionRegistryError::InvalidUiDocument {
            kind,
            document: document.to_string(),
        });
    }
    Ok(())
}

#[cfg(test)]
#[path = "template_contributions/tests/single_trim_tests.rs"]
mod single_trim_tests;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EditorUiTemplateDescriptor {
    id: String,
    ui_document: String,
    #[serde(skip)]
    plugin_root: Option<PathBuf>,
}

impl EditorUiTemplateDescriptor {
    pub fn new(id: impl Into<String>, ui_document: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            ui_document: ui_document.into(),
            plugin_root: None,
        }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn ui_document(&self) -> &str {
        &self.ui_document
    }

    pub(crate) fn plugin_root(&self) -> Option<&Path> {
        self.plugin_root.as_deref()
    }

    pub(crate) fn inherit_plugin_root_from(&mut self, previous: Option<&Self>) {
        if self.plugin_root.is_none() {
            self.plugin_root = previous.and_then(|descriptor| descriptor.plugin_root.clone());
        }
    }
}

#[derive(Clone)]
pub(super) struct EditorUiTemplatePaneDataSourceRegistration {
    source: Arc<dyn EditorUiTemplatePaneDataSource>,
}

impl fmt::Debug for EditorUiTemplatePaneDataSourceRegistration {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("EditorUiTemplatePaneDataSourceRegistration")
    }
}

impl PartialEq for EditorUiTemplatePaneDataSourceRegistration {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.source, &other.source)
    }
}

#[cfg(test)]
#[path = "tests/template_contributions.rs"]
mod tests;
