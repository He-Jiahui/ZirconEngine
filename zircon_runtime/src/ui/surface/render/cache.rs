use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiRenderCommand, UiRenderExtract},
};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct UiSurfaceRenderCache {
    entries: BTreeMap<UiNodeId, UiCachedRenderCommandBucket>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct UiCachedRenderCommand {
    command: UiRenderCommand,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct UiFrameKey {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl From<UiFrame> for UiFrameKey {
    fn from(frame: UiFrame) -> Self {
        Self {
            x: canonical_frame_component(frame.x),
            y: canonical_frame_component(frame.y),
            width: canonical_frame_component(frame.width),
            height: canonical_frame_component(frame.height),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
enum UiCachedRenderCommandBucket {
    // Keep persisted one-command node caches compatible with the prior format.
    Single(UiCachedRenderCommand),
    Multiple(Vec<UiCachedRenderCommand>),
}

impl UiCachedRenderCommandBucket {
    fn get(&self, node_command_index: u32) -> Option<&UiCachedRenderCommand> {
        match self {
            Self::Single(command) => (node_command_index == 0).then_some(command),
            Self::Multiple(commands) => commands.get(node_command_index as usize),
        }
    }

    fn replace_or_append(&mut self, node_command_index: u32, command: UiCachedRenderCommand) {
        match self {
            Self::Single(current) if node_command_index == 0 => *current = command,
            Self::Single(current) => {
                debug_assert_eq!(node_command_index, 1);
                *self = Self::Multiple(vec![current.clone(), command]);
            }
            Self::Multiple(commands) => {
                let command_index = node_command_index as usize;
                if command_index < commands.len() {
                    commands[command_index] = command;
                } else {
                    debug_assert_eq!(command_index, commands.len());
                    commands.push(command);
                }
            }
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Single(_) => 1,
            Self::Multiple(commands) => commands.len(),
        }
    }

    fn remove_from(&mut self, first_removed_index: u32, damage: &mut HashSet<UiFrameKey>) {
        match self {
            Self::Single(command) if first_removed_index == 0 => {
                push_damage(damage, command.command.frame);
                *self = Self::Multiple(Vec::new());
            }
            Self::Single(_) => {}
            Self::Multiple(commands) => {
                let first_removed_index = first_removed_index as usize;
                for entry in commands.iter().skip(first_removed_index) {
                    push_damage(damage, entry.command.frame);
                }
                commands.truncate(first_removed_index);
                if commands.len() == 1 {
                    let command = commands.pop().expect("the command count was checked");
                    *self = Self::Single(command);
                }
            }
        }
    }

    fn is_empty(&self) -> bool {
        matches!(self, Self::Multiple(commands) if commands.is_empty())
    }
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
    pub fn update(&mut self, extract: UiRenderExtract, force_rebuild: bool) -> UiRenderCacheUpdate {
        let mut stats = UiSurfaceRenderCacheStats::default();
        let mut retained_commands = Vec::with_capacity(extract.list.commands.len());
        let mut seen_command_counts = BTreeMap::new();
        let mut damage = HashSet::new();

        for command in extract.list.commands {
            let next_node_command_index =
                seen_command_counts.entry(command.node_id).or_insert(0_u32);
            let node_command_index = *next_node_command_index;
            *next_node_command_index = node_command_index
                .checked_add(1)
                .expect("a UI node cannot emit more than u32::MAX render commands");

            match self
                .entries
                .get(&command.node_id)
                .and_then(|entries| entries.get(node_command_index))
            {
                Some(entry) if !force_rebuild && entry.command == command => {
                    stats.reused_command_count += 1;
                    retained_commands.push(command);
                }
                Some(entry) => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(entry.command.frame, command.frame));
                    self.entries
                        .get_mut(&command.node_id)
                        .expect("the cached command was present")
                        .replace_or_append(
                            node_command_index,
                            UiCachedRenderCommand {
                                command: command.clone(),
                            },
                        );
                    retained_commands.push(command);
                }
                None => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, command.frame);
                    let cached_command = UiCachedRenderCommand {
                        command: command.clone(),
                    };
                    match self.entries.get_mut(&command.node_id) {
                        Some(entries) => {
                            entries.replace_or_append(node_command_index, cached_command)
                        }
                        None => {
                            debug_assert_eq!(node_command_index, 0);
                            self.entries.insert(
                                command.node_id,
                                UiCachedRenderCommandBucket::Single(cached_command),
                            );
                        }
                    }
                    retained_commands.push(command);
                }
            }
        }

        let stale_nodes = self
            .entries
            .iter()
            .filter_map(|(node_id, entries)| {
                let retained_count = *seen_command_counts.get(node_id).unwrap_or(&0);
                (entries.len() > retained_count as usize).then_some((*node_id, retained_count))
            })
            .collect::<Vec<_>>();
        for (node_id, retained_count) in stale_nodes {
            self.entries
                .get_mut(&node_id)
                .expect("the stale cache bucket was present")
                .remove_from(retained_count, &mut damage);
        }
        self.entries.retain(|_, entries| !entries.is_empty());
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

fn push_damage(damage: &mut HashSet<UiFrameKey>, frame: UiFrame) {
    if frame.width > 0.0 && frame.height > 0.0 {
        damage.insert(UiFrameKey::from(frame));
    }
}

fn canonical_frame_component(component: f32) -> u32 {
    if component == 0.0 {
        0
    } else {
        component.to_bits()
    }
}

fn union_frame(left: UiFrame, right: UiFrame) -> UiFrame {
    let x = left.x.min(right.x);
    let y = left.y.min(right.y);
    let right_edge = left.right().max(right.right());
    let bottom_edge = left.bottom().max(right.bottom());
    UiFrame::new(x, y, (right_edge - x).max(0.0), (bottom_edge - y).max(0.0))
}

#[cfg(test)]
mod tests {
    use zircon_runtime_interface::ui::{
        event_ui::UiTreeId,
        surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
    };

    use super::*;

    fn extract(commands: Vec<UiRenderCommand>) -> UiRenderExtract {
        UiRenderExtract {
            tree_id: UiTreeId::new("ui.cache.multi-command"),
            list: UiRenderList { commands },
        }
    }

    fn quad(node_id: u64, frame: UiFrame) -> UiRenderCommand {
        UiRenderCommand {
            node_id: UiNodeId::new(node_id),
            kind: UiRenderCommandKind::Quad,
            frame,
            clip_frame: None,
            z_index: 0,
            style: UiResolvedStyle::default(),
            text_layout: None,
            text: None,
            image: None,
            opacity: 1.0,
        }
    }

    #[test]
    fn render_cache_reuses_each_command_emitted_by_one_node() {
        let commands = vec![
            quad(7, UiFrame::new(4.0, 8.0, 12.0, 16.0)),
            quad(7, UiFrame::new(24.0, 8.0, 12.0, 16.0)),
        ];
        let mut cache = UiSurfaceRenderCache::default();

        let first = cache.update(extract(commands.clone()), false);
        assert_eq!(first.stats.rebuilt_command_count, 2);
        assert_eq!(first.stats.reused_command_count, 0);
        assert_eq!(first.stats.damage_rect_count, 2);

        let stable = cache.update(extract(commands.clone()), false);
        assert_eq!(stable.stats.rebuilt_command_count, 0);
        assert_eq!(stable.stats.reused_command_count, 2);
        assert_eq!(stable.stats.damage_rect_count, 0);
        assert_eq!(stable.extract.list.commands, commands);

        let serialized = serde_json::to_string(&cache).expect("cache should serialize as JSON");
        let mut restored = serde_json::from_str::<UiSurfaceRenderCache>(&serialized)
            .expect("cache should restore");
        let restored_stable = restored.update(extract(commands.clone()), false);
        assert_eq!(restored_stable.stats.rebuilt_command_count, 0);
        assert_eq!(restored_stable.stats.reused_command_count, 2);

        let removed_second_command = cache.update(extract(vec![commands[0].clone()]), false);
        assert_eq!(removed_second_command.stats.rebuilt_command_count, 0);
        assert_eq!(removed_second_command.stats.reused_command_count, 1);
        assert_eq!(removed_second_command.stats.damage_rect_count, 1);
    }

    #[test]
    fn render_cache_deserializes_legacy_single_command_entries() {
        let command = quad(9, UiFrame::new(4.0, 8.0, 12.0, 16.0));
        let legacy_entries = BTreeMap::from([(
            UiNodeId::new(9),
            UiCachedRenderCommand {
                command: command.clone(),
            },
        )]);
        let legacy_json = serde_json::json!({ "entries": legacy_entries });
        let mut cache = serde_json::from_value::<UiSurfaceRenderCache>(legacy_json)
            .expect("legacy cache should deserialize");

        let update = cache.update(extract(vec![command]), false);
        assert_eq!(update.stats.rebuilt_command_count, 0);
        assert_eq!(update.stats.reused_command_count, 1);
    }

    #[test]
    fn render_cache_counts_a_shared_damage_frame_once() {
        let shared_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
        let commands = (0..1_024)
            .map(|node_id| quad(node_id, shared_frame))
            .collect::<Vec<_>>();
        let mut cache = UiSurfaceRenderCache::default();

        let first = cache.update(extract(commands), false);

        assert_eq!(first.stats.rebuilt_command_count, 1_024);
        assert_eq!(first.stats.damage_rect_count, 1);
    }
}
