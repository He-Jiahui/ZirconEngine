use std::collections::BTreeMap;

use crate::core::editor_event::{EditorEvent, EditorEventTransient};
use crate::core::i18n::{EditorI18nService, EditorLocale};

use super::{
    CommandEvalCtx, EditorCommandDescriptor, EditorCommandMenuProjection, EditorCommandRegistry,
    MenuBarModel, MenuItemModel, MenuModel,
};

const MENU_ORDER: [&str; 7] = [
    "file",
    "edit",
    "selection",
    "play",
    "view",
    "window",
    "help",
];

pub(super) fn menu_bar_model(
    registry: &EditorCommandRegistry,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    context: &CommandEvalCtx,
) -> MenuBarModel {
    let mut roots = BTreeMap::<String, MenuRootProjection>::new();
    for descriptor in registry.commands() {
        let Some(menu_path) = command_menu_path(descriptor) else {
            continue;
        };
        let root = roots
            .entry(menu_path.root().id().as_str().to_owned())
            .or_default();
        root.label.get_or_insert_with(|| {
            descriptor
                .presentation()
                .resolve_key(i18n, locale, menu_path.root().label_key())
                .as_ref()
                .to_owned()
        });
        root.items.insert(descriptor, i18n, locale, context);
    }

    let mut menus = Vec::with_capacity(roots.len());
    for root_id in MENU_ORDER {
        if let Some(root) = roots.remove(root_id) {
            menus.push(root.into_model());
        }
    }
    menus.extend(roots.into_values().map(MenuRootProjection::into_model));
    MenuBarModel { menus }
}

pub(super) fn menu_model(
    registry: &EditorCommandRegistry,
    root_id: &str,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    context: &CommandEvalCtx,
) -> Option<MenuModel> {
    let mut label = None;
    let mut items = MenuLevelProjection::default();
    for descriptor in registry.commands() {
        let Some(path) = command_menu_path(descriptor) else {
            continue;
        };
        if path.root().id().as_str() != root_id {
            continue;
        }
        label.get_or_insert_with(|| {
            descriptor
                .presentation()
                .resolve_key(i18n, locale, path.root().label_key())
                .as_ref()
                .to_owned()
        });
        items.insert(descriptor, i18n, locale, context);
    }

    (!items.is_empty()).then(|| MenuModel {
        label: label.expect("non-empty menu has a localized root label"),
        items: items.into_items(),
    })
}

#[derive(Default)]
struct MenuRootProjection {
    label: Option<String>,
    items: MenuLevelProjection,
}

impl MenuRootProjection {
    fn into_model(self) -> MenuModel {
        MenuModel {
            label: self
                .label
                .expect("non-empty menu bucket has a localized root label"),
            items: self.items.into_items(),
        }
    }
}

#[derive(Default)]
struct MenuLevelProjection {
    leaves: BTreeMap<String, MenuItemModel>,
    groups: BTreeMap<String, MenuGroupProjection>,
}

struct MenuGroupProjection {
    label: String,
    items: MenuLevelProjection,
}

impl MenuLevelProjection {
    fn is_empty(&self) -> bool {
        self.leaves.is_empty() && self.groups.is_empty()
    }

    fn insert(
        &mut self,
        descriptor: &EditorCommandDescriptor,
        i18n: &EditorI18nService,
        locale: &EditorLocale,
        context: &CommandEvalCtx,
    ) {
        let path = descriptor
            .menu_path()
            .expect("menu item projection requires a structured menu path");
        let mut level = self;
        for group in path.groups() {
            let group_id = group.id().as_str().to_owned();
            let group_label = descriptor
                .presentation()
                .resolve_key(i18n, locale, group.label_key())
                .as_ref()
                .to_owned();
            level = &mut level
                .groups
                .entry(group_id)
                .or_insert_with(|| MenuGroupProjection {
                    label: group_label,
                    items: MenuLevelProjection::default(),
                })
                .items;
        }
        level
            .leaves
            .entry(path.leaf().id().to_string())
            .or_insert_with(|| command_menu_item_from_path(descriptor, i18n, locale, context));
    }

    fn into_items(self) -> Vec<MenuItemModel> {
        let mut items = Vec::with_capacity(self.leaves.len() + self.groups.len());
        items.extend(self.leaves.into_values());
        items.extend(
            self.groups
                .into_values()
                .map(|group| MenuItemModel::branch(group.label, group.items.into_items())),
        );
        items
    }
}

fn command_menu_path(
    descriptor: &EditorCommandDescriptor,
) -> Option<&super::EditorCommandMenuPath> {
    if matches!(
        descriptor.event(),
        Some(EditorEvent::Transient(
            EditorEventTransient::OpenCommandPalette
        ))
    ) {
        return None;
    }
    if descriptor.menu_projection() != EditorCommandMenuProjection::CommandRegistry {
        return None;
    }
    descriptor.menu_path()
}

fn menu_bar_slot(top_level: &str) -> Option<usize> {
    MENU_ORDER
        .iter()
        .position(|candidate| *candidate == top_level)
}

fn command_menu_item_from_path(
    descriptor: &EditorCommandDescriptor,
    i18n: &EditorI18nService,
    locale: &EditorLocale,
    context: &CommandEvalCtx,
) -> MenuItemModel {
    let path = descriptor
        .menu_path()
        .expect("menu item projection requires a structured menu path");
    let label = descriptor
        .presentation()
        .resolve_key(i18n, locale, path.leaf().label_key());
    let shortcut = descriptor.default_chord().map(ToString::to_string);

    MenuItemModel::leaf(
        label.as_ref(),
        None,
        Some(descriptor.id().clone()),
        shortcut,
        descriptor.is_enabled(context),
    )
}

#[cfg(test)]
mod performance_tests;
