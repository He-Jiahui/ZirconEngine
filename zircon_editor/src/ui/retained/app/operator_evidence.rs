//! Opt-in read-only observations for native operator smoke tests. No polling,
//! mutation endpoint or steady-state timer is installed by this observer.
use super::RetainedEditorHost;
use serde_json::json;

impl RetainedEditorHost {
    pub(super) fn publish_operator_evidence(&self) {
        let Some(path) = std::env::var_os("ZIRCON_EDITOR_OPERATOR_EVIDENCE") else {
            return;
        };
        // Observe the already committed UI model. Producing evidence must not
        // rebuild chrome or enter backend/shell locks on a preview-only tick.
        let Some(committed) = self.committed_shell_state.as_ref() else {
            return;
        };
        let chrome = &committed.chrome;
        let generation = self.ui.get_host_presentation_generation();
        let viewport = &generation.structure().host_layout.viewport_content_frame;
        let images = generation.viewport_images();
        let splitters = &generation.structure().host_scene_data.resize_layer;
        let rect = |frame: &crate::ui::retained_host::host_contract::data::FrameRect| {
            [frame.x, frame.y, frame.width, frame.height]
        };
        let close = &generation.structure().close_prompt;
        let discard = &close.discard_button_frame;
        let image_observation = |image: Option<
            &crate::ui::retained_host::host_contract::data::HostViewportImageData,
        >| {
            image.map(|image| json!({"width": image.width, "height": image.height,
                "valid": image.is_valid(), "resource": image.resource_key,
                "generation": image.play_frame_identity.as_ref().map(|identity| identity.generation()),
                "cpu_bytes": image.rgba.as_ref().map_or(0, |rgba| rgba.len()),
                "cpu_center_rgba": image.rgba.as_ref().and_then(|rgba| {
                    let offset = ((image.height / 2 * image.width + image.width / 2) * 4) as usize;
                    rgba.get(offset..offset + 4)
                }),
                "cpu_samples": image.rgba.as_ref().map(|rgba| {
                    [(0.25_f32, 0.25_f32), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)]
                        .into_iter().filter_map(|(x, y)| {
                            let px = (x * image.width as f32) as u32;
                            let py = (y * image.height as f32) as u32;
                            let offset = ((py * image.width + px) * 4) as usize;
                            rgba.get(offset..offset + 4).map(|pixel| json!({"x": x, "y": y, "rgba": pixel}))
                        }).collect::<Vec<_>>()
                }),
                "play_identity": image.play_frame_identity.as_ref().map(|identity| format!("{identity:?}"))}))
        };
        let controls = generation
            .structure()
            .workbench_window_nodes
            .iter()
            .map(|node| {
                json!({"id": node.control_id.as_str(), "action": node.action_id.as_str(),
                "text": node.text.as_str(), "value": node.value_text.as_str(),
                "disabled": node.disabled, "popup_open": node.popup_open,
                "menu_items": node.menu_items.iter().map(|item| item.to_string()).collect::<Vec<_>>(),
                "frame": [node.frame.x, node.frame.y, node.frame.width, node.frame.height]})
            })
            .collect::<Vec<_>>();
        let scene = chrome
            .scene_entries
            .iter()
            .map(|row| {
                json!({"id": row.entity.to_string(), "name": row.display_name,
                "selected": chrome.scene_entries.is_selected(row.entity)})
            })
            .collect::<Vec<_>>();
        let inspector = chrome.inspector.as_ref().map(|value| {
            json!({"id": value.id.to_string(), "name": value.name,
                "translation": value.translation, "rotation_degrees": value.rotation_degrees,
                "scale": value.scale})
        });
        let context = self.runtime.context().command_eval().snapshot();
        let commands = {
            let registry = self.runtime.commands().lock();
            [
                "runtime.play_mode.enter",
                "runtime.play_mode.exit",
                "scene.node.create_cube",
            ]
            .into_iter()
            .map(|id| {
                json!({"id": id,
                    "enabled": registry.command(id).map(|d| d.is_enabled(&context)),
                    "descriptor": registry.command(id).map(|d| format!("{d:?}"))})
            })
            .collect::<Vec<_>>()
        };
        let observation = json!({"version": 1,
            "unix_ms": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default().as_millis(),
            "project_open": chrome.project_open, "status": chrome.status_line,
            "document_pane": generation.structure().host_scene_data.document_dock.pane.kind.as_str(),
            "viewport_frame": [viewport.x, viewport.y, viewport.width, viewport.height],
            "splitters": {"bottom": rect(&splitters.bottom_splitter_frame),
                "left": rect(&splitters.left_splitter_frame), "right": rect(&splitters.right_splitter_frame)},
            "document_surfaces": generation.structure().host_scene_data.document_surfaces().iter()
                .map(|surface| json!({"key": surface.surface_key.as_str(), "pane": surface.pane.kind.as_str()})).collect::<Vec<_>>(),
            "game_image": image_observation(images.game()),
            "play_transition": generation.structure().play_transition,
            "close_prompt": {"visible": close.visible, "can_save": close.can_save,
                "discard_frame": [discard.x, discard.y, discard.width, discard.height]},
            "play_mode": format!("{:?}", self.runtime.play_sessions().mode()),
            "command_context": context, "commands": commands,
            "scene": scene, "inspector": inspector, "controls": controls,
            "gizmo_lines": self.operator_gizmo_lines()});
        // Atomic replacement keeps readers from observing half a JSON document.
        let path = std::path::PathBuf::from(path);
        let temporary = path.with_extension("pending.json");
        let result = serde_json::to_vec(&observation)
            .map_err(std::io::Error::other)
            .and_then(|bytes| std::fs::write(&temporary, bytes))
            .and_then(|()| std::fs::rename(&temporary, &path));
        if let Err(error) = result {
            eprintln!("operator evidence could not be written: {error}");
        }
    }
    fn operator_gizmo_lines(&self) -> Vec<serde_json::Value> {
        if std::env::var_os("ZIRCON_EDITOR_OPERATOR_GIZMO").is_none()
            || self.runtime.play_sessions().mode() != crate::core::play::PlayModeKind::Edit
        {
            return Vec::new();
        }
        let shell = self.runtime.shell().lock();
        let state = &shell.state;
        let Ok(Some(camera)) = state.viewport_camera_snapshot() else {
            return Vec::new();
        };
        state
            .world
            .with_world(|scene| {
                let selected = state
                    .viewport_controller
                    .selection()
                    .active_primary()
                    .and_then(|id| scene.local_transform(id).map(|transform| (id, transform)));
                state
                    .viewport_controller
                    .handle_screen_lines_for_transform(selected, &camera, self.viewport_size)
                    .into_iter()
                    .map(|line| {
                        json!({"start": line.start().to_array(),
                    "end": line.end().to_array(), "axis": format!("{:?}", line.axis())})
                    })
                    .collect()
            })
            .ok()
            .flatten()
            .unwrap_or_default()
    }
}
