use std::collections::HashMap;

use serde_json::{Map, Value};
use woc_runtime::{InventoryItemProjection, InventoryWindowProjection};

use crate::preferences::PreferenceStorage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryCategory {
    All,
    Weapon,
    Armor,
    Consumable,
    Material,
    Quest,
}

pub const INVENTORY_CATEGORIES: [InventoryCategory; 6] = [
    InventoryCategory::All,
    InventoryCategory::Weapon,
    InventoryCategory::Armor,
    InventoryCategory::Consumable,
    InventoryCategory::Material,
    InventoryCategory::Quest,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventorySort {
    Recent,
    Quality,
    Name,
}

pub const INVENTORY_SORTS: [InventorySort; 3] = [
    InventorySort::Recent,
    InventorySort::Quality,
    InventorySort::Name,
];

pub const INVENTORY_FILTER_STORAGE_KEY: &str = "woc_bag_filter";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryFilter {
    pub category: InventoryCategory,
    pub sort: InventorySort,
    pub search: String,
}

impl Default for InventoryFilter {
    fn default() -> Self {
        Self {
            category: InventoryCategory::All,
            sort: InventorySort::Recent,
            search: String::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryItemKind {
    Weapon,
    Armor,
    HeldOffhand,
    Food,
    Drink,
    Potion,
    Elixir,
    Junk,
    Tool,
    Quest,
    Bag,
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryQuality {
    Legendary,
    Epic,
    Rare,
    Uncommon,
    Common,
    Poor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryItemPresentation {
    pub item_id: String,
    pub display_name: String,
    pub kind: InventoryItemKind,
    pub quality: Option<InventoryQuality>,
    pub no_market_list: bool,
    pub has_use_effect: bool,
    pub no_discard: bool,
}

impl InventoryItemPresentation {
    pub fn new(
        item_id: impl Into<String>,
        display_name: impl Into<String>,
        kind: InventoryItemKind,
        quality: Option<InventoryQuality>,
    ) -> Self {
        Self {
            item_id: item_id.into(),
            display_name: display_name.into(),
            kind,
            quality,
            no_market_list: false,
            has_use_effect: false,
            no_discard: false,
        }
    }

    pub fn with_interaction_flags(
        mut self,
        no_market_list: bool,
        has_use_effect: bool,
        no_discard: bool,
    ) -> Self {
        self.no_market_list = no_market_list;
        self.has_use_effect = has_use_effect;
        self.no_discard = no_discard;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryItemView {
    pub inventory_index: u16,
    pub cell_hint: Option<u16>,
    pub item_id: String,
    pub count: u32,
    pub instance_id: Option<u64>,
    pub soulbound: bool,
    pub presentation: Option<InventoryItemPresentation>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryBagSocketView {
    pub socket: usize,
    pub item_id: Option<String>,
    pub slots: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryGridState {
    Empty,
    NoMatch,
    Items,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InventoryWindowView {
    pub state: InventoryGridState,
    pub cells: Vec<Option<InventoryItemView>>,
    pub visible: Vec<InventoryItemView>,
    pub empty_cells: usize,
    pub overflow: usize,
    pub manual_order: bool,
    pub show_filter_bar: bool,
    pub backpack_slots: u16,
    pub bag_sockets: Vec<InventoryBagSocketView>,
    pub used: usize,
    pub capacity: u16,
    pub copper: u64,
}

pub fn inventory_filter_is_default(filter: &InventoryFilter) -> bool {
    filter.category == InventoryCategory::All && filter.search.trim().is_empty()
}

pub fn inventory_order_is_manual(filter: &InventoryFilter) -> bool {
    filter.sort == InventorySort::Recent && inventory_filter_is_default(filter)
}

pub fn build_inventory_window_view(
    projection: &InventoryWindowProjection,
    catalog: &[InventoryItemPresentation],
    filter: &InventoryFilter,
) -> InventoryWindowView {
    let catalog = catalog
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect::<HashMap<_, _>>();
    let items = projection
        .items
        .iter()
        .map(|item| item_view(item, catalog.get(item.item_id.as_str()).copied()))
        .collect::<Vec<_>>();

    let query = filter.search.trim().to_lowercase();
    let mut visible = items
        .iter()
        .filter(|item| {
            let Some(presentation) = item.presentation.as_ref() else {
                return false;
            };
            matches_category(presentation.kind, filter.category)
                && (query.is_empty() || presentation.display_name.to_lowercase().contains(&query))
        })
        .cloned()
        .collect::<Vec<_>>();
    match filter.sort {
        InventorySort::Recent => {}
        InventorySort::Quality => visible.sort_by_key(|item| {
            quality_rank(
                item.presentation
                    .as_ref()
                    .and_then(|presentation| presentation.quality),
            )
        }),
        InventorySort::Name => visible.sort_by(|left, right| {
            let left = left
                .presentation
                .as_ref()
                .map_or("", |presentation| presentation.display_name.as_str());
            let right = right
                .presentation
                .as_ref()
                .map_or("", |presentation| presentation.display_name.as_str());
            left.cmp(right)
        }),
    }

    let show_empty_cells = inventory_filter_is_default(filter);
    let capacity = usize::from(projection.capacity);
    let empty_cells = if show_empty_cells {
        capacity.saturating_sub(items.len())
    } else {
        0
    };
    let overflow = items.len().saturating_sub(capacity);
    let manual_order = inventory_order_is_manual(filter);
    let cells = if manual_order {
        layout_inventory_cells(&items, capacity)
    } else {
        Vec::new()
    };
    let state = if items.is_empty() {
        if empty_cells > 0 {
            InventoryGridState::Items
        } else {
            InventoryGridState::Empty
        }
    } else if visible.is_empty() {
        InventoryGridState::NoMatch
    } else {
        InventoryGridState::Items
    };

    InventoryWindowView {
        state,
        cells,
        visible,
        empty_cells,
        overflow,
        manual_order,
        show_filter_bar: !items.is_empty(),
        backpack_slots: projection.backpack_slots,
        bag_sockets: projection
            .bags
            .iter()
            .enumerate()
            .map(|(socket, bag)| InventoryBagSocketView {
                socket,
                item_id: bag.as_ref().map(|bag| bag.item_id.clone()),
                slots: bag.as_ref().map_or(0, |bag| bag.slots),
            })
            .collect(),
        used: items.len(),
        capacity: projection.capacity,
        copper: projection.copper,
    }
}

pub fn encode_inventory_filter(filter: &InventoryFilter) -> String {
    let mut fields = Map::new();
    fields.insert(
        "category".into(),
        Value::String(filter.category.as_str().into()),
    );
    fields.insert("sort".into(), Value::String(filter.sort.as_str().into()));
    fields.insert("search".into(), Value::String(filter.search.clone()));
    Value::Object(fields).to_string()
}

pub fn decode_inventory_filter(raw: Option<&str>) -> InventoryFilter {
    let Some(raw) = raw else {
        return InventoryFilter::default();
    };
    let Ok(Value::Object(fields)) = serde_json::from_str::<Value>(raw) else {
        return InventoryFilter::default();
    };
    InventoryFilter {
        category: fields
            .get("category")
            .and_then(Value::as_str)
            .and_then(InventoryCategory::from_str)
            .unwrap_or(InventoryCategory::All),
        sort: fields
            .get("sort")
            .and_then(Value::as_str)
            .and_then(InventorySort::from_str)
            .unwrap_or(InventorySort::Recent),
        search: fields
            .get("search")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    }
}

pub fn load_inventory_filter<S>(storage: &S) -> InventoryFilter
where
    S: PreferenceStorage,
{
    let raw = storage.read(INVENTORY_FILTER_STORAGE_KEY).ok().flatten();
    decode_inventory_filter(raw.as_deref())
}

pub fn persist_inventory_filter<S>(storage: &S, filter: &InventoryFilter)
where
    S: PreferenceStorage,
{
    let encoded = encode_inventory_filter(filter);
    let _ = storage.write(INVENTORY_FILTER_STORAGE_KEY, &encoded);
}

fn item_view(
    item: &InventoryItemProjection,
    presentation: Option<&InventoryItemPresentation>,
) -> InventoryItemView {
    InventoryItemView {
        inventory_index: item.inventory_index,
        cell_hint: item.cell_hint,
        item_id: item.item_id.clone(),
        count: item.count,
        instance_id: item.instance_id,
        soulbound: item.soulbound,
        presentation: presentation.cloned(),
    }
}

fn layout_inventory_cells(
    items: &[InventoryItemView],
    capacity: usize,
) -> Vec<Option<InventoryItemView>> {
    let mut cells = vec![None; capacity];
    let mut unplaced = Vec::new();
    for item in items.iter().cloned() {
        let hint = item.cell_hint.map(usize::from);
        if let Some(hint) = hint.filter(|hint| *hint < cells.len() && cells[*hint].is_none()) {
            cells[hint] = Some(item);
        } else {
            unplaced.push(item);
        }
    }

    let mut next = 0;
    for item in unplaced {
        while next < cells.len() && cells[next].is_some() {
            next += 1;
        }
        if next >= cells.len() {
            cells.push(Some(item));
        } else {
            cells[next] = Some(item);
        }
    }
    cells
}

fn matches_category(kind: InventoryItemKind, category: InventoryCategory) -> bool {
    match category {
        InventoryCategory::All => true,
        InventoryCategory::Weapon => kind == InventoryItemKind::Weapon,
        InventoryCategory::Armor => {
            matches!(
                kind,
                InventoryItemKind::Armor | InventoryItemKind::HeldOffhand
            )
        }
        InventoryCategory::Consumable => matches!(
            kind,
            InventoryItemKind::Food
                | InventoryItemKind::Drink
                | InventoryItemKind::Potion
                | InventoryItemKind::Elixir
        ),
        InventoryCategory::Material => {
            matches!(kind, InventoryItemKind::Junk | InventoryItemKind::Tool)
        }
        InventoryCategory::Quest => kind == InventoryItemKind::Quest,
    }
}

fn quality_rank(quality: Option<InventoryQuality>) -> u8 {
    match quality.unwrap_or(InventoryQuality::Common) {
        InventoryQuality::Legendary => 0,
        InventoryQuality::Epic => 1,
        InventoryQuality::Rare => 2,
        InventoryQuality::Uncommon => 3,
        InventoryQuality::Common => 4,
        InventoryQuality::Poor => 5,
    }
}

impl InventoryCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::Weapon => "weapon",
            Self::Armor => "armor",
            Self::Consumable => "consumable",
            Self::Material => "material",
            Self::Quest => "quest",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        INVENTORY_CATEGORIES
            .into_iter()
            .find(|category| category.as_str() == value)
    }
}

impl InventorySort {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Recent => "recent",
            Self::Quality => "quality",
            Self::Name => "name",
        }
    }

    fn from_str(value: &str) -> Option<Self> {
        INVENTORY_SORTS
            .into_iter()
            .find(|sort| sort.as_str() == value)
    }
}
