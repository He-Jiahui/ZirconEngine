use serde::Serialize;

use crate::projects::project_template_catalog;
use crate::settings::HubLanguage;

use super::HubTextBundle;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct HubProjectTemplate {
    pub id: String,
    pub title: String,
    pub option_label: String,
    pub category: String,
    pub description: String,
    pub enabled: bool,
    pub status: String,
    pub disabled_reason: Option<String>,
}

pub(super) fn project_template_rows(language: HubLanguage) -> Vec<HubProjectTemplate> {
    let text = HubTextBundle::new(language);
    project_template_catalog()
        .iter()
        .map(|template| {
            let title = localized_template_title(template.id, language).to_string();
            let status = if template.enabled {
                text.pair("Available", "可用")
            } else {
                text.pair("Coming Soon", "敬请期待")
            }
            .to_string();
            HubProjectTemplate {
                id: template.id.to_string(),
                option_label: template_option_label(&title, &status, template.enabled, language),
                title,
                category: localized_template_category(template.category, language).to_string(),
                description: localized_template_description(template.id, language).to_string(),
                enabled: template.enabled,
                status,
                disabled_reason: (!template.enabled).then(|| {
                    text.pair(
                        "This template is reserved for a future local workflow.",
                        "该模板为后续本地工作流预留。",
                    )
                    .to_string()
                }),
            }
        })
        .collect()
}

pub(super) fn project_template_label(template_id: Option<&str>, language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    let Some(template_id) = template_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return text.pair("No template recorded", "未记录模板").to_string();
    };

    localized_template_title(template_id, language).to_string()
}

fn template_option_label(
    title: &str,
    status: &str,
    enabled: bool,
    language: HubLanguage,
) -> String {
    if enabled {
        return title.to_string();
    }

    match language {
        HubLanguage::Chinese => format!("{title}（{status}）"),
        HubLanguage::English => format!("{title} ({status})"),
    }
}

fn localized_template_title(id: &str, language: HubLanguage) -> &'static str {
    match (language, id) {
        (HubLanguage::Chinese, "renderable-empty") => "可渲染空项目",
        (HubLanguage::Chinese, "2d-scene") => "2D 场景",
        (HubLanguage::Chinese, "3d-scene") => "3D 场景",
        (HubLanguage::Chinese, "sample-world") => "示例世界",
        (_, "renderable-empty") => "Renderable Empty",
        (_, "2d-scene") => "2D Scene",
        (_, "3d-scene") => "3D Scene",
        (_, "sample-world") => "Sample World",
        _ => "Project Template",
    }
}

fn localized_template_category(category: &str, language: HubLanguage) -> &'static str {
    match (language, category) {
        (HubLanguage::Chinese, "Core") => "核心",
        (HubLanguage::Chinese, "Sample") => "示例",
        (_, "Core") => "Core",
        (_, "Sample") => "Sample",
        _ => "Template",
    }
}

fn localized_template_description(id: &str, language: HubLanguage) -> &'static str {
    match (language, id) {
        (HubLanguage::Chinese, "renderable-empty") => "使用当前引擎运行时创建最小可渲染项目。",
        (HubLanguage::Chinese, "2d-scene") => "为 2D 渲染器工作流预留。",
        (HubLanguage::Chinese, "3d-scene") => "为 3D 场景工作流预留。",
        (HubLanguage::Chinese, "sample-world") => "为示例内容生成预留。",
        (_, "renderable-empty") => "Minimal renderable project with the current engine runtime.",
        (_, "2d-scene") => "Reserved for the 2D renderer workflow.",
        (_, "3d-scene") => "Reserved for the 3D scene workflow.",
        (_, "sample-world") => "Reserved for sample content generation.",
        _ => "Project template.",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_template_option_label_is_localized_before_react_renders_it() {
        let templates = project_template_rows(HubLanguage::Chinese);
        let template = templates
            .iter()
            .find(|template| template.id == "2d-scene")
            .expect("disabled 2D template should be present");

        assert!(!template.enabled);
        assert_eq!(template.status, "敬请期待");
        assert_eq!(template.option_label, "2D 场景（敬请期待）");
    }

    #[test]
    fn selected_project_template_label_localizes_stable_template_ids() {
        assert_eq!(
            project_template_label(Some("renderable-empty"), HubLanguage::Chinese),
            "可渲染空项目"
        );
        assert_eq!(
            project_template_label(None, HubLanguage::Chinese),
            "未记录模板"
        );
    }
}
