use woc_client::{
    bank_deposit_opens_prompt, inventory_destroy_action, inventory_primary_action,
    inventory_shift_links, inventory_tooltip_hint, resolve_deposit_submit,
    resolve_live_inventory_index, InventoryDestroyAction, InventoryInteractionInfo,
    InventoryInteractionMode, InventoryItemKind, InventoryItemPresentation, InventoryItemView,
    InventoryPrimaryAction, InventoryQuality, InventoryTooltipHint,
};
use woc_runtime::{InventoryItemProjection, InventoryWindowProjection};

fn info(kind: InventoryItemKind) -> InventoryInteractionInfo {
    InventoryInteractionInfo {
        kind,
        no_market_list: false,
        has_use_effect: false,
        no_discard: false,
        soulbound: false,
    }
}

fn mode() -> InventoryInteractionMode {
    InventoryInteractionMode::default()
}

#[test]
fn primary_action_preserves_the_target_mode_priority_and_soulbound_gate() {
    let weapon = info(InventoryItemKind::Weapon);
    for (mode, expected) in [
        (
            InventoryInteractionMode {
                trade_open: true,
                mail_attach: true,
                market_sell: true,
                vendor_open: true,
                bank_deposit: true,
                pet_feed: true,
            },
            InventoryPrimaryAction::Trade,
        ),
        (
            InventoryInteractionMode {
                mail_attach: true,
                market_sell: true,
                vendor_open: true,
                bank_deposit: true,
                pet_feed: true,
                ..mode()
            },
            InventoryPrimaryAction::MailAttach,
        ),
        (
            InventoryInteractionMode {
                market_sell: true,
                vendor_open: true,
                bank_deposit: true,
                pet_feed: true,
                ..mode()
            },
            InventoryPrimaryAction::MarketSell,
        ),
        (
            InventoryInteractionMode {
                vendor_open: true,
                bank_deposit: true,
                pet_feed: true,
                ..mode()
            },
            InventoryPrimaryAction::VendorSell,
        ),
        (
            InventoryInteractionMode {
                bank_deposit: true,
                pet_feed: true,
                ..mode()
            },
            InventoryPrimaryAction::BankDeposit,
        ),
    ] {
        assert_eq!(inventory_primary_action(&weapon, mode), expected);
    }

    let mut soulbound = weapon;
    soulbound.soulbound = true;
    for active_mode in [
        InventoryInteractionMode {
            trade_open: true,
            ..mode()
        },
        InventoryInteractionMode {
            mail_attach: true,
            ..mode()
        },
        InventoryInteractionMode {
            market_sell: true,
            ..mode()
        },
        InventoryInteractionMode {
            vendor_open: true,
            ..mode()
        },
    ] {
        assert_eq!(
            inventory_primary_action(&soulbound, active_mode),
            InventoryPrimaryAction::TransferBlockedSoulbound
        );
    }
}

#[test]
fn item_kind_branches_match_mail_market_bank_pet_quest_bag_and_use_behavior() {
    let mut quest = info(InventoryItemKind::Quest);
    assert_eq!(
        inventory_primary_action(
            &quest,
            InventoryInteractionMode {
                mail_attach: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::MailAttachBlocked
    );
    assert_eq!(
        inventory_primary_action(
            &quest,
            InventoryInteractionMode {
                market_sell: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::MarketSellBlockedQuest
    );
    assert_eq!(
        inventory_primary_action(
            &quest,
            InventoryInteractionMode {
                bank_deposit: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::BankDepositBlockedQuest
    );
    assert_eq!(
        inventory_primary_action(&quest, mode()),
        InventoryPrimaryAction::DiscardQuest
    );

    quest.no_market_list = true;
    assert_eq!(
        inventory_primary_action(
            &quest,
            InventoryInteractionMode {
                market_sell: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::MarketSellBlockedQuest
    );
    let mut protected = info(InventoryItemKind::Tool);
    protected.no_market_list = true;
    assert_eq!(
        inventory_primary_action(
            &protected,
            InventoryInteractionMode {
                market_sell: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::MarketSellBlockedNoMarket
    );
    assert_eq!(
        inventory_primary_action(&info(InventoryItemKind::Bag), mode()),
        InventoryPrimaryAction::EquipBag
    );
    assert_eq!(
        inventory_primary_action(
            &info(InventoryItemKind::Food),
            InventoryInteractionMode {
                pet_feed: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::PetFeed
    );
    assert_eq!(
        inventory_primary_action(
            &info(InventoryItemKind::Weapon),
            InventoryInteractionMode {
                pet_feed: true,
                ..mode()
            }
        ),
        InventoryPrimaryAction::PetFeedBlocked
    );
    assert_eq!(
        inventory_primary_action(&info(InventoryItemKind::Potion), mode()),
        InventoryPrimaryAction::Use
    );
}

#[test]
fn destroy_shift_link_and_tooltip_decisions_match_the_reference_affordances() {
    let mut item = info(InventoryItemKind::Weapon);
    assert_eq!(
        inventory_destroy_action(&item, mode()),
        InventoryDestroyAction::Discard
    );
    item.no_discard = true;
    assert_eq!(
        inventory_destroy_action(&item, mode()),
        InventoryDestroyAction::DiscardBlocked
    );
    assert_eq!(
        inventory_destroy_action(
            &item,
            InventoryInteractionMode {
                vendor_open: true,
                ..mode()
            }
        ),
        InventoryDestroyAction::None
    );
    assert!(!inventory_shift_links(InventoryInteractionMode {
        vendor_open: true,
        ..mode()
    }));
    assert!(!inventory_shift_links(InventoryInteractionMode {
        bank_deposit: true,
        ..mode()
    }));
    assert!(inventory_shift_links(mode()));

    assert_eq!(
        inventory_tooltip_hint(&info(InventoryItemKind::Quest), mode()),
        InventoryTooltipHint::ClickDestroy
    );
    assert_eq!(
        inventory_tooltip_hint(&info(InventoryItemKind::HeldOffhand), mode()),
        InventoryTooltipHint::ClickEquip
    );
    assert_eq!(
        inventory_tooltip_hint(&info(InventoryItemKind::Food), mode()),
        InventoryTooltipHint::ClickConsume
    );
    assert_eq!(
        inventory_tooltip_hint(&info(InventoryItemKind::Potion), mode()),
        InventoryTooltipHint::ClickUseInstant
    );
    let mut usable = info(InventoryItemKind::Tool);
    usable.has_use_effect = true;
    assert_eq!(
        inventory_tooltip_hint(&usable, mode()),
        InventoryTooltipHint::ClickUse
    );
    assert_eq!(
        InventoryTooltipHint::ClickUse.translation_key(),
        "itemUi.tooltip.clickUse"
    );
}

fn projected_item(index: u16, item_id: &str, count: u32) -> InventoryItemProjection {
    InventoryItemProjection {
        inventory_index: index,
        cell_hint: Some(index),
        item_id: item_id.into(),
        count,
        instance_id: None,
        soulbound: false,
    }
}

fn clicked(item: &InventoryItemProjection) -> InventoryItemView {
    InventoryItemView {
        inventory_index: item.inventory_index,
        cell_hint: item.cell_hint,
        item_id: item.item_id.clone(),
        count: item.count,
        instance_id: item.instance_id,
        soulbound: item.soulbound,
        presentation: None,
    }
}

#[test]
fn live_index_resolution_and_partial_deposit_refuse_stale_or_indivisible_items() {
    let inventory = InventoryWindowProjection {
        backpack_slots: 4,
        capacity: 4,
        copper: 0,
        bags: vec![None, None, None, None],
        items: vec![
            projected_item(0, "wolf_pelt", 5),
            projected_item(1, "wolf_pelt", 2),
        ],
    };
    let first = clicked(&inventory.items[0]);
    let second = clicked(&inventory.items[1]);
    assert_eq!(resolve_live_inventory_index(&inventory, &first), Some(0));
    assert_eq!(resolve_live_inventory_index(&inventory, &second), Some(1));

    let mut changed = inventory.clone();
    changed.items[0].count = 4;
    assert_eq!(resolve_live_inventory_index(&changed, &first), None);
    assert!(bank_deposit_opens_prompt(&inventory.items[0]));

    let mut instanced = inventory.items[0].clone();
    instanced.instance_id = Some(77);
    assert!(!bank_deposit_opens_prompt(&instanced));
    instanced.instance_id = None;
    instanced.count = 1;
    assert!(!bank_deposit_opens_prompt(&instanced));

    assert_eq!(
        resolve_deposit_submit(Some(&inventory.items[0]), &first, 99, 4),
        Some(4)
    );
    assert_eq!(
        resolve_deposit_submit(Some(&inventory.items[0]), &first, -3, 4),
        Some(1)
    );
    assert_eq!(
        resolve_deposit_submit(Some(&inventory.items[1]), &first, 2, 4),
        Some(2)
    );
    let other = projected_item(0, "iron_ore", 5);
    assert_eq!(resolve_deposit_submit(Some(&other), &first, 2, 4), None);
    assert_eq!(resolve_deposit_submit(None, &first, 2, 4), None);
}

#[test]
fn interaction_facts_compose_local_definition_flags_with_authoritative_soulbinding() {
    let mut view = clicked(&projected_item(0, "fishing_rod", 1));
    view.soulbound = true;
    view.presentation = Some(
        InventoryItemPresentation::new(
            "fishing_rod",
            "Fishing Rod",
            InventoryItemKind::Tool,
            Some(InventoryQuality::Common),
        )
        .with_interaction_flags(true, true, true),
    );

    assert_eq!(
        InventoryInteractionInfo::from_view(&view),
        Some(InventoryInteractionInfo {
            kind: InventoryItemKind::Tool,
            no_market_list: true,
            has_use_effect: true,
            no_discard: true,
            soulbound: true,
        })
    );
    view.presentation = None;
    assert_eq!(InventoryInteractionInfo::from_view(&view), None);
}
