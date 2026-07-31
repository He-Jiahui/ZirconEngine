use woc_runtime::{InventoryItemProjection, InventoryWindowProjection};

use super::{InventoryItemKind, InventoryItemView};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InventoryInteractionInfo {
    pub kind: InventoryItemKind,
    pub no_market_list: bool,
    pub has_use_effect: bool,
    pub no_discard: bool,
    pub soulbound: bool,
}

impl InventoryInteractionInfo {
    pub fn from_view(view: &InventoryItemView) -> Option<Self> {
        let presentation = view.presentation.as_ref()?;
        Some(Self {
            kind: presentation.kind,
            no_market_list: presentation.no_market_list,
            has_use_effect: presentation.has_use_effect,
            no_discard: presentation.no_discard,
            soulbound: view.soulbound,
        })
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InventoryInteractionMode {
    pub trade_open: bool,
    pub mail_attach: bool,
    pub market_sell: bool,
    pub vendor_open: bool,
    pub bank_deposit: bool,
    pub pet_feed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryPrimaryAction {
    TransferBlockedSoulbound,
    Trade,
    MailAttach,
    MailAttachBlocked,
    MarketSell,
    MarketSellBlockedQuest,
    MarketSellBlockedNoMarket,
    VendorSell,
    BankDeposit,
    BankDepositBlockedQuest,
    PetFeed,
    PetFeedBlocked,
    DiscardQuest,
    EquipBag,
    Use,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryDestroyAction {
    Discard,
    DiscardBlocked,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InventoryTooltipHint {
    Soulbound,
    ClickTradeOffer,
    CannotMarket,
    ClickMarketList,
    CannotVendor,
    ClickSell,
    BankDeposit,
    CannotBankDeposit,
    ClickDestroy,
    ClickEquip,
    ClickConsume,
    ClickUseInstant,
    ClickUse,
    ClickMailAttach,
    CannotMail,
    None,
}

pub fn inventory_primary_action(
    item: &InventoryInteractionInfo,
    mode: InventoryInteractionMode,
) -> InventoryPrimaryAction {
    if item.soulbound
        && (mode.trade_open || mode.mail_attach || mode.market_sell || mode.vendor_open)
    {
        return InventoryPrimaryAction::TransferBlockedSoulbound;
    }
    if mode.trade_open {
        return InventoryPrimaryAction::Trade;
    }
    if mode.mail_attach {
        return if item.kind == InventoryItemKind::Quest || item.no_market_list {
            InventoryPrimaryAction::MailAttachBlocked
        } else {
            InventoryPrimaryAction::MailAttach
        };
    }
    if mode.market_sell {
        return if item.kind == InventoryItemKind::Quest {
            InventoryPrimaryAction::MarketSellBlockedQuest
        } else if item.no_market_list {
            InventoryPrimaryAction::MarketSellBlockedNoMarket
        } else {
            InventoryPrimaryAction::MarketSell
        };
    }
    if mode.vendor_open {
        return InventoryPrimaryAction::VendorSell;
    }
    if mode.bank_deposit {
        return if item.kind == InventoryItemKind::Quest {
            InventoryPrimaryAction::BankDepositBlockedQuest
        } else {
            InventoryPrimaryAction::BankDeposit
        };
    }
    if mode.pet_feed {
        return if item.kind == InventoryItemKind::Food {
            InventoryPrimaryAction::PetFeed
        } else {
            InventoryPrimaryAction::PetFeedBlocked
        };
    }
    if item.kind == InventoryItemKind::Quest {
        return InventoryPrimaryAction::DiscardQuest;
    }
    if item.kind == InventoryItemKind::Bag {
        return InventoryPrimaryAction::EquipBag;
    }
    InventoryPrimaryAction::Use
}

pub fn inventory_shift_links(mode: InventoryInteractionMode) -> bool {
    !mode.vendor_open && !mode.bank_deposit
}

pub fn inventory_destroy_action(
    item: &InventoryInteractionInfo,
    mode: InventoryInteractionMode,
) -> InventoryDestroyAction {
    if mode.trade_open
        || mode.mail_attach
        || mode.market_sell
        || mode.vendor_open
        || mode.pet_feed
        || mode.bank_deposit
    {
        InventoryDestroyAction::None
    } else if item.no_discard {
        InventoryDestroyAction::DiscardBlocked
    } else {
        InventoryDestroyAction::Discard
    }
}

pub fn inventory_tooltip_hint(
    item: &InventoryInteractionInfo,
    mode: InventoryInteractionMode,
) -> InventoryTooltipHint {
    if item.soulbound
        && (mode.trade_open || mode.mail_attach || mode.market_sell || mode.vendor_open)
    {
        return InventoryTooltipHint::Soulbound;
    }
    if mode.trade_open {
        return InventoryTooltipHint::ClickTradeOffer;
    }
    if mode.mail_attach {
        return if item.kind == InventoryItemKind::Quest || item.no_market_list {
            InventoryTooltipHint::CannotMail
        } else {
            InventoryTooltipHint::ClickMailAttach
        };
    }
    if mode.market_sell {
        return if item.kind == InventoryItemKind::Quest || item.no_market_list {
            InventoryTooltipHint::CannotMarket
        } else {
            InventoryTooltipHint::ClickMarketList
        };
    }
    if mode.vendor_open {
        return if item.kind == InventoryItemKind::Quest {
            InventoryTooltipHint::CannotVendor
        } else {
            InventoryTooltipHint::ClickSell
        };
    }
    if mode.bank_deposit {
        return if item.kind == InventoryItemKind::Quest {
            InventoryTooltipHint::CannotBankDeposit
        } else {
            InventoryTooltipHint::BankDeposit
        };
    }
    if item.kind == InventoryItemKind::Quest {
        return InventoryTooltipHint::ClickDestroy;
    }
    if matches!(
        item.kind,
        InventoryItemKind::Weapon
            | InventoryItemKind::Armor
            | InventoryItemKind::HeldOffhand
            | InventoryItemKind::Bag
    ) {
        return InventoryTooltipHint::ClickEquip;
    }
    if matches!(
        item.kind,
        InventoryItemKind::Food | InventoryItemKind::Drink
    ) {
        return InventoryTooltipHint::ClickConsume;
    }
    if item.kind == InventoryItemKind::Potion {
        return InventoryTooltipHint::ClickUseInstant;
    }
    if item.has_use_effect {
        return InventoryTooltipHint::ClickUse;
    }
    InventoryTooltipHint::None
}

pub fn resolve_live_inventory_index(
    inventory: &InventoryWindowProjection,
    clicked: &InventoryItemView,
) -> Option<usize> {
    let index = usize::from(clicked.inventory_index);
    let live = inventory.items.get(index)?;
    item_matches_view(live, clicked).then_some(index)
}

pub fn bank_deposit_opens_prompt(item: &InventoryItemProjection) -> bool {
    item.count > 1 && item.instance_id.is_none()
}

pub fn resolve_deposit_submit(
    live: Option<&InventoryItemProjection>,
    captured: &InventoryItemView,
    requested: i64,
    max_count: u32,
) -> Option<u32> {
    let live = live.filter(|live| live.item_id == captured.item_id)?;
    let requested = u32::try_from(requested.max(1)).unwrap_or(u32::MAX);
    Some(requested.min(max_count).min(live.count).max(1))
}

fn item_matches_view(item: &InventoryItemProjection, view: &InventoryItemView) -> bool {
    item.inventory_index == view.inventory_index
        && item.cell_hint == view.cell_hint
        && item.item_id == view.item_id
        && item.count == view.count
        && item.instance_id == view.instance_id
        && item.soulbound == view.soulbound
}

impl InventoryTooltipHint {
    pub const fn translation_key(self) -> &'static str {
        match self {
            Self::Soulbound => "hudChrome.itemSoulbound",
            Self::ClickTradeOffer => "itemUi.tooltip.clickTradeOffer",
            Self::CannotMarket => "itemUi.tooltip.cannotMarket",
            Self::ClickMarketList => "itemUi.tooltip.clickMarketList",
            Self::CannotVendor => "itemUi.tooltip.cannotVendor",
            Self::ClickSell => "itemUi.tooltip.clickSell",
            Self::BankDeposit => "hudChrome.bank.depositHint",
            Self::CannotBankDeposit => "hudChrome.bank.cannotDeposit",
            Self::ClickDestroy => "itemUi.tooltip.clickDestroy",
            Self::ClickEquip => "itemUi.tooltip.clickEquip",
            Self::ClickConsume => "itemUi.tooltip.clickConsume",
            Self::ClickUseInstant => "itemUi.tooltip.clickUseInstant",
            Self::ClickUse => "itemUi.tooltip.clickUse",
            Self::ClickMailAttach => "hudChrome.mailbox.clickAttach",
            Self::CannotMail => "hudChrome.mailbox.cannotMail",
            Self::None => "",
        }
    }
}
