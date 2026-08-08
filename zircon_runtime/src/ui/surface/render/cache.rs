use std::collections::{BTreeMap, BTreeSet, HashSet};

use serde::{Deserialize, Serialize};
use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiArrangedTree, UiRenderCommand, UiRenderExtract},
};

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UiSurfaceRenderCache {
    entries: BTreeMap<UiNodeId, UiCachedRenderCommandBucket>,
    #[serde(default, skip_serializing, skip_deserializing)]
    command_ranges: BTreeMap<UiNodeId, (usize, usize)>,
    #[serde(default, skip_serializing, skip_deserializing)]
    geometry_patchable_node_ids: BTreeSet<UiNodeId>,
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
        self.geometry_patchable_node_ids.clear();
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

        let extract = UiRenderExtract {
            tree_id: extract.tree_id,
            list: zircon_runtime_interface::ui::surface::UiRenderList {
                commands: retained_commands,
            },
            raster_scale: extract.raster_scale,
        };
        self.reindex_command_ranges(&extract);

        UiRenderCacheUpdate { extract, stats }
    }

    pub fn update_for_arranged(
        &mut self,
        extract: UiRenderExtract,
        force_rebuild: bool,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> UiRenderCacheUpdate {
        let update = self.update(extract, force_rebuild);
        self.refresh_geometry_patchable_nodes(
            &update.extract,
            arranged_tree,
            arranged_node_indices,
        );
        update
    }

    pub fn patch_geometry(
        &mut self,
        extract: &mut UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
    ) -> Result<UiSurfaceRenderCacheStats, ()> {
        if changed_node_ids.is_empty()
            || (!extract.list.commands.is_empty() && self.command_ranges.is_empty())
        {
            return Err(());
        }

        let mut patches = Vec::new();
        for node_id in changed_node_ids {
            if !self.geometry_patchable_node_ids.contains(node_id) {
                return Err(());
            }
            let Some((start, end)) = self.command_ranges.get(node_id).copied() else {
                let Some(node_index) = arranged_node_indices.get(node_id).copied() else {
                    return Err(());
                };
                let Some(node) = arranged_tree
                    .nodes
                    .get(node_index)
                    .filter(|node| node.node_id == *node_id)
                else {
                    return Err(());
                };
                if node.is_render_visible() {
                    return Err(());
                }
                continue;
            };
            let Some(node_index) = arranged_node_indices.get(node_id).copied() else {
                return Err(());
            };
            let Some(node) = arranged_tree
                .nodes
                .get(node_index)
                .filter(|node| node.node_id == *node_id)
            else {
                return Err(());
            };
            let Some(commands) = extract.list.commands.get(start..end) else {
                return Err(());
            };
            let Some(bucket) = self.entries.get(node_id) else {
                return Err(());
            };
            if bucket.len() != commands.len() || bucket.len() != 1 {
                return Err(());
            }
            for (command_index, command) in commands.iter().enumerate() {
                let Some(cached) = bucket.get(command_index as u32) else {
                    return Err(());
                };
                if command.node_id != *node_id || cached.command.node_id != *node_id {
                    return Err(());
                }
                if cached.command.frame.width != node.frame.width
                    || cached.command.frame.height != node.frame.height
                    || cached.command.text_layout.is_some()
                {
                    return Err(());
                }
            }
            patches.push((*node_id, start, end, node.frame, node.clip_frame));
        }

        let mut stats = UiSurfaceRenderCacheStats::default();
        let mut damage = HashSet::new();
        for (node_id, start, end, frame, clip_frame) in patches {
            let commands = extract.list.commands.get_mut(start..end).ok_or(())?;
            let bucket = self.entries.get_mut(&node_id).ok_or(())?;
            for (command_index, command) in commands.iter_mut().enumerate() {
                let cached = bucket.get(command_index as u32).ok_or(())?;
                push_damage(&mut damage, union_frame(cached.command.frame, frame));
                command.frame = frame;
                command.clip_frame = Some(clip_frame);
                bucket.replace_or_append(
                    command_index as u32,
                    UiCachedRenderCommand {
                        command: command.clone(),
                    },
                );
                stats.reused_command_count += 1;
            }
        }
        stats.damage_rect_count = damage.len();
        Ok(stats)
    }

    pub fn patch_nodes(
        &mut self,
        extract: &mut UiRenderExtract,
        changed_node_ids: &std::collections::BTreeSet<UiNodeId>,
        changed_extract: UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> Result<UiSurfaceRenderCacheStats, ()> {
        if changed_node_ids.is_empty()
            || (!extract.list.commands.is_empty() && self.command_ranges.is_empty())
        {
            return Err(());
        }

        let mut next_commands = BTreeMap::<UiNodeId, Vec<UiRenderCommand>>::new();
        for command in changed_extract.list.commands {
            next_commands
                .entry(command.node_id)
                .or_default()
                .push(command);
        }
        if next_commands
            .keys()
            .any(|node_id| !changed_node_ids.contains(node_id))
        {
            return Err(());
        }

        let mut patches = Vec::new();
        for node_id in changed_node_ids {
            let range = self.command_ranges.get(node_id).copied();
            let commands = next_commands.remove(node_id).unwrap_or_default();
            let Some((start, end)) = range else {
                if commands.is_empty() {
                    continue;
                }
                return Err(());
            };
            if commands.len() != end.saturating_sub(start) {
                return Err(());
            }
            let Some(current_commands) = extract.list.commands.get(start..end) else {
                return Err(());
            };
            let Some(bucket) = self.entries.get(node_id) else {
                return Err(());
            };
            if bucket.len() != commands.len() {
                return Err(());
            }
            for (command_index, current) in current_commands.iter().enumerate() {
                let Some(cached) = bucket.get(command_index as u32) else {
                    return Err(());
                };
                if current.node_id != *node_id || cached.command.node_id != *node_id {
                    return Err(());
                }
            }
            patches.push((*node_id, start, end, commands));
        }

        let mut stats = UiSurfaceRenderCacheStats::default();
        let mut damage = HashSet::new();
        for (node_id, start, end, commands) in patches {
            let current_commands = extract.list.commands.get_mut(start..end).ok_or(())?;
            let bucket = self.entries.get_mut(&node_id).ok_or(())?;
            for (command_index, (current, next)) in current_commands
                .iter_mut()
                .zip(commands.into_iter())
                .enumerate()
            {
                let cached = bucket.get(command_index as u32).ok_or(())?;
                if cached.command == next {
                    stats.reused_command_count += 1;
                } else {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(cached.command.frame, next.frame));
                    bucket.replace_or_append(
                        command_index as u32,
                        UiCachedRenderCommand {
                            command: next.clone(),
                        },
                    );
                }
                *current = next;
            }
            self.refresh_geometry_patchable_node(
                node_id,
                extract,
                arranged_tree,
                arranged_node_indices,
            );
        }
        stats.damage_rect_count = damage.len();
        Ok(stats)
    }

    fn reindex_command_ranges(&mut self, extract: &UiRenderExtract) {
        self.command_ranges.clear();
        let mut start = 0;
        while start < extract.list.commands.len() {
            let node_id = extract.list.commands[start].node_id;
            let mut end = start + 1;
            while end < extract.list.commands.len() && extract.list.commands[end].node_id == node_id
            {
                end += 1;
            }
            self.command_ranges.insert(node_id, (start, end));
            start = end;
        }
    }

    fn refresh_geometry_patchable_nodes(
        &mut self,
        extract: &UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        self.geometry_patchable_node_ids.clear();
        for node_id in self.command_ranges.keys().copied().collect::<Vec<_>>() {
            self.refresh_geometry_patchable_node(
                node_id,
                extract,
                arranged_tree,
                arranged_node_indices,
            );
        }
    }

    fn refresh_geometry_patchable_node(
        &mut self,
        node_id: UiNodeId,
        extract: &UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        self.geometry_patchable_node_ids.remove(&node_id);
        let Some((start, end)) = self.command_ranges.get(&node_id).copied() else {
            return;
        };
        if end.saturating_sub(start) != 1 {
            return;
        }
        let Some(command) = extract.list.commands.get(start) else {
            return;
        };
        let Some(arranged_index) = arranged_node_indices.get(&node_id).copied() else {
            return;
        };
        let Some(arranged) = arranged_tree
            .nodes
            .get(arranged_index)
            .filter(|arranged| arranged.node_id == node_id)
        else {
            return;
        };
        if command.frame == arranged.frame
            && command.clip_frame == Some(arranged.clip_frame)
            && command.text_layout.is_none()
        {
            self.geometry_patchable_node_ids.insert(node_id);
        }
    }
}

impl PartialEq for UiSurfaceRenderCache {
    fn eq(&self, other: &Self) -> bool {
        self.entries == other.entries
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
    use std::collections::{BTreeMap, BTreeSet};

    use zircon_runtime_interface::ui::{
        event_ui::{UiNodePath, UiTreeId},
        surface::{UiRenderCommandKind, UiRenderList, UiResolvedStyle},
        tree::{UiInputPolicy, UiVisibility},
    };

    use super::*;

    fn extract(commands: Vec<UiRenderCommand>) -> UiRenderExtract {
        UiRenderExtract {
            tree_id: UiTreeId::new("ui.cache.multi-command"),
            list: UiRenderList { commands },
            raster_scale: 1.0,
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
    fn render_cache_patches_position_only_geometry_without_reextracting() {
        let node_id = UiNodeId::new(11);
        let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.cache.geometry"),
            roots: vec![node_id],
            nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
                node_id,
                node_path: UiNodePath::new("root/node"),
                parent: None,
                children: Vec::new(),
                frame: old_frame,
                clip_frame: old_frame,
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                clip_to_bounds: false,
                control_id: None,
                slot: None,
            }],
            draw_order: vec![node_id],
            canvas_layers: Vec::new(),
        };
        let mut cache = UiSurfaceRenderCache::default();
        let mut owner_command = quad(11, old_frame);
        owner_command.clip_frame = Some(old_frame);
        let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
        let first = cache.update_for_arranged(
            extract(vec![owner_command]),
            false,
            &arranged_tree,
            &arranged_node_indices,
        );
        let mut patched_extract = first.extract;
        arranged_tree.nodes[0].frame.x = 24.0;
        arranged_tree.nodes[0].clip_frame.x = 24.0;

        let stats = cache
            .patch_geometry(
                &mut patched_extract,
                &arranged_tree,
                &arranged_node_indices,
                &BTreeSet::from([node_id]),
            )
            .expect("position-only geometry should patch");

        assert_eq!(stats.rebuilt_command_count, 0);
        assert_eq!(stats.reused_command_count, 1);
        assert_eq!(stats.damage_rect_count, 1);
        assert_eq!(patched_extract.list.commands[0].frame.x, 24.0);
    }

    #[test]
    fn render_cache_rejects_text_layout_geometry_patch_without_mutating_extract() {
        let node_id = UiNodeId::new(14);
        let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.cache.text-geometry"),
            roots: vec![node_id],
            nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
                node_id,
                node_path: UiNodePath::new("root/text"),
                parent: None,
                children: Vec::new(),
                frame: old_frame,
                clip_frame: old_frame,
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                clip_to_bounds: false,
                control_id: None,
                slot: None,
            }],
            draw_order: vec![node_id],
            canvas_layers: Vec::new(),
        };
        let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
        let mut text_command = quad(14, old_frame);
        text_command.clip_frame = Some(old_frame);
        text_command.text_layout = Some(Default::default());
        let mut cache = UiSurfaceRenderCache::default();
        let first = cache.update_for_arranged(
            extract(vec![text_command]),
            false,
            &arranged_tree,
            &arranged_node_indices,
        );
        let mut retained_extract = first.extract;
        let before = retained_extract.clone();
        arranged_tree.nodes[0].frame.x = 24.0;
        arranged_tree.nodes[0].clip_frame.x = 24.0;

        assert_eq!(
            cache.patch_geometry(
                &mut retained_extract,
                &arranged_tree,
                &arranged_node_indices,
                &BTreeSet::from([node_id]),
            ),
            Err(())
        );
        assert_eq!(retained_extract, before);
    }

    #[test]
    fn render_cache_rejects_single_command_with_non_owner_geometry() {
        let node_id = UiNodeId::new(12);
        let owner_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
        let arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.cache.non-owner-geometry"),
            roots: vec![node_id],
            nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
                node_id,
                node_path: UiNodePath::new("root/node"),
                parent: None,
                children: Vec::new(),
                frame: owner_frame,
                clip_frame: owner_frame,
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                clip_to_bounds: false,
                control_id: None,
                slot: None,
            }],
            draw_order: vec![node_id],
            canvas_layers: Vec::new(),
        };
        let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
        let mut command = quad(12, UiFrame::new(6.0, 8.0, 12.0, 16.0));
        command.clip_frame = Some(owner_frame);
        let mut cache = UiSurfaceRenderCache::default();
        let first = cache.update_for_arranged(
            extract(vec![command]),
            false,
            &arranged_tree,
            &arranged_node_indices,
        );
        let mut patched_extract = first.extract;

        assert_eq!(
            cache.patch_geometry(
                &mut patched_extract,
                &arranged_tree,
                &arranged_node_indices,
                &BTreeSet::from([node_id]),
            ),
            Err(())
        );
    }

    #[test]
    fn local_reextract_keeps_exact_owner_command_geometry_patchable() {
        let node_id = UiNodeId::new(13);
        let old_frame = UiFrame::new(4.0, 8.0, 12.0, 16.0);
        let mut arranged_tree = UiArrangedTree {
            tree_id: UiTreeId::new("ui.cache.local-reextract"),
            roots: vec![node_id],
            nodes: vec![zircon_runtime_interface::ui::surface::UiArrangedNode {
                node_id,
                node_path: UiNodePath::new("root/node"),
                parent: None,
                children: Vec::new(),
                frame: old_frame,
                clip_frame: old_frame,
                z_index: 0,
                paint_order: 0,
                visibility: UiVisibility::Visible,
                input_policy: UiInputPolicy::Receive,
                enabled: true,
                clickable: false,
                hoverable: false,
                focusable: false,
                clip_to_bounds: false,
                control_id: None,
                slot: None,
            }],
            draw_order: vec![node_id],
            canvas_layers: Vec::new(),
        };
        let arranged_node_indices = BTreeMap::from([(node_id, 0)]);
        let mut owner_command = quad(13, old_frame);
        owner_command.clip_frame = Some(old_frame);
        let mut cache = UiSurfaceRenderCache::default();
        let first = cache.update_for_arranged(
            extract(vec![owner_command.clone()]),
            false,
            &arranged_tree,
            &arranged_node_indices,
        );
        let mut retained_extract = first.extract;

        cache
            .patch_nodes(
                &mut retained_extract,
                &BTreeSet::from([node_id]),
                extract(vec![owner_command]),
                &arranged_tree,
                &arranged_node_indices,
            )
            .expect("same owner command should reextract locally");
        arranged_tree.nodes[0].frame.x = 24.0;
        arranged_tree.nodes[0].clip_frame.x = 24.0;

        cache
            .patch_geometry(
                &mut retained_extract,
                &arranged_tree,
                &arranged_node_indices,
                &BTreeSet::from([node_id]),
            )
            .expect("local reextract should preserve later geometry patching");
        assert_eq!(retained_extract.list.commands[0].frame.x, 24.0);
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
