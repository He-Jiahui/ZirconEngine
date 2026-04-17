use super::*;

fn dispatch_with_callback_source(
    weak: &std::rc::Weak<std::cell::RefCell<SlintEditorHost>>,
    source_ui: &WorkbenchShell,
    callback: impl FnOnce(&mut SlintEditorHost),
) {
    if let Some(host) = weak.upgrade() {
        let source_window_id = resolve_callback_source_window_id(&source_ui);
        host.borrow_mut()
            .with_callback_source_window(source_window_id, callback);
    }
}

pub(super) fn wire_callbacks(ui: &WorkbenchShell, host: &Rc<RefCell<SlintEditorHost>>) {
    let weak = Rc::downgrade(host);
    ui.on_menu_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_menu_pointer_moved(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_moved(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_menu_pointer_scrolled(move |x, y, delta| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_scrolled(x, y, delta);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_activity_rail_pointer_clicked(move |side, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .activity_rail_pointer_clicked(side.as_str(), x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_host_page_pointer_clicked(move |tab_index, tab_x, tab_width, point_x, point_y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .host_page_pointer_clicked(tab_index, tab_x, tab_width, point_x, point_y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_document_tab_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().document_tab_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    ui.on_document_tab_close_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().document_tab_close_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    ui.on_floating_window_header_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .floating_window_header_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_recent_pointer_clicked(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_clicked(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_recent_pointer_moved(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_moved(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_recent_pointer_scrolled(move |x, y, delta, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_scrolled(x, y, delta, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_drawer_header_pointer_clicked(
        move |surface_key, tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().drawer_header_pointer_clicked(
                    surface_key.as_str(),
                    tab_index,
                    tab_x,
                    tab_width,
                    point_x,
                    point_y,
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    ui.on_workbench_drag_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().workbench_drag_pointer_event(kind, x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_workbench_resize_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().workbench_resize_pointer_event(kind, x, y);
        }
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_hierarchy_pointer_clicked(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_clicked(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_hierarchy_pointer_moved(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_moved(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_hierarchy_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_console_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.console_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_inspector_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.inspector_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_inspector_control_changed(move |control_id, value| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_inspector_control_changed(control_id.as_str(), value.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_inspector_control_clicked(move |control_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_inspector_control_clicked(control_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_pane_surface_control_clicked(move |control_id, action_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_pane_surface_control_clicked(control_id.as_str(), action_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    ui.on_mesh_import_path_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            let mut host = host.borrow_mut();
            let result =
                callback_dispatch::dispatch_mesh_import_path_edit(&host.runtime, value.to_string());
            host.apply_dispatch_result(result);
        }
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_control_changed(move |source, control_id, value| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_asset_control_changed(
                source.as_str(),
                control_id.as_str(),
                value.as_str(),
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_control_clicked(move |source, control_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_asset_control_clicked(source.as_str(), control_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_tree_pointer_clicked(move |surface_mode, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_tree_pointer_clicked(surface_mode.as_str(), x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_tree_pointer_moved(move |surface_mode, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_tree_pointer_moved(surface_mode.as_str(), x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_tree_pointer_scrolled(move |surface_mode, x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_tree_pointer_scrolled(surface_mode.as_str(), x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_content_pointer_clicked(move |surface_mode, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_content_pointer_clicked(surface_mode.as_str(), x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_content_pointer_moved(move |surface_mode, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_content_pointer_moved(surface_mode.as_str(), x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_content_pointer_scrolled(move |surface_mode, x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_content_pointer_scrolled(surface_mode.as_str(), x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_reference_pointer_clicked(move |surface_mode, list_kind, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_reference_pointer_clicked(
                surface_mode.as_str(),
                list_kind.as_str(),
                x,
                y,
                width,
                height,
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_reference_pointer_moved(move |surface_mode, list_kind, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.asset_reference_pointer_moved(
                surface_mode.as_str(),
                list_kind.as_str(),
                x,
                y,
                width,
                height,
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_asset_reference_pointer_scrolled(
        move |surface_mode, list_kind, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_scrolled(
                    surface_mode.as_str(),
                    list_kind.as_str(),
                    x,
                    y,
                    delta,
                    width,
                    height,
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_browser_asset_details_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.browser_asset_details_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_control_changed(move |control_id, value| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().dispatch_welcome_surface_control(
                control_id.as_str(),
                UiEventKind::Change,
                vec![UiBindingValue::string(value.as_str())],
            );
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_control_clicked(move |control_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().dispatch_welcome_surface_control(
                control_id.as_str(),
                UiEventKind::Click,
                Vec::new(),
            );
        }
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_viewport_pointer_event(move |kind, button, x, y, delta| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.viewport_pointer_event(kind, button, x, y, delta);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_viewport_toolbar_pointer_clicked(
        move |surface_key,
              control_id,
              control_x,
              control_y,
              control_width,
              control_height,
              point_x,
              point_y| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.viewport_toolbar_pointer_clicked(
                    surface_key.as_str(),
                    control_id.as_str(),
                    control_x,
                    control_y,
                    control_width,
                    control_height,
                    point_x,
                    point_y,
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_action(move |instance_id, action_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_action(instance_id.as_str(), action_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_style_class_action(move |instance_id, action_id, class_name| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_style_class_action(
                instance_id.as_str(),
                action_id.as_str(),
                class_name.as_str(),
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_inspector_widget_action(move |instance_id, action_id, value| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_inspector_widget_action(
                instance_id.as_str(),
                action_id.as_str(),
                value.as_str(),
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_style_rule_action(move |instance_id, action_id, item_index, selector| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_style_rule_action(
                instance_id.as_str(),
                action_id.as_str(),
                item_index,
                selector.as_str(),
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_matched_style_rule_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_matched_style_rule_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_style_rule_declaration_action(
        move |instance_id, action_id, item_index, declaration_path, declaration_value| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_style_rule_declaration_action(
                    instance_id.as_str(),
                    action_id.as_str(),
                    item_index,
                    declaration_path.as_str(),
                    declaration_value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_style_token_action(
        move |instance_id, action_id, item_index, token_name, token_value| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_style_token_action(
                    instance_id.as_str(),
                    action_id.as_str(),
                    item_index,
                    token_name.as_str(),
                    token_value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_source_edited(move |instance_id, value| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_source_edited(instance_id.as_str(), value.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_drag_hover(move |instance_id, surface_x, surface_y| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_drag_hover(instance_id.as_str(), surface_x, surface_y);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_drag_drop(move |instance_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_drag_drop(instance_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_drag_cancel(move |instance_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_drag_cancel(instance_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_target_candidate_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_target_candidate_selected(
                instance_id.as_str(),
                item_index,
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_target_confirm(move |instance_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_target_confirm(instance_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_palette_target_cancel(move |instance_id| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_palette_target_cancel(instance_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_hierarchy_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_hierarchy_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_hierarchy_activated(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_hierarchy_activated(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_preview_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_preview_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_source_outline_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_source_outline_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_preview_activated(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_preview_activated(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_preview_mock_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_preview_mock_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_preview_mock_action(move |instance_id, action_id, value| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_preview_mock_action(
                instance_id.as_str(),
                action_id.as_str(),
                value.as_str(),
            );
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_binding_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_binding_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_binding_event_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_binding_event_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_binding_action_kind_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_binding_action_kind_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_binding_payload_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_binding_payload_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_binding_payload_action(
        move |instance_id, action_id, payload_key, payload_value| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_binding_payload_action(
                    instance_id.as_str(),
                    action_id.as_str(),
                    payload_key.as_str(),
                    payload_value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_slot_semantic_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_slot_semantic_selected(instance_id.as_str(), item_index);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    ui.on_ui_asset_layout_semantic_selected(move |instance_id, item_index| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_ui_asset_layout_semantic_selected(instance_id.as_str(), item_index);
        });
    });
}
