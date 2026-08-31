use std::collections::{BTreeMap, BTreeSet, HashSet};

use zircon_runtime_interface::ui::{
    event_ui::UiNodeId,
    layout::UiFrame,
    surface::{UiArrangedTree, UiRenderCommand, UiRenderExtract},
};

#[derive(Clone, Debug, Default)]
pub struct UiSurfaceRenderCache {
    // Full payload authority stays in UiSurface::render_extract; this derived index only
    // locates prior commands and retains frames for fail-closed damage accounting.
    entries: BTreeMap<UiNodeId, UiCachedRenderCommandBucket>,
    command_ranges: BTreeMap<UiNodeId, (usize, usize)>,
    geometry_patchable_node_ids: BTreeSet<UiNodeId>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct UiCachedRenderCommandMetadata {
    command_index: usize,
    frame: UiFrame,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct UiFrameKey {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug)]
struct UiRenderCommandBuildState {
    command_count: u32,
    range_start: usize,
    range_end: usize,
    contiguous: bool,
}

impl UiRenderCommandBuildState {
    fn new(command_offset: usize) -> Self {
        Self {
            command_count: 0,
            range_start: command_offset,
            range_end: command_offset,
            contiguous: true,
        }
    }

    fn observe(&mut self, command_offset: usize) -> u32 {
        let node_command_index = self.command_count;
        self.command_count = node_command_index
            .checked_add(1)
            .expect("a UI node cannot emit more than u32::MAX render commands");
        if self.range_end != command_offset {
            self.contiguous = false;
        }
        self.range_end = command_offset + 1;
        node_command_index
    }

    fn command_range(self) -> Option<(usize, usize)> {
        self.contiguous
            .then_some((self.range_start, self.range_end))
    }
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

#[derive(Clone, Debug, PartialEq)]
enum UiCachedRenderCommandBucket {
    Single(UiCachedRenderCommandMetadata),
    Multiple(Vec<UiCachedRenderCommandMetadata>),
}

impl UiCachedRenderCommandBucket {
    fn get(&self, node_command_index: u32) -> Option<&UiCachedRenderCommandMetadata> {
        match self {
            Self::Single(command) => (node_command_index == 0).then_some(command),
            Self::Multiple(commands) => commands.get(node_command_index as usize),
        }
    }

    fn replace_or_append(
        &mut self,
        node_command_index: u32,
        command: UiCachedRenderCommandMetadata,
    ) {
        match self {
            Self::Single(current) if node_command_index == 0 => *current = command,
            Self::Single(current) => {
                debug_assert_eq!(node_command_index, 1);
                *self = Self::Multiple(vec![*current, command]);
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
                push_damage(damage, command.frame);
                *self = Self::Multiple(Vec::new());
            }
            Self::Single(_) => {}
            Self::Multiple(commands) => {
                let first_removed_index = first_removed_index as usize;
                for entry in commands.iter().skip(first_removed_index) {
                    push_damage(damage, entry.frame);
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
    pub(crate) fn commands_for_node<'a>(
        &self,
        extract: &'a UiRenderExtract,
        node_id: UiNodeId,
    ) -> Option<(usize, &'a [UiRenderCommand])> {
        let (start, end) = self.command_ranges.get(&node_id).copied()?;
        extract
            .list
            .commands
            .get(start..end)
            .map(|commands| (start, commands))
    }

    pub fn update(
        &mut self,
        previous_extract: &UiRenderExtract,
        extract: UiRenderExtract,
        force_rebuild: bool,
    ) -> UiRenderCacheUpdate {
        self.geometry_patchable_node_ids.clear();
        let mut stats = UiSurfaceRenderCacheStats::default();
        let mut command_build_states = BTreeMap::new();
        let mut damage = HashSet::new();
        let cache_metadata_was_empty = self.entries.is_empty();
        if cache_metadata_was_empty {
            for previous in &previous_extract.list.commands {
                push_damage(&mut damage, previous.frame);
            }
        }

        for (command_offset, command) in extract.list.commands.iter().enumerate() {
            let node_command_index = command_build_states
                .entry(command.node_id)
                .or_insert_with(|| UiRenderCommandBuildState::new(command_offset))
                .observe(command_offset);

            let cached = self
                .entries
                .get(&command.node_id)
                .and_then(|entries| entries.get(node_command_index))
                .copied();
            let previous_command = cached.and_then(|entry| {
                previous_extract
                    .list
                    .commands
                    .get(entry.command_index)
                    .filter(|previous| {
                        previous.node_id == command.node_id && previous.frame == entry.frame
                    })
            });

            match (cached, previous_command) {
                (Some(_), Some(previous)) if !force_rebuild && previous == command => {
                    stats.reused_command_count += 1;
                }
                (Some(_), Some(previous)) => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(previous.frame, command.frame));
                }
                (Some(entry), None) => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(entry.frame, command.frame));
                }
                (None, _) => {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, command.frame);
                }
            }

            let metadata = UiCachedRenderCommandMetadata {
                command_index: command_offset,
                frame: command.frame,
            };
            match self.entries.get_mut(&command.node_id) {
                Some(entries) => entries.replace_or_append(node_command_index, metadata),
                None => {
                    debug_assert_eq!(node_command_index, 0);
                    self.entries.insert(
                        command.node_id,
                        UiCachedRenderCommandBucket::Single(metadata),
                    );
                }
            }
        }

        self.entries.retain(|node_id, entries| {
            let retained_count = command_build_states
                .get(node_id)
                .map_or(0, |state| state.command_count);
            if entries.len() > retained_count as usize {
                entries.remove_from(retained_count, &mut damage);
            }
            !entries.is_empty()
        });
        stats.damage_rect_count = damage.len();
        self.command_ranges = command_build_states
            .into_iter()
            .filter_map(|(node_id, state)| state.command_range().map(|range| (node_id, range)))
            .collect();

        UiRenderCacheUpdate { extract, stats }
    }

    pub fn update_for_arranged(
        &mut self,
        previous_extract: &UiRenderExtract,
        extract: UiRenderExtract,
        force_rebuild: bool,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> UiRenderCacheUpdate {
        let update = self.update(previous_extract, extract, force_rebuild);
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
                if command.node_id != *node_id || cached.command_index != start + command_index {
                    return Err(());
                }
                if command.frame.width != node.frame.width
                    || command.frame.height != node.frame.height
                    || command.text_layout.is_some()
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
                let command_offset = start + command_index;
                if cached.command_index != command_offset {
                    return Err(());
                }
                push_damage(&mut damage, union_frame(command.frame, frame));
                command.frame = frame;
                command.clip_frame = Some(clip_frame);
                bucket.replace_or_append(
                    command_index as u32,
                    UiCachedRenderCommandMetadata {
                        command_index: command_offset,
                        frame: command.frame,
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
                if current.node_id != *node_id || cached.command_index != start + command_index {
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
                let command_offset = start + command_index;
                if cached.command_index != command_offset {
                    return Err(());
                }
                if *current == next {
                    stats.reused_command_count += 1;
                } else {
                    stats.rebuilt_command_count += 1;
                    push_damage(&mut damage, union_frame(current.frame, next.frame));
                }
                *current = next;
                bucket.replace_or_append(
                    command_index as u32,
                    UiCachedRenderCommandMetadata {
                        command_index: command_offset,
                        frame: current.frame,
                    },
                );
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

    fn refresh_geometry_patchable_nodes(
        &mut self,
        extract: &UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        let mut patchable_node_ids = BTreeSet::new();
        for node_id in self.command_ranges.keys().copied() {
            if self.node_is_geometry_patchable(
                node_id,
                extract,
                arranged_tree,
                arranged_node_indices,
            ) {
                patchable_node_ids.insert(node_id);
            }
        }
        self.geometry_patchable_node_ids = patchable_node_ids;
    }

    fn refresh_geometry_patchable_node(
        &mut self,
        node_id: UiNodeId,
        extract: &UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) {
        if self.node_is_geometry_patchable(node_id, extract, arranged_tree, arranged_node_indices) {
            self.geometry_patchable_node_ids.insert(node_id);
        } else {
            self.geometry_patchable_node_ids.remove(&node_id);
        }
    }

    fn node_is_geometry_patchable(
        &self,
        node_id: UiNodeId,
        extract: &UiRenderExtract,
        arranged_tree: &UiArrangedTree,
        arranged_node_indices: &BTreeMap<UiNodeId, usize>,
    ) -> bool {
        let Some((start, end)) = self.command_ranges.get(&node_id).copied() else {
            return false;
        };
        if end.saturating_sub(start) != 1 {
            return false;
        }
        let Some(command) = extract.list.commands.get(start) else {
            return false;
        };
        let Some(arranged_index) = arranged_node_indices.get(&node_id).copied() else {
            return false;
        };
        let Some(arranged) = arranged_tree
            .nodes
            .get(arranged_index)
            .filter(|arranged| arranged.node_id == node_id)
        else {
            return false;
        };
        command.frame == arranged.frame
            && command.clip_frame == Some(arranged.clip_frame)
            && command.text_layout.is_none()
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
#[path = "cache/tests/mod.rs"]
mod tests;
