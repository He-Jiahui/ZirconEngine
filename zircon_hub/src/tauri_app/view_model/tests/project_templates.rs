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
