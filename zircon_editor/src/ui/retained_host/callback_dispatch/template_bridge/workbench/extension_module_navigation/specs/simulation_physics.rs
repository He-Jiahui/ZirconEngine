use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const COLLISION_PROXY_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionCollisionProxyProxyTab",
    "WorkbenchExtensionCollisionProxyChannelsTab",
    "WorkbenchExtensionCollisionProxyContactsTab",
];
const COLLISION_PROXY_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.collision_proxy.proxy_tab.select",
        "WorkbenchExtensionCollisionProxyProxyTab",
    ),
    action(
        "workbench.extension.collision_proxy.channels_tab.select",
        "WorkbenchExtensionCollisionProxyChannelsTab",
    ),
    action(
        "workbench.extension.collision_proxy.contacts_tab.select",
        "WorkbenchExtensionCollisionProxyContactsTab",
    ),
];
const COLLISION_PROXY_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionCollisionProxyRockCliffRow",
    "WorkbenchExtensionCollisionProxyHullProxyARow",
    "WorkbenchExtensionCollisionProxyChannelPlayerRow",
    "WorkbenchExtensionCollisionProxySourceMeshTableRow",
    "WorkbenchExtensionCollisionProxyDecimatorTableRow",
    "WorkbenchExtensionCollisionProxyHullMergeTableRow",
    "WorkbenchExtensionCollisionProxyChannelMaskTableRow",
    "WorkbenchExtensionCollisionProxyOutputRow",
];
const COLLISION_PROXY_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.collision_proxy.rock_cliff_row.select",
        "WorkbenchExtensionCollisionProxyRockCliffRow",
    ),
    action(
        "workbench.extension.collision_proxy.hull_proxy_a_row.select",
        "WorkbenchExtensionCollisionProxyHullProxyARow",
    ),
    action(
        "workbench.extension.collision_proxy.channel_player_row.select",
        "WorkbenchExtensionCollisionProxyChannelPlayerRow",
    ),
    action(
        "workbench.extension.collision_proxy.source_mesh_table_row.select",
        "WorkbenchExtensionCollisionProxySourceMeshTableRow",
    ),
    action(
        "workbench.extension.collision_proxy.decimator_table_row.select",
        "WorkbenchExtensionCollisionProxyDecimatorTableRow",
    ),
    action(
        "workbench.extension.collision_proxy.hull_merge_table_row.select",
        "WorkbenchExtensionCollisionProxyHullMergeTableRow",
    ),
    action(
        "workbench.extension.collision_proxy.channel_mask_table_row.select",
        "WorkbenchExtensionCollisionProxyChannelMaskTableRow",
    ),
    action(
        "workbench.extension.collision_proxy.output.select",
        "WorkbenchExtensionCollisionProxyOutputRow",
    ),
];
const COLLISION_PROXY_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsCollisionProxyButton",
    "WorkbenchExtensionCollisionProxyBakeButton",
    "WorkbenchExtensionCollisionProxyTestContactsButton",
];
const COLLISION_PROXY_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.collision_proxy.open",
        "WorkbenchAssetsCollisionProxyButton",
    ),
    action(
        "workbench.extension.collision_proxy.bake.invoke",
        "WorkbenchExtensionCollisionProxyBakeButton",
    ),
    action(
        "workbench.extension.collision_proxy.test_contacts.invoke",
        "WorkbenchExtensionCollisionProxyTestContactsButton",
    ),
];
const COLLISION_PROXY_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.collision_proxy.proxy.edit",
    "workbench.extension.collision_proxy.proxy.commit",
    "workbench.extension.collision_proxy.channel.edit",
    "workbench.extension.collision_proxy.channel.commit",
    "workbench.extension.collision_proxy.lod.edit",
    "workbench.extension.collision_proxy.lod.commit",
];

pub(super) const COLLISION_PROXY_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.collision_proxy.open",
    "WorkbenchExtensionCollisionProxyWorkspace",
    COLLISION_PROXY_TAB_CONTROLS,
    COLLISION_PROXY_TAB_ACTIONS,
    COLLISION_PROXY_ROW_CONTROLS,
    COLLISION_PROXY_ROW_ACTIONS,
    COLLISION_PROXY_COMMAND_CONTROLS,
    COLLISION_PROXY_COMMAND_ACTIONS,
    COLLISION_PROXY_FIELD_ACTIONS,
);
