use super::*;
use crate::ui::slint_host::{PaneSurfaceHostContext, UiHostContext};
use slint::SharedString;

fn dispatch_with_callback_source(
    weak: &std::rc::Weak<std::cell::RefCell<SlintEditorHost>>,
    source_ui: &UiHostWindow,
    callback: impl FnOnce(&mut SlintEditorHost),
) {
    if let Some(host) = weak.upgrade() {
        let source_window_id = resolve_callback_source_window_id(&source_ui);
        host.borrow_mut()
            .with_callback_source_window(source_window_id, callback);
    }
}

pub(super) fn wire_callbacks(ui: &UiHostWindow, host: &Rc<RefCell<SlintEditorHost>>) {
    let host_shell = ui.global::<UiHostContext>();
    let weak = Rc::downgrade(host);
    host_shell.on_frame_requested(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().tick();
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_moved(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_moved(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_menu_pointer_scrolled(move |x, y, delta| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().menu_pointer_scrolled(x, y, delta);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_activity_rail_pointer_clicked(move |side, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .activity_rail_pointer_clicked(side.as_str(), x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_host_page_pointer_clicked(
        move |tab_index, tab_x, tab_width, point_x, point_y| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut()
                    .host_page_pointer_clicked(tab_index, tab_x, tab_width, point_x, point_y);
            }
        },
    );

    let weak = Rc::downgrade(host);
    host_shell.on_document_tab_pointer_clicked(
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
    host_shell.on_document_tab_close_pointer_clicked(
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
    host_shell.on_floating_window_header_pointer_clicked(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .floating_window_header_pointer_clicked(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_drawer_header_pointer_clicked(
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
    host_shell.on_host_drag_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().host_drag_pointer_event(kind, x, y);
        }
    });

    let weak = Rc::downgrade(host);
    host_shell.on_host_resize_pointer_event(move |kind, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().host_resize_pointer_event(kind, x, y);
        }
    });

    let pane_surface_host = ui.global::<PaneSurfaceHostContext>();

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_clicked(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_clicked(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_moved(move |x, y, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_moved(x, y, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_recent_pointer_scrolled(move |x, y, delta, width, height| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .welcome_recent_pointer_scrolled(x, y, delta, width, height);
        }
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_clicked(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_clicked(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_moved(move |x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_moved(x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_hierarchy_pointer_event(move |kind, button, x, y, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.hierarchy_pointer_event(kind, button, x, y, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_console_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.console_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_pointer_scrolled(move |x, y, delta, width, height| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.inspector_pointer_scrolled(x, y, delta, width, height);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_reference_pointer_event(
        move |kind, button, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.inspector_reference_pointer_event(kind, button, x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_control_changed(
        move |control_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_inspector_control_changed(control_id.as_str(), value.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_inspector_control_clicked(move |control_id: SharedString| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.dispatch_inspector_control_clicked(control_id.as_str());
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_surface_control_clicked(
        move |control_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_pane_surface_control_clicked(control_id.as_str(), action_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_activated(
        move |control_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_activated(
                    control_id.as_str(),
                    action_id.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_drag_delta(
        move |control_id: SharedString, action_id: SharedString, delta: f32| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_drag_delta(
                    control_id.as_str(),
                    action_id.as_str(),
                    f64::from(delta),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_edited(
        move |control_id: SharedString, action_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_edited(
                    control_id.as_str(),
                    action_id.as_str(),
                    value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_control_context_requested(
        move |control_id: SharedString, action_id: SharedString, x: f32, y: f32| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_control_context_requested(
                    control_id.as_str(),
                    action_id.as_str(),
                    f64::from(x),
                    f64::from(y),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_component_showcase_option_selected(
        move |control_id: SharedString, action_id: SharedString, option_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_component_showcase_option_selected(
                    control_id.as_str(),
                    action_id.as_str(),
                    option_id.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    pane_surface_host.on_mesh_import_path_edited(move |value: SharedString| {
        if let Some(host) = weak.upgrade() {
            let mut host = host.borrow_mut();
            let result =
                callback_dispatch::dispatch_mesh_import_path_edit(&host.runtime, value.to_string());
            host.apply_dispatch_result(result);
        }
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_control_changed(
        move |source: SharedString, control_id: SharedString, value: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_asset_control_changed(
                    source.as_str(),
                    control_id.as_str(),
                    value.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_control_clicked(
        move |source: SharedString, control_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_asset_control_clicked(source.as_str(), control_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_clicked(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_clicked(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_moved(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_moved(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_tree_pointer_scrolled(
        move |surface_mode: SharedString, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_tree_pointer_scrolled(surface_mode.as_str(), x, y, delta, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_clicked(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_clicked(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_event(
        move |surface_mode: SharedString, kind, button, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_event(
                    surface_mode.as_str(),
                    kind,
                    button,
                    x,
                    y,
                    width,
                    height,
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_moved(
        move |surface_mode: SharedString, x, y, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_moved(surface_mode.as_str(), x, y, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_content_pointer_scrolled(
        move |surface_mode: SharedString, x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_content_pointer_scrolled(
                    surface_mode.as_str(),
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
    pane_surface_host.on_asset_reference_pointer_clicked(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, width, height| {
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
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_reference_pointer_event(
        move |surface_mode: SharedString,
              list_kind: SharedString,
              kind,
              button,
              x,
              y,
              width,
              height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.asset_reference_pointer_event(
                    surface_mode.as_str(),
                    list_kind.as_str(),
                    kind,
                    button,
                    x,
                    y,
                    width,
                    height,
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_reference_pointer_moved(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, width, height| {
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
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_asset_reference_pointer_scrolled(
        move |surface_mode: SharedString, list_kind: SharedString, x, y, delta, width, height| {
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
    pane_surface_host.on_browser_asset_details_pointer_scrolled(
        move |x, y, delta, width, height| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.browser_asset_details_pointer_scrolled(x, y, delta, width, height);
            });
        },
    );

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_control_changed(
        move |control_id: SharedString, value: SharedString| {
            if let Some(host) = weak.upgrade() {
                host.borrow_mut().dispatch_welcome_surface_control(
                    control_id.as_str(),
                    UiEventKind::Change,
                    vec![UiBindingValue::string(value.as_str())],
                );
            }
        },
    );

    let weak = Rc::downgrade(host);
    pane_surface_host.on_welcome_control_clicked(move |control_id: SharedString| {
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
    pane_surface_host.on_viewport_pointer_event(move |kind, button, x, y, delta| {
        dispatch_with_callback_source(&weak, &source_ui, |host| {
            host.viewport_pointer_event(kind, button, x, y, delta);
        });
    });

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_viewport_toolbar_pointer_clicked(
        move |surface_key: SharedString,
              control_id: SharedString,
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
    pane_surface_host.on_ui_asset_action(
        move |instance_id: SharedString, action_id: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_action(instance_id.as_str(), action_id.as_str());
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_ui_asset_detail_event(
        move |instance_id: SharedString,
              detail_id: SharedString,
              action_id: SharedString,
              item_index,
              primary: SharedString,
              secondary: SharedString| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_detail_event(
                    instance_id.as_str(),
                    detail_id.as_str(),
                    action_id.as_str(),
                    item_index,
                    primary.as_str(),
                    secondary.as_str(),
                );
            });
        },
    );

    let weak = Rc::downgrade(host);
    let source_ui = ui.clone_strong();
    pane_surface_host.on_ui_asset_collection_event(
        move |instance_id: SharedString,
              collection_id: SharedString,
              event_kind: SharedString,
              item_index| {
            dispatch_with_callback_source(&weak, &source_ui, |host| {
                host.dispatch_ui_asset_collection_event(
                    instance_id.as_str(),
                    collection_id.as_str(),
                    event_kind.as_str(),
                    item_index,
                );
            });
        },
    );
}
