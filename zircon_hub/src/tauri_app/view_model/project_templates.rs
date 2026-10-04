//! 把模板目录的稳定身份与可用性投影为本地化选择项和项目详情标签。
//! 模板启用由目录定义，界面显示的标题与原因不能当作模板身份传回。

use serde::Serialize;

use crate::projects::project_template_catalog;
use crate::settings::HubLanguage;

use super::HubTextBundle;

/// 创建表单和详情共同使用的模板显示协议；身份、可用性和文案各有独立字段。
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

/// 从当前模板目录投影选项；页面直接显示完整选择标签，并遵循禁用原因。
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

/// 供历史项目元数据只读显示；没有记录或未知模板不应被解释为可创建选项。
pub(super) fn project_template_label(template_id: Option<&str>, language: HubLanguage) -> String {
    let text = HubTextBundle::new(language);
    let Some(template_id) = template_id.map(str::trim).filter(|id| !id.is_empty()) else {
        return text.pair("No template recorded", "未记录模板").to_string();
    };

    localized_template_title(template_id, language).to_string()
}

// 选择项标签在此完成语言相关标点组合，页面不再拼接标题与状态。
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
#[path = "tests/project_templates.rs"]
mod tests;
