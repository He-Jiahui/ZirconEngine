use crate::settings::HubLanguage;

#[test]
fn ui_text_defaults_to_chinese_shell_and_project_copy() {
    let text = super::ui_text(HubLanguage::Chinese);

    assert_eq!(text.shell.action_failed, "操作失败");
    assert_eq!(text.shell.nav_items[0].label, "项目");
    assert_eq!(text.actions.new_project, "新建项目");
    assert_eq!(text.actions.open_resource, "打开资源");
    assert_eq!(text.editor.plugin_coming_soon_panel, "预留插件操作");
    assert_eq!(text.shell.active_engine, "当前引擎");
    assert_eq!(text.shell.no_source_engine_registered, "未注册源码引擎");
    assert_eq!(text.shell.user_account, "我的账户");
    assert_eq!(text.shell.workspace_profile, "Zircon Hub 工作区");
    assert_eq!(text.shell.up_to_date, "本地版本");
    assert_eq!(text.shell.check_for_updates, "更新检查预留");
    assert_eq!(
        text.shell.check_for_updates_detail,
        "本地 v1 不启用远程更新服务。"
    );
    assert_eq!(text.shell.expand, "展开");
    assert_eq!(text.shell.demo_mode_badge, "演示数据");
    assert_eq!(text.projects.search_placeholder, "搜索项目...");
    assert_eq!(text.projects.filter_label, "筛选项目");
    assert_eq!(text.projects.sort_label, "排序项目");
    assert_eq!(text.catalog.search_placeholder_prefix, "搜索");
    assert_eq!(text.catalog.search_placeholder_separator, "");
    assert_eq!(text.catalog.search_placeholder_suffix, "...");
}

#[test]
fn ui_text_strings_are_non_empty_except_explicit_separator() {
    for language in [HubLanguage::English, HubLanguage::Chinese] {
        let value =
            serde_json::to_value(super::ui_text(language)).expect("ui text should serialize");

        assert_non_empty_strings(&value, "");
    }
}

fn assert_non_empty_strings(value: &serde_json::Value, path: &str) {
    match value {
        serde_json::Value::String(text) => {
            if path == "catalog.searchPlaceholderSeparator" {
                return;
            }
            assert!(!text.trim().is_empty(), "empty UI text at {path}");
        }
        serde_json::Value::Array(values) => {
            for (index, child) in values.iter().enumerate() {
                assert_non_empty_strings(child, &format!("{path}[{index}]"));
            }
        }
        serde_json::Value::Object(fields) => {
            for (key, child) in fields {
                let next_path = if path.is_empty() {
                    key.to_string()
                } else {
                    format!("{path}.{key}")
                };
                assert_non_empty_strings(child, &next_path);
            }
        }
        _ => {}
    }
}
