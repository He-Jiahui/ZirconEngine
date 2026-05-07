use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderExtract},
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceRenderCache {
    entries: BTreeMap<UiNodeId, UiCachedRenderCommand>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UiCachedRenderCommand {
    command: UiRenderCommand,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct UiRenderCacheUpdate {
    pub extract: UiRenderExtract,
    pub stats: UiSurfaceRenderCacheStats,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct UiSurfaceRenderCacheStats {
    pub reused_command_count: usize,
    pub rebuilt_command_count: usize,
    pub damage_rect_count: usize,
}

impl UiSurfaceRenderCache {
    pub fn update(
        &mut self,
        extract: UiRenderExtract,
        force_rebuild: bool,
    ) -> UiRenderCacheUpdate {
        let mut stats = UiSurfaceRenderCacheStats::default();
        let mut retained_commands = Vec::with_capacity(extract.list.commands.len());
        let mut seen_nodes = BTreeSet::new();
        let mut damage = Vec::new();

        for command in extract.list.commands {
            seen_nodes.insert(command.node_id);
            match self.entries.get(&command.node_id) {
                Some(entry) if !force_rebuild && entry.command == command => {
                    stats.reused_command_count += 1;
                    retained_commands.push(entry.command.clone());
                }
                Some(entry) => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(entry.command.frame, command.frame));
                    self.entries.insert(
                        command.node_id,
                        UiCachedRenderCommand {
                            command: command.clone(),
                        },
                    );
                    retained_commands.push(command);
                }
                None => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, command.frame);
                    self.entries.insert(
                        command.node_id,
                        UiCachedRenderCommand {
                            command: command.clone(),
                        },
                    );
                    retained_commands.push(command);
                }
            }
        }

        let removed_nodes = self
            .entries
            .keys()
            .copied()
            .filter(|node_id| !seen_nodes.contains(node_id))
            .collect::<Vec<_>>();
        for node_id in removed_nodes {
            if let Some(entry) = self.entries.remove(&node_id) {
                push_damage(&mut damage, entry.command.frame);
            }
        }
        stats.damage_rect_count = damage.len();

        UiRenderCacheUpdate {
            extract: UiRenderExtract {
                tree_id: extract.tree_id,
                list: zircon_runtime_interface::ui::surface::UiRenderList {
                    commands: retained_commands,
                },
            },
            stats,
        }
    }
}

fn push_damage(damage: &mut Vec<UiFrame>, frame: UiFrame) {
    if frame.width > 0.0 && frame.height > 0.0 && !damage.contains(&frame) {
        damage.push(frame);
    }
}

fn union_frame(left: UiFrame, right: UiFrame) -> UiFrame {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left.right().max(right.right());
    let bottom_edge = left.bottom().max(right.bottom());
    UiFrame::new(x, y, (right_edge - x).max(0.0), (bottom_edge - y).max(0.0))
}
