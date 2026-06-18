use std::collections::BTreeSet;

use super::{ChromeCommandKind, ChromeCommandLayer, ChromeCommandStream};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) struct ChromeCommandStreamStats {
    pub(in crate::ui::retained_host::host_contract) command_count: usize,
    pub(in crate::ui::retained_host::host_contract) static_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) dynamic_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) text_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) image_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) clip_command_count: usize,
    pub(in crate::ui::retained_host::host_contract) image_upload_bytes: u64,
    pub(in crate::ui::retained_host::host_contract) draw_call_count: u64,
}

impl ChromeCommandStream {
    pub(in crate::ui::retained_host::host_contract) fn stats(&self) -> ChromeCommandStreamStats {
        let mut uploaded_image_keys = BTreeSet::new();
        let mut stats = ChromeCommandStreamStats {
            command_count: self.commands().len(),
            ..ChromeCommandStreamStats::default()
        };
        for command in self.commands() {
            match command.layer {
                ChromeCommandLayer::Static => stats.static_command_count += 1,
                ChromeCommandLayer::Dynamic => stats.dynamic_command_count += 1,
                ChromeCommandLayer::Text => stats.text_command_count += 1,
                ChromeCommandLayer::Viewport => stats.dynamic_command_count += 1,
            }
            match &command.kind {
                ChromeCommandKind::Quad { .. }
                | ChromeCommandKind::Border { .. }
                | ChromeCommandKind::Text { .. } => stats.draw_call_count += 1,
                ChromeCommandKind::Image { payload } => {
                    stats.image_command_count += 1;
                    if payload.rgba.is_some()
                        && uploaded_image_keys.insert(payload.resource_key.as_str())
                    {
                        stats.image_upload_bytes = stats
                            .image_upload_bytes
                            .saturating_add(payload.upload_bytes);
                    }
                    stats.draw_call_count += 1;
                }
                ChromeCommandKind::Clip => stats.clip_command_count += 1,
            }
        }
        stats
    }
}
