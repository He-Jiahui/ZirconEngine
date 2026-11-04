//! Opt-in observations of committed native UI state for the Linux acceptance driver.
//! This adds no mutation endpoint, timer, or backend/shell lock acquisition.
use super::RetainedEditorHost;
use serde_json::json;

impl RetainedEditorHost {
    pub(super) fn publish_interaction_evidence(&self) {
        let Some(path) = std::env::var_os("ZIRCON_EDITOR_INTERACTION_EVIDENCE") else {
            return;
        };
        let Some(committed) = self.committed_shell_state.as_ref() else {
            return;
        };
        let chrome = &committed.chrome;
        let generation = self.ui.get_host_presentation_generation();
        let presentation = generation.structure();
        let viewport = &presentation.host_layout.viewport_content_frame;
        let close = &presentation.close_prompt;
        let discard = &close.discard_button_frame;
        let scene = chrome
            .scene_entries
            .iter()
            .map(|row| {
                json!({
                    "id": row.entity.to_string(), "name": row.display_name,
                    "selected": chrome.scene_entries.is_selected(row.entity)
                })
            })
            .collect::<Vec<_>>();
        let inspector = chrome.inspector.as_ref().map(|value| {
            json!({
                "id": value.id.to_string(), "name": value.name,
                "translation": value.translation, "rotation_degrees": value.rotation_degrees,
                "scale": value.scale
            })
        });
        let controls = presentation.workbench_window_nodes.iter().map(|node| json!({
            "id": node.control_id.as_str(), "text": node.text.as_str(),
            "value": node.value_text.as_str(), "disabled": node.disabled,
            "popup_open": node.popup_open,
            "menu_items": node.menu_items.iter().map(|item| item.to_string()).collect::<Vec<_>>(),
            "frame": [node.frame.x, node.frame.y, node.frame.width, node.frame.height]
        })).collect::<Vec<_>>();
        let game = generation.viewport_images().game().map(|image| {
            let samples = image.rgba.as_ref().map(|rgba| {
                [(0.25_f32, 0.25_f32), (0.75, 0.25), (0.5, 0.5), (0.25, 0.75), (0.75, 0.75)]
                    .into_iter().filter_map(|(x, y)| {
                        let px = (x * image.width as f32) as u32;
                        let py = (y * image.height as f32) as u32;
                        let offset = ((py * image.width + px) * 4) as usize;
                        rgba.get(offset..offset + 4).map(|pixel| json!({"x": x, "y": y, "rgba": pixel}))
                    }).collect::<Vec<_>>()
            });
            json!({"valid": image.is_valid(), "width": image.width, "height": image.height,
                "resource": image.resource_key, "identity": image.play_frame_identity.as_ref().map(|id| format!("{id:?}")),
                "generation": image.play_frame_identity.as_ref().map(|id| id.generation()),
                "cpu_bytes": image.rgba.as_ref().map_or(0, |rgba| rgba.len()), "samples": samples})
        });
        let observation = json!({"version": 1, "project_open": chrome.project_open,
            "status": chrome.status_line, "scene": scene, "inspector": inspector,
            "controls": controls, "game_image": game,
            "document_pane": presentation.host_scene_data.document_dock.pane.kind.as_str(),
            "viewport_frame": [viewport.x, viewport.y, viewport.width, viewport.height],
            "play_mode": format!("{:?}", self.runtime.play_sessions().mode()),
            "play_transition": presentation.play_transition,
            "close_prompt": {"visible": close.visible,
                "discard_frame": [discard.x, discard.y, discard.width, discard.height]}});
        let path = std::path::PathBuf::from(path);
        let temporary = path.with_extension("pending.json");
        let result = serde_json::to_vec(&observation)
            .map_err(std::io::Error::other)
            .and_then(|bytes| std::fs::write(&temporary, bytes))
            .and_then(|()| std::fs::rename(&temporary, &path));
        if let Err(error) = result {
            eprintln!("interaction evidence could not be written: {error}");
        }
    }
}
