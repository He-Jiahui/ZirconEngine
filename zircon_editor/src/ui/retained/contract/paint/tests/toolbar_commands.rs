use super::super::super::template_nodes::push_template_node_commands;
use super::support::positioned_button_node;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_text::measure_runtime_text_width;

#[test]
fn workbench_toolbar_icon_and_label_keep_starship_gap_without_overlap() {
    let mut node = positioned_button_node(
        "WorkbenchModuleCompile",
        "Compile",
        "filled",
        8.0,
        6.0,
        104.0,
        30.0,
    );
    node.icon_name = "zircon_editor_shell/toolbar/compile.svg".into();
    node.action_id = "workbench.module.compile".into();
    node.layout_content_offset_x = 4.0;
    let origin = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 124.0,
        height: 44.0,
    };
    let mut commands = Vec::new();

    push_template_node_commands(&mut commands, &node, &origin, &origin, None, 0);

    let icon = commands
        .iter()
        .find(|command| command.image_pixels.is_some())
        .expect("toolbar command should paint a leading icon inside the button");
    let text = commands
        .iter()
        .find(|command| command.text.as_deref() == Some("Compile"))
        .expect("toolbar command should paint its full authored label");
    let runtime_width = measure_runtime_text_width("Compile", text.font_size);

    let icon_label_gap = text.frame.x - (icon.frame.x + icon.frame.width);
    assert!(
        (icon_label_gap - node.layout_content_offset_x).abs() <= 0.01,
        "native Button painting must honor the authored Starship icon-label gap: authored={}, actual={icon_label_gap}, icon={:?}, text={:?}",
        node.layout_content_offset_x,
        icon.frame,
        text.frame,
    );
    assert!(
        text.frame.width >= runtime_width,
        "Compile should retain its full Runtime Text width: frame={}, runtime={runtime_width}",
        text.frame.width,
    );
    assert!(
        text.frame.x + runtime_width <= node.frame.x + node.frame.width - 8.0 + 0.5,
        "label ink must remain inside the authored Starship right padding"
    );
}

#[test]
fn authored_toolbar_button_icon_and_single_line_label_share_the_content_center() {
    use crate::ui::retained_host::host_contract::paint_text::HostTextLayoutPolicy;
    use crate::ui::retained_host::host_contract::paint_theme::{
        enter_host_paint_theme_scope, host_paint_theme_snapshot_at_scale_for_test,
    };

    for scale in [1.0, 1.25, 1.5] {
        let _theme =
            enter_host_paint_theme_scope(host_paint_theme_snapshot_at_scale_for_test(scale));
        for (id, label, icon_name, x, width) in [
            (
                "WorkbenchToolSelect",
                "Selection Mode",
                "zircon_editor_shell/toolbar/select.svg",
                72.0,
                152.0,
            ),
            (
                "WorkbenchToolbarAddActor",
                "Add",
                "zircon_editor_shell/controls/add.svg",
                228.0,
                72.0,
            ),
        ] {
            let mut node = positioned_button_node(
                id,
                label,
                "text",
                x * scale,
                61.0 * scale,
                width * scale,
                32.0 * scale,
            );
            node.icon_name = icon_name.into();
            node.font_size = 13.333333;
            node.font_weight = 400;
            node.line_height_ratio = 1.2;
            node.layout_content_offset_x = 4.0 * scale;
            let bounds = FrameRect {
                x: 0.0,
                y: 0.0,
                width: 1280.0 * scale,
                height: 800.0 * scale,
            };
            let mut commands = Vec::new();
            push_template_node_commands(&mut commands, &node, &bounds, &bounds, None, 0);

            let images = commands
                .iter()
                .filter(|command| command.image_pixels.is_some())
                .collect::<Vec<_>>();
            let labels = commands
                .iter()
                .filter(|command| command.text.is_some())
                .collect::<Vec<_>>();
            assert_eq!(images.len(), 1, "{id} at scale {scale}");
            assert_eq!(labels.len(), 1, "{id} must not also paint fallback text");
            let text = labels[0];
            let icon = images[0];
            assert_eq!(text.text.as_deref(), Some(label));
            assert_eq!(
                text.text_layout_policy,
                HostTextLayoutPolicy::SingleLineEllipsis
            );
            assert_eq!(text.frame.height, text.line_height);
            let content_center = node.frame.y + node.frame.height * 0.5;
            assert!((text.frame.y + text.frame.height * 0.5 - content_center).abs() < 0.01);
            assert!((icon.frame.y + icon.frame.height * 0.5 - content_center).abs() < 0.01);
            assert!(text.frame.x >= icon.frame.x + icon.frame.width);
            assert_eq!(text.clip_frame.as_ref(), Some(&bounds));
        }
    }
}

#[test]
fn authored_toolbar_button_ownership_keeps_icon_and_existing_specialized_families() {
    use super::super::{is_workbench_button, push_button_commands};

    let bounds = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 240.0,
        height: 64.0,
    };
    for (id, role, component_role, icon_family) in [
        ("WorkbenchToolbarMenu", "IconButton", "icon-button", true),
        ("WorkbenchToolSelect", "", "", true),
        ("WorkbenchMiniAdd", "IconButton", "icon-button", true),
        ("WorkbenchRailScene", "IconButton", "icon-button", true),
        ("WorkbenchStatusTasks", "IconButton", "icon-button", false),
        ("WorkbenchDrawerTabComponents", "Button", "button", false),
        ("WorkbenchToolDropdown", "Dropdown", "dropdown", false),
        ("WorkbenchToolToggle", "Toggle", "toggle", false),
    ] {
        let mut node = positioned_button_node(id, "Reserved label", "text", 8.0, 6.0, 160.0, 36.0);
        node.role = role.into();
        node.component_role = component_role.into();
        node.icon_name = "zircon_editor_shell/toolbar/select.svg".into();
        let rect = FrameRect {
            x: node.frame.x,
            y: node.frame.y,
            width: node.frame.width,
            height: node.frame.height,
        };
        let mut commands = Vec::new();
        assert!(!is_workbench_button(&node), "{id}");
        assert!(!push_button_commands(
            &mut commands,
            &node,
            &rect,
            &bounds,
            0,
            1.0
        ));
        assert!(commands.is_empty());
        if icon_family {
            push_template_node_commands(&mut commands, &node, &bounds, &bounds, None, 0);
            assert_eq!(
                commands
                    .iter()
                    .filter(|command| command.image_pixels.is_some())
                    .count(),
                1,
                "{id} keeps its original icon painter"
            );
            assert!(
                commands.iter().all(|command| command.text.is_none()),
                "{id}"
            );
        }
    }
}
