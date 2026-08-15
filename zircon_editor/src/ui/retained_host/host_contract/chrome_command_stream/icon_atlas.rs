use std::collections::{BTreeMap, BTreeSet};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use super::{ChromeCommand, ChromeCommandKind, ChromeImageUvRect};

const MIN_ICON_ATLAS_PAGE_EDGE: u32 = 64;
const MAX_ICON_ATLAS_PAGE_EDGE: u32 = 512;
const ICON_ATLAS_PADDING: u32 = 1;
const MAX_ATLASED_ICON_EDGE: u32 = 64;
const RGBA_BYTES_PER_PIXEL: usize = 4;
const MAX_ICON_ATLAS_PAGES: usize = 64;
const MAX_ICON_ATLAS_BYTES: usize = 64 * 1024 * 1024;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct IconSourceKey {
    resource_key: String,
    generation: u64,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy, Debug)]
struct IconAtlasSlot {
    page_index: usize,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

struct IconAtlasPage {
    rgba: Arc<[u8]>,
    edge: u32,
    generation: u64,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
    sealed: bool,
    last_used: u64,
}

impl IconAtlasPage {
    fn new(edge: u32) -> Self {
        Self {
            rgba: vec![0; edge as usize * edge as usize * RGBA_BYTES_PER_PIXEL].into(),
            edge,
            generation: 0,
            cursor_x: 0,
            cursor_y: 0,
            row_height: 0,
            sealed: false,
            last_used: 0,
        }
    }
}

#[derive(Default)]
struct EditorIconAtlas {
    pages: Vec<IconAtlasPage>,
    slots: BTreeMap<IconSourceKey, IconAtlasSlot>,
    resident_bytes: usize,
    access_clock: u64,
}

pub(super) fn pack_editor_icons_into_atlas(commands: &mut [ChromeCommand]) {
    let atlas = editor_icon_atlas();
    atlas
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
        .pack(commands);
}

pub(in crate::ui::retained_host) fn invalidate_editor_icon_atlas() {
    if let Some(atlas) = EDITOR_ICON_ATLAS.get() {
        *atlas.lock().unwrap_or_else(|poison| poison.into_inner()) = EditorIconAtlas::default();
    }
}

impl EditorIconAtlas {
    fn pack(&mut self, commands: &mut [ChromeCommand]) {
        let access = self.next_access();
        let mut active_pages = BTreeSet::new();
        for key in commands
            .iter()
            .filter_map(icon_source_from_command)
            .map(|item| item.0)
        {
            let Some(slot) = self.slots.get(&key).copied() else {
                continue;
            };
            if let Some(page) = self.pages.get_mut(slot.page_index) {
                page.last_used = access;
                active_pages.insert(slot.page_index);
            }
        }
        let pending = commands
            .iter()
            .filter_map(icon_source_from_command)
            .filter(|(key, _)| !self.slots.contains_key(key))
            .collect::<BTreeMap<_, _>>();
        let page_edge = preferred_page_edge(pending.keys());
        let mut changed_pages = BTreeSet::new();
        for (key, rgba) in pending {
            let Some(slot) = self.allocate(key.width, key.height, page_edge, access, &active_pages)
            else {
                continue;
            };
            self.write_slot(slot, rgba.as_ref());
            changed_pages.insert(slot.page_index);
            active_pages.insert(slot.page_index);
            self.slots.insert(key, slot);
        }
        for page_index in changed_pages {
            if let Some(page) = self.pages.get_mut(page_index) {
                page.generation = NEXT_ICON_ATLAS_GENERATION.fetch_add(1, Ordering::Relaxed);
                page.sealed = true;
            }
        }

        for command in commands {
            let ChromeCommandKind::Image { payload } = &mut command.kind else {
                continue;
            };
            let key = IconSourceKey {
                resource_key: payload.resource_key.clone(),
                generation: payload.resource_generation,
                width: payload.width,
                height: payload.height,
            };
            let Some(slot) = self.slots.get(&key).copied() else {
                continue;
            };
            let page = &self.pages[slot.page_index];
            payload.resource_key = page_resource_key(slot.page_index);
            payload.resource_generation = page.generation;
            payload.width = page.edge;
            payload.height = page.edge;
            payload.upload_bytes = page.rgba.len() as u64;
            payload.rgba = Some(Arc::clone(&page.rgba));
            payload.atlas_uv = Some(slot.uv(page.edge));
        }
    }

    fn allocate(
        &mut self,
        width: u32,
        height: u32,
        page_edge: u32,
        access: u64,
        active_pages: &BTreeSet<usize>,
    ) -> Option<IconAtlasSlot> {
        for (page_index, page) in self.pages.iter_mut().enumerate() {
            if page.sealed {
                continue;
            }
            if let Some(slot) = page.allocate(page_index, width, height) {
                page.last_used = access;
                return Some(slot);
            }
        }
        let new_page_bytes = icon_atlas_page_bytes(page_edge)?;
        let can_add_page = self.pages.len() < MAX_ICON_ATLAS_PAGES
            && self.resident_bytes.saturating_add(new_page_bytes) <= MAX_ICON_ATLAS_BYTES;
        let page_index = if can_add_page {
            self.pages.len()
        } else {
            let page_index = self
                .pages
                .iter()
                .enumerate()
                .filter(|(page_index, _)| !active_pages.contains(page_index))
                .min_by_key(|(page_index, page)| (page.last_used, *page_index))
                .map(|(page_index, _)| page_index)?;
            self.remove_page_slots(page_index);
            let previous_bytes = self.pages[page_index].rgba.len();
            self.resident_bytes = self.resident_bytes.saturating_sub(previous_bytes);
            page_index
        };
        let mut page = IconAtlasPage::new(page_edge);
        page.last_used = access;
        let slot = page.allocate(page_index, width, height)?;
        self.resident_bytes = self.resident_bytes.saturating_add(page.rgba.len());
        if page_index == self.pages.len() {
            self.pages.push(page);
        } else {
            self.pages[page_index] = page;
        }
        Some(slot)
    }

    fn remove_page_slots(&mut self, page_index: usize) {
        self.slots.retain(|_, slot| slot.page_index != page_index);
    }

    fn next_access(&mut self) -> u64 {
        self.access_clock = self.access_clock.saturating_add(1);
        self.access_clock
    }

    fn write_slot(&mut self, slot: IconAtlasSlot, source: &[u8]) {
        let Some(page) = self.pages.get_mut(slot.page_index) else {
            return;
        };
        let target = Arc::make_mut(&mut page.rgba);
        for source_y in 0..slot.height {
            let source_offset = source_y as usize * slot.width as usize * RGBA_BYTES_PER_PIXEL;
            let target_offset = atlas_pixel_offset(page.edge, slot.x, slot.y + source_y);
            let row_bytes = slot.width as usize * RGBA_BYTES_PER_PIXEL;
            target[target_offset..target_offset + row_bytes]
                .copy_from_slice(&source[source_offset..source_offset + row_bytes]);

            let first = target[target_offset..target_offset + RGBA_BYTES_PER_PIXEL].to_vec();
            let last_offset = target_offset + row_bytes - RGBA_BYTES_PER_PIXEL;
            let last = target[last_offset..last_offset + RGBA_BYTES_PER_PIXEL].to_vec();
            let left_offset =
                atlas_pixel_offset(page.edge, slot.x - ICON_ATLAS_PADDING, slot.y + source_y);
            let right_offset =
                atlas_pixel_offset(page.edge, slot.x + slot.width, slot.y + source_y);
            target[left_offset..left_offset + RGBA_BYTES_PER_PIXEL].copy_from_slice(&first);
            target[right_offset..right_offset + RGBA_BYTES_PER_PIXEL].copy_from_slice(&last);
        }

        let padded_row_bytes =
            (slot.width + ICON_ATLAS_PADDING * 2) as usize * RGBA_BYTES_PER_PIXEL;
        let first_row = atlas_pixel_offset(page.edge, slot.x - ICON_ATLAS_PADDING, slot.y);
        let last_row = atlas_pixel_offset(
            page.edge,
            slot.x - ICON_ATLAS_PADDING,
            slot.y + slot.height - ICON_ATLAS_PADDING,
        );
        let top_row = atlas_pixel_offset(
            page.edge,
            slot.x - ICON_ATLAS_PADDING,
            slot.y - ICON_ATLAS_PADDING,
        );
        let bottom_row =
            atlas_pixel_offset(page.edge, slot.x - ICON_ATLAS_PADDING, slot.y + slot.height);
        let first = target[first_row..first_row + padded_row_bytes].to_vec();
        let last = target[last_row..last_row + padded_row_bytes].to_vec();
        target[top_row..top_row + padded_row_bytes].copy_from_slice(&first);
        target[bottom_row..bottom_row + padded_row_bytes].copy_from_slice(&last);
    }
}

impl IconAtlasPage {
    fn allocate(&mut self, page_index: usize, width: u32, height: u32) -> Option<IconAtlasSlot> {
        let padded_width = width.checked_add(ICON_ATLAS_PADDING * 2)?;
        let padded_height = height.checked_add(ICON_ATLAS_PADDING * 2)?;
        if self.cursor_x + padded_width > self.edge {
            self.cursor_x = 0;
            self.cursor_y = self.cursor_y.checked_add(self.row_height)?;
            self.row_height = 0;
        }
        if self.cursor_y + padded_height > self.edge {
            return None;
        }
        let slot = IconAtlasSlot {
            page_index,
            x: self.cursor_x + ICON_ATLAS_PADDING,
            y: self.cursor_y + ICON_ATLAS_PADDING,
            width,
            height,
        };
        self.cursor_x += padded_width;
        self.row_height = self.row_height.max(padded_height);
        Some(slot)
    }
}

impl IconAtlasSlot {
    fn uv(self, page_edge: u32) -> ChromeImageUvRect {
        let edge = page_edge as f32;
        ChromeImageUvRect {
            min: [self.x as f32 / edge, self.y as f32 / edge],
            max: [
                (self.x + self.width) as f32 / edge,
                (self.y + self.height) as f32 / edge,
            ],
        }
    }
}

fn icon_source_from_command(command: &ChromeCommand) -> Option<(IconSourceKey, Arc<[u8]>)> {
    let ChromeCommandKind::Image { payload } = &command.kind else {
        return None;
    };
    let rgba = payload.rgba.as_ref()?;
    if payload.atlas_uv.is_some()
        || !is_editor_icon_key(payload.resource_key.as_str())
        || payload.width == 0
        || payload.height == 0
        || payload.width > MAX_ATLASED_ICON_EDGE
        || payload.height > MAX_ATLASED_ICON_EDGE
        || rgba.len() != payload.width as usize * payload.height as usize * RGBA_BYTES_PER_PIXEL
    {
        return None;
    }
    Some((
        IconSourceKey {
            resource_key: payload.resource_key.clone(),
            generation: payload.resource_generation,
            width: payload.width,
            height: payload.height,
        },
        Arc::clone(rgba),
    ))
}

fn is_editor_icon_key(resource_key: &str) -> bool {
    resource_key.starts_with("icon:")
        || resource_key.starts_with("icon-raster:")
        || resource_key.starts_with("template-icon:")
        || resource_key.starts_with("missing-icon:")
}

fn preferred_page_edge<'a>(keys: impl Iterator<Item = &'a IconSourceKey>) -> u32 {
    let (area, max_extent) = keys.fold((0_u64, MIN_ICON_ATLAS_PAGE_EDGE), |state, key| {
        let width = key.width.saturating_add(ICON_ATLAS_PADDING * 2);
        let height = key.height.saturating_add(ICON_ATLAS_PADDING * 2);
        (
            state.0.saturating_add(u64::from(width) * u64::from(height)),
            state.1.max(width).max(height),
        )
    });
    let mut edge = MIN_ICON_ATLAS_PAGE_EDGE;
    while edge < max_extent
        || (u64::from(edge) * u64::from(edge) < area && edge < MAX_ICON_ATLAS_PAGE_EDGE)
    {
        edge = edge.saturating_mul(2).min(MAX_ICON_ATLAS_PAGE_EDGE);
    }
    edge
}

fn atlas_pixel_offset(edge: u32, x: u32, y: u32) -> usize {
    (y as usize * edge as usize + x as usize) * RGBA_BYTES_PER_PIXEL
}

fn icon_atlas_page_bytes(edge: u32) -> Option<usize> {
    usize::try_from(edge)
        .ok()?
        .checked_mul(usize::try_from(edge).ok()?)?
        .checked_mul(RGBA_BYTES_PER_PIXEL)
}

fn page_resource_key(page_index: usize) -> String {
    format!("atlas://editor/svg-icons/page-{page_index}")
}

fn editor_icon_atlas() -> &'static Mutex<EditorIconAtlas> {
    EDITOR_ICON_ATLAS.get_or_init(|| Mutex::new(EditorIconAtlas::default()))
}

static EDITOR_ICON_ATLAS: OnceLock<Mutex<EditorIconAtlas>> = OnceLock::new();
static NEXT_ICON_ATLAS_GENERATION: AtomicU64 = AtomicU64::new(1);

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::ui::retained_host::host_contract::chrome_command_stream::{
        ChromeCommandLayer, ChromeImagePayload,
    };
    use crate::ui::retained_host::host_contract::data::FrameRect;

    #[test]
    fn distinct_icons_share_one_stable_atlas_resource_and_keep_distinct_uvs() {
        let mut atlas = EditorIconAtlas::default();
        let mut first_frame = vec![icon("icon:save", 11), icon("template-icon:close", 22)];

        atlas.pack(&mut first_frame);

        let first = payload(&first_frame[0]);
        let second = payload(&first_frame[1]);
        assert_eq!(first.resource_key, second.resource_key);
        assert_eq!(first.resource_generation, second.resource_generation);
        assert_ne!(first.atlas_uv, second.atlas_uv);
        assert!(Arc::ptr_eq(
            first.rgba.as_ref().expect("atlas pixels"),
            second.rgba.as_ref().expect("shared atlas pixels")
        ));

        let generation = first.resource_generation;
        let mut second_frame = vec![icon("icon:save", 11), icon("template-icon:close", 22)];
        atlas.pack(&mut second_frame);
        assert_eq!(payload(&second_frame[0]).resource_generation, generation);
        assert_eq!(payload(&second_frame[1]).resource_generation, generation);
    }

    #[test]
    fn adding_an_icon_keeps_published_pages_immutable() {
        let mut atlas = EditorIconAtlas::default();
        let mut first_frame = vec![icon("icon:save", 11)];
        atlas.pack(&mut first_frame);
        let first_generation = payload(&first_frame[0]).resource_generation;

        let mut second_frame = vec![icon("icon:save", 11), icon("icon:close", 22)];
        atlas.pack(&mut second_frame);

        let existing = payload(&second_frame[0]);
        let added = payload(&second_frame[1]);
        assert_eq!(existing.resource_generation, first_generation);
        assert_ne!(added.resource_generation, first_generation);
        assert_ne!(added.resource_key, existing.resource_key);
    }

    #[test]
    fn changed_icon_content_does_not_advance_an_unrelated_page_generation() {
        let mut atlas = EditorIconAtlas::default();
        let mut first_frame = vec![
            icon("icon-raster:retained-image:save-v1", 11),
            icon("icon-raster:retained-image:close-v1", 22),
        ];
        atlas.pack(&mut first_frame);
        let shared_generation = payload(&first_frame[0]).resource_generation;

        let mut changed_frame = vec![
            icon("icon-raster:retained-image:save-v2", 33),
            icon("icon-raster:retained-image:close-v1", 22),
        ];
        atlas.pack(&mut changed_frame);

        assert_ne!(
            payload(&changed_frame[0]).resource_generation,
            shared_generation
        );
        assert_eq!(
            payload(&changed_frame[1]).resource_generation,
            shared_generation
        );
    }

    #[test]
    fn atlas_pages_are_lru_bounded_without_rekeying_surviving_pages() {
        let mut atlas = EditorIconAtlas::default();
        let mut first_generation = 0;
        for index in 0..=MAX_ICON_ATLAS_PAGES {
            let mut commands = vec![icon(&format!("icon:bounded-{index}"), index as u8)];
            atlas.pack(&mut commands);
            if index == 0 {
                first_generation = payload(&commands[0]).resource_generation;
            }
        }

        assert_eq!(atlas.pages.len(), MAX_ICON_ATLAS_PAGES);
        assert!(atlas.resident_bytes <= MAX_ICON_ATLAS_BYTES);
        assert!(!atlas
            .slots
            .keys()
            .any(|key| key.resource_key == "icon:bounded-0"));

        let mut latest = vec![icon(
            &format!("icon:bounded-{MAX_ICON_ATLAS_PAGES}"),
            MAX_ICON_ATLAS_PAGES as u8,
        )];
        atlas.pack(&mut latest);
        assert_ne!(payload(&latest[0]).resource_generation, first_generation);
    }

    #[test]
    fn ordinary_images_remain_standalone_resources() {
        let mut atlas = EditorIconAtlas::default();
        let mut commands = vec![icon("image:preview", 11)];

        atlas.pack(&mut commands);

        let payload = payload(&commands[0]);
        assert_eq!(payload.resource_key, "image:preview");
        assert!(payload.atlas_uv.is_none());
    }

    fn icon(resource_key: &str, color: u8) -> ChromeCommand {
        ChromeCommand {
            layer: ChromeCommandLayer::Static,
            z_index: 0,
            frame: FrameRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 2.0,
            },
            clip: None,
            kind: ChromeCommandKind::Image {
                payload: ChromeImagePayload {
                    resource_key: resource_key.to_string(),
                    resource_generation: 0,
                    width: 2,
                    height: 2,
                    upload_bytes: 16,
                    rgba: Some(vec![color; 16].into()),
                    atlas_uv: None,
                },
            },
        }
    }

    fn payload(command: &ChromeCommand) -> &ChromeImagePayload {
        let ChromeCommandKind::Image { payload } = &command.kind else {
            panic!("expected image command");
        };
        payload
    }
}
