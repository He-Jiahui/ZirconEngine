use std::cell::{Cell, RefCell};
use std::collections::HashMap;

use woc_client::{
    build_inventory_window_view, decode_inventory_filter, encode_inventory_filter,
    inventory_filter_is_default, inventory_order_is_manual, load_inventory_filter,
    persist_inventory_filter, InventoryCategory, InventoryFilter, InventoryGridState,
    InventoryItemKind, InventoryItemPresentation, InventoryQuality, InventorySort,
    PreferenceStorage, INVENTORY_CATEGORIES, INVENTORY_FILTER_STORAGE_KEY, INVENTORY_SORTS,
};
use woc_runtime::{EquippedBagProjection, InventoryItemProjection, InventoryWindowProjection};

fn item(inventory_index: u16, cell_hint: Option<u16>, item_id: &str) -> InventoryItemProjection {
    InventoryItemProjection {
        inventory_index,
        cell_hint,
        item_id: item_id.into(),
        count: 1,
        instance_id: None,
        soulbound: false,
    }
}

fn projection() -> InventoryWindowProjection {
    InventoryWindowProjection {
        backpack_slots: 4,
        capacity: 8,
        copper: 12_345,
        bags: vec![
            Some(EquippedBagProjection {
                item_id: "small_pouch".into(),
                slots: 4,
            }),
            None,
            None,
            None,
        ],
        items: vec![
            item(0, Some(3), "potion"),
            item(1, Some(0), "blade"),
            item(2, Some(0), "pelt"),
            item(3, Some(99), "ghost"),
            item(4, None, "helm"),
        ],
    }
}

fn catalog() -> Vec<InventoryItemPresentation> {
    vec![
        InventoryItemPresentation::new(
            "potion",
            "Minor Healing Potion",
            InventoryItemKind::Potion,
            Some(InventoryQuality::Common),
        ),
        InventoryItemPresentation::new(
            "blade",
            "Redbrook Blade",
            InventoryItemKind::Weapon,
            Some(InventoryQuality::Uncommon),
        ),
        InventoryItemPresentation::new(
            "pelt",
            "Wolf Pelt",
            InventoryItemKind::Junk,
            Some(InventoryQuality::Poor),
        ),
        InventoryItemPresentation::new(
            "helm",
            "Iron Helm",
            InventoryItemKind::Armor,
            Some(InventoryQuality::Rare),
        ),
    ]
}

fn ids(view: &woc_client::InventoryWindowView) -> Vec<&str> {
    view.visible
        .iter()
        .map(|item| item.item_id.as_str())
        .collect()
}

#[test]
fn default_view_keeps_real_cells_and_resolves_legacy_hints_without_losing_items() {
    let view = build_inventory_window_view(&projection(), &catalog(), &InventoryFilter::default());

    assert_eq!(view.state, InventoryGridState::Items);
    assert!(view.manual_order);
    assert_eq!(view.cells.len(), 8);
    assert_eq!(view.cells[0].as_ref().unwrap().item_id, "blade");
    assert_eq!(view.cells[1].as_ref().unwrap().item_id, "pelt");
    assert_eq!(view.cells[2].as_ref().unwrap().item_id, "ghost");
    assert_eq!(view.cells[3].as_ref().unwrap().item_id, "potion");
    assert_eq!(view.cells[4].as_ref().unwrap().item_id, "helm");
    assert_eq!(view.empty_cells, 3);
    assert_eq!(view.overflow, 0);
    assert_eq!(view.used, 5);
    assert_eq!(view.capacity, 8);
    assert_eq!(view.copper, 12_345);
    assert_eq!(view.bag_sockets.len(), 4);
    assert_eq!(view.bag_sockets[0].slots, 4);

    // The target drops unknown definitions from the filtered list but keeps the
    // raw stack in the real-cell layout so asset drift cannot delete an item.
    assert_eq!(ids(&view), vec!["potion", "blade", "pelt", "helm"]);
    assert!(view.cells[2].as_ref().unwrap().presentation.is_none());
}

#[test]
fn categories_search_and_stable_sorts_match_the_reference_bag_filter() {
    let projection = projection();
    let catalog = catalog();

    let filtered = |category, sort, search: &str| {
        build_inventory_window_view(
            &projection,
            &catalog,
            &InventoryFilter {
                category,
                sort,
                search: search.into(),
            },
        )
    };

    assert_eq!(
        ids(&filtered(
            InventoryCategory::Weapon,
            InventorySort::Recent,
            ""
        )),
        vec!["blade"]
    );
    assert_eq!(
        ids(&filtered(
            InventoryCategory::Armor,
            InventorySort::Recent,
            ""
        )),
        vec!["helm"]
    );
    assert_eq!(
        ids(&filtered(
            InventoryCategory::Consumable,
            InventorySort::Recent,
            ""
        )),
        vec!["potion"]
    );
    assert_eq!(
        ids(&filtered(
            InventoryCategory::Material,
            InventorySort::Recent,
            ""
        )),
        vec!["pelt"]
    );
    assert_eq!(
        ids(&filtered(
            InventoryCategory::All,
            InventorySort::Recent,
            "  redBROOK  "
        )),
        vec!["blade"]
    );
    assert_eq!(
        ids(&filtered(
            InventoryCategory::All,
            InventorySort::Quality,
            ""
        )),
        vec!["helm", "blade", "potion", "pelt"]
    );
    assert_eq!(
        ids(&filtered(InventoryCategory::All, InventorySort::Name, "")),
        vec!["helm", "potion", "blade", "pelt"]
    );
}

#[test]
fn every_filtered_or_sorted_view_is_a_derived_list_without_drop_cells() {
    for (filter, expected_empty_cells) in [
        (
            InventoryFilter {
                category: InventoryCategory::Weapon,
                ..InventoryFilter::default()
            },
            0,
        ),
        (
            InventoryFilter {
                sort: InventorySort::Quality,
                ..InventoryFilter::default()
            },
            3,
        ),
        (
            InventoryFilter {
                search: "blade".into(),
                ..InventoryFilter::default()
            },
            0,
        ),
    ] {
        let view = build_inventory_window_view(&projection(), &catalog(), &filter);
        assert!(!view.manual_order);
        assert!(view.cells.is_empty());
        assert_eq!(view.empty_cells, expected_empty_cells);
    }
}

#[test]
fn empty_capacity_and_legacy_overflow_match_the_reference_grid_states() {
    let mut empty = projection();
    empty.items.clear();
    let view = build_inventory_window_view(&empty, &catalog(), &InventoryFilter::default());
    assert_eq!(view.state, InventoryGridState::Items);
    assert_eq!(view.cells, vec![None; 8]);
    assert_eq!(view.empty_cells, 8);
    assert!(!view.show_filter_bar);

    let mut overflow = projection();
    overflow.capacity = 4;
    overflow.backpack_slots = 4;
    overflow.bags = vec![None, None, None, None];
    let view = build_inventory_window_view(&overflow, &catalog(), &InventoryFilter::default());
    assert_eq!(view.cells.len(), 5);
    assert_eq!(view.overflow, 1);
}

#[test]
fn filter_persistence_is_tolerant_and_pins_the_exact_option_order() {
    assert_eq!(
        INVENTORY_CATEGORIES,
        [
            InventoryCategory::All,
            InventoryCategory::Weapon,
            InventoryCategory::Armor,
            InventoryCategory::Consumable,
            InventoryCategory::Material,
            InventoryCategory::Quest,
        ]
    );
    assert_eq!(
        INVENTORY_SORTS,
        [
            InventorySort::Recent,
            InventorySort::Quality,
            InventorySort::Name,
        ]
    );

    let state = InventoryFilter {
        category: InventoryCategory::Armor,
        sort: InventorySort::Name,
        search: "iron".into(),
    };
    assert_eq!(
        decode_inventory_filter(Some(&encode_inventory_filter(&state))),
        state
    );
    assert_eq!(
        decode_inventory_filter(Some("not json")),
        InventoryFilter::default()
    );
    assert_eq!(
        decode_inventory_filter(Some(
            r#"{"category":"weapon","sort":"quality","search":123}"#
        )),
        InventoryFilter {
            category: InventoryCategory::Weapon,
            sort: InventorySort::Quality,
            search: String::new(),
        }
    );
    assert!(inventory_filter_is_default(&InventoryFilter {
        sort: InventorySort::Name,
        search: "   ".into(),
        ..InventoryFilter::default()
    }));
    assert!(!inventory_order_is_manual(&InventoryFilter {
        sort: InventorySort::Name,
        ..InventoryFilter::default()
    }));
}

#[derive(Default)]
struct MemoryStorage {
    values: RefCell<HashMap<String, String>>,
    unavailable: Cell<bool>,
}

impl PreferenceStorage for MemoryStorage {
    type Error = ();

    fn read(&self, key: &str) -> Result<Option<String>, Self::Error> {
        if self.unavailable.get() {
            return Err(());
        }
        Ok(self.values.borrow().get(key).cloned())
    }

    fn write(&self, key: &str, value: &str) -> Result<(), Self::Error> {
        if self.unavailable.get() {
            return Err(());
        }
        self.values
            .borrow_mut()
            .insert(key.to_owned(), value.to_owned());
        Ok(())
    }
}

#[test]
fn inventory_filter_storage_uses_the_target_key_and_degrades_when_unavailable() {
    let storage = MemoryStorage::default();
    storage.values.borrow_mut().insert(
        INVENTORY_FILTER_STORAGE_KEY.into(),
        r#"{"category":"quest","sort":"quality","search":"sigil"}"#.into(),
    );
    assert_eq!(
        load_inventory_filter(&storage),
        InventoryFilter {
            category: InventoryCategory::Quest,
            sort: InventorySort::Quality,
            search: "sigil".into(),
        }
    );

    let changed = InventoryFilter {
        category: InventoryCategory::Material,
        sort: InventorySort::Name,
        search: "ore".into(),
    };
    persist_inventory_filter(&storage, &changed);
    assert_eq!(load_inventory_filter(&storage), changed);

    storage.unavailable.set(true);
    assert_eq!(load_inventory_filter(&storage), InventoryFilter::default());
    persist_inventory_filter(&storage, &InventoryFilter::default());
}
