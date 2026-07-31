use woc_runtime::{
    ClientWindowProjection, InventoryWindowProjection, QuestLogWindowProjection,
    INVENTORY_BAG_SOCKET_COUNT,
};

use crate::{input::ClientGameplayIntent, preferences::OptionsPanelId};

use super::{
    encode_inventory_filter, inventory_primary_action, resolve_live_inventory_index,
    InventoryCategory, InventoryFilter, InventoryInteractionInfo, InventoryInteractionMode,
    InventoryItemView, InventoryPrimaryAction, InventorySort, QuestLogView, QuestLogWindowState,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClientWindowId {
    Inventory,
    QuestLog,
    Settings,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ClientWindowRouteEffect {
    PersistInventoryFilter(String),
    Authority(ClientGameplayIntent),
    InventoryItemAction {
        inventory_index: usize,
        action: InventoryPrimaryAction,
    },
    ShareQuest {
        quest_id: String,
    },
    ConfirmAbandonQuest {
        quest_id: String,
    },
    ResetSettings,
    ShowOptionsMenu,
    Closed(ClientWindowId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClientWindowRouteError {
    UnknownRoute(String),
    MissingTextValue,
    WindowClosed(ClientWindowId),
    EmptyBagSocket { socket: usize },
    StaleInventoryItem { inventory_index: usize },
    MissingItemPresentation { inventory_index: usize },
    NoSelectedQuest,
    UnavailableSettingsPanel(OptionsPanelId),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClientWindowController {
    inventory_open: bool,
    quest_log_open: bool,
    settings_panel: Option<OptionsPanelId>,
    inventory_filter: InventoryFilter,
    quest_log: QuestLogWindowState,
}

impl ClientWindowController {
    pub fn new(stored_inventory_filter: Option<&str>) -> Self {
        Self {
            inventory_open: false,
            quest_log_open: false,
            settings_panel: None,
            inventory_filter: super::decode_inventory_filter(stored_inventory_filter),
            quest_log: QuestLogWindowState::default(),
        }
    }

    pub fn is_open(&self, window: ClientWindowId) -> bool {
        match window {
            ClientWindowId::Inventory => self.inventory_open,
            ClientWindowId::QuestLog => self.quest_log_open,
            ClientWindowId::Settings => self.settings_panel.is_some(),
        }
    }

    pub fn inventory_filter(&self) -> &InventoryFilter {
        &self.inventory_filter
    }

    pub fn selected_quest_id(&self) -> Option<&str> {
        self.quest_log.selected_quest_id()
    }

    pub fn settings_panel(&self) -> Option<OptionsPanelId> {
        self.settings_panel
    }

    pub fn open_inventory(&mut self) {
        self.inventory_open = true;
    }

    pub fn open_quest_log(
        &mut self,
        projection: &QuestLogWindowProjection,
        selected_quest_id: Option<&str>,
    ) -> QuestLogView {
        self.inventory_open = false;
        self.settings_panel = None;
        self.quest_log_open = true;
        if let Some(quest_id) = selected_quest_id {
            self.quest_log.open_with_quest(quest_id);
        }
        self.quest_log.render(projection)
    }

    pub fn select_quest(&mut self, projection: &QuestLogWindowProjection, quest_id: &str) -> bool {
        self.quest_log.select(projection, quest_id)
    }

    pub fn quest_log_view(&mut self, projection: &QuestLogWindowProjection) -> QuestLogView {
        self.quest_log.render(projection)
    }

    pub fn open_settings(
        &mut self,
        panel: OptionsPanelId,
        bug_report_available: bool,
    ) -> Result<(), ClientWindowRouteError> {
        if panel == OptionsPanelId::BugReport && !bug_report_available {
            return Err(ClientWindowRouteError::UnavailableSettingsPanel(panel));
        }
        self.inventory_open = false;
        self.quest_log_open = false;
        self.settings_panel = Some(panel);
        Ok(())
    }

    pub fn activate_inventory_item(
        &self,
        projection: &InventoryWindowProjection,
        clicked: &InventoryItemView,
        mode: InventoryInteractionMode,
    ) -> Result<ClientWindowRouteEffect, ClientWindowRouteError> {
        self.require_open(ClientWindowId::Inventory)?;
        let inventory_index = resolve_live_inventory_index(projection, clicked).ok_or(
            ClientWindowRouteError::StaleInventoryItem {
                inventory_index: usize::from(clicked.inventory_index),
            },
        )?;
        let item = InventoryInteractionInfo::from_view(clicked)
            .ok_or(ClientWindowRouteError::MissingItemPresentation { inventory_index })?;
        Ok(ClientWindowRouteEffect::InventoryItemAction {
            inventory_index,
            action: inventory_primary_action(&item, mode),
        })
    }

    pub fn handle_route(
        &mut self,
        projection: &ClientWindowProjection,
        route: &str,
        text_value: Option<&str>,
    ) -> Result<ClientWindowRouteEffect, ClientWindowRouteError> {
        let route = ParsedRoute::parse(route)?;
        self.require_open(route.window())?;
        match route {
            ParsedRoute::InventoryClose => {
                self.inventory_open = false;
                Ok(ClientWindowRouteEffect::Closed(ClientWindowId::Inventory))
            }
            ParsedRoute::InventoryCategory(category) => {
                self.inventory_filter.category = category;
                Ok(self.persist_inventory_filter())
            }
            ParsedRoute::InventorySort(sort) => {
                self.inventory_filter.sort = sort;
                Ok(self.persist_inventory_filter())
            }
            ParsedRoute::InventorySearch => {
                let text = text_value.ok_or(ClientWindowRouteError::MissingTextValue)?;
                self.inventory_filter.search = text.to_owned();
                Ok(self.persist_inventory_filter())
            }
            ParsedRoute::InventoryBagSocket(socket) => {
                if projection
                    .inventory
                    .bags
                    .get(socket)
                    .and_then(Option::as_ref)
                    .is_none()
                {
                    return Err(ClientWindowRouteError::EmptyBagSocket { socket });
                }
                Ok(ClientWindowRouteEffect::Authority(
                    ClientGameplayIntent::UnequipBag {
                        socket: u32::try_from(socket).expect("the four WOC bag sockets fit a u32"),
                    },
                ))
            }
            ParsedRoute::QuestLogClose => {
                self.quest_log_open = false;
                Ok(ClientWindowRouteEffect::Closed(ClientWindowId::QuestLog))
            }
            ParsedRoute::QuestLogShare => {
                let quest_id = self
                    .quest_log
                    .selected_quest_id()
                    .ok_or(ClientWindowRouteError::NoSelectedQuest)?
                    .to_owned();
                Ok(ClientWindowRouteEffect::ShareQuest { quest_id })
            }
            ParsedRoute::QuestLogAbandon => {
                let quest_id = self
                    .quest_log
                    .selected_quest_id()
                    .ok_or(ClientWindowRouteError::NoSelectedQuest)?
                    .to_owned();
                Ok(ClientWindowRouteEffect::ConfirmAbandonQuest { quest_id })
            }
            ParsedRoute::SettingsBack => {
                self.settings_panel = None;
                Ok(ClientWindowRouteEffect::ShowOptionsMenu)
            }
            ParsedRoute::SettingsReset => Ok(ClientWindowRouteEffect::ResetSettings),
            ParsedRoute::SettingsClose => {
                self.settings_panel = None;
                Ok(ClientWindowRouteEffect::Closed(ClientWindowId::Settings))
            }
        }
    }

    fn require_open(&self, window: ClientWindowId) -> Result<(), ClientWindowRouteError> {
        if self.is_open(window) {
            Ok(())
        } else {
            Err(ClientWindowRouteError::WindowClosed(window))
        }
    }

    fn persist_inventory_filter(&self) -> ClientWindowRouteEffect {
        ClientWindowRouteEffect::PersistInventoryFilter(encode_inventory_filter(
            &self.inventory_filter,
        ))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParsedRoute {
    InventoryClose,
    InventoryCategory(InventoryCategory),
    InventorySort(InventorySort),
    InventorySearch,
    InventoryBagSocket(usize),
    QuestLogClose,
    QuestLogShare,
    QuestLogAbandon,
    SettingsBack,
    SettingsReset,
    SettingsClose,
}

impl ParsedRoute {
    fn parse(route: &str) -> Result<Self, ClientWindowRouteError> {
        let parsed = match route {
            "woc.window.inventory.close" => Some(Self::InventoryClose),
            "woc.window.inventory.search" => Some(Self::InventorySearch),
            "woc.window.quest_log.close" => Some(Self::QuestLogClose),
            "woc.window.quest_log.share" => Some(Self::QuestLogShare),
            "woc.window.quest_log.abandon" => Some(Self::QuestLogAbandon),
            "woc.window.settings.back" => Some(Self::SettingsBack),
            "woc.window.settings.reset" => Some(Self::SettingsReset),
            "woc.window.settings.close" => Some(Self::SettingsClose),
            _ => parse_inventory_route(route),
        };
        parsed.ok_or_else(|| ClientWindowRouteError::UnknownRoute(route.to_owned()))
    }

    fn window(self) -> ClientWindowId {
        match self {
            Self::InventoryClose
            | Self::InventoryCategory(_)
            | Self::InventorySort(_)
            | Self::InventorySearch
            | Self::InventoryBagSocket(_) => ClientWindowId::Inventory,
            Self::QuestLogClose | Self::QuestLogShare | Self::QuestLogAbandon => {
                ClientWindowId::QuestLog
            }
            Self::SettingsBack | Self::SettingsReset | Self::SettingsClose => {
                ClientWindowId::Settings
            }
        }
    }
}

fn parse_inventory_route(route: &str) -> Option<ParsedRoute> {
    if let Some(category) = route.strip_prefix("woc.window.inventory.filter.") {
        return match category {
            "all" => Some(ParsedRoute::InventoryCategory(InventoryCategory::All)),
            "weapon" => Some(ParsedRoute::InventoryCategory(InventoryCategory::Weapon)),
            "armor" => Some(ParsedRoute::InventoryCategory(InventoryCategory::Armor)),
            "consumable" => Some(ParsedRoute::InventoryCategory(
                InventoryCategory::Consumable,
            )),
            "material" => Some(ParsedRoute::InventoryCategory(InventoryCategory::Material)),
            "quest" => Some(ParsedRoute::InventoryCategory(InventoryCategory::Quest)),
            _ => None,
        };
    }
    if let Some(sort) = route.strip_prefix("woc.window.inventory.sort.") {
        return match sort {
            "recent" => Some(ParsedRoute::InventorySort(InventorySort::Recent)),
            "quality" => Some(ParsedRoute::InventorySort(InventorySort::Quality)),
            "name" => Some(ParsedRoute::InventorySort(InventorySort::Name)),
            _ => None,
        };
    }
    let socket = route
        .strip_prefix("woc.window.inventory.bag_socket.")?
        .parse::<usize>()
        .ok()?;
    (socket < INVENTORY_BAG_SOCKET_COUNT).then_some(ParsedRoute::InventoryBagSocket(socket))
}
