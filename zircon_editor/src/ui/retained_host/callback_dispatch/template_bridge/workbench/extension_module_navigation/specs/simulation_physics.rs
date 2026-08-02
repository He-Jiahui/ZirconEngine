use super::types::{ActionControl, ExtensionNavigationSpec, action, spec};

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

const PHYSICS_COLLISION_TAB_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPhysicsCollisionBodiesTab",
    "WorkbenchExtensionPhysicsCollisionMaterialsTab",
    "WorkbenchExtensionPhysicsCollisionContactsTab",
];
const PHYSICS_COLLISION_TAB_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.physics_collision.bodies_tab.select",
        "WorkbenchExtensionPhysicsCollisionBodiesTab",
    ),
    action(
        "workbench.extension.physics_collision.materials_tab.select",
        "WorkbenchExtensionPhysicsCollisionMaterialsTab",
    ),
    action(
        "workbench.extension.physics_collision.contacts_tab.select",
        "WorkbenchExtensionPhysicsCollisionContactsTab",
    ),
];
const PHYSICS_COLLISION_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPhysicsCollisionPlayerCapsuleRow",
    "WorkbenchExtensionPhysicsCollisionMaterialIceRow",
    "WorkbenchExtensionPhysicsCollisionContactWallRow",
    "WorkbenchExtensionPhysicsCollisionPlayerCapsuleTableRow",
    "WorkbenchExtensionPhysicsCollisionIceMaterialTableRow",
    "WorkbenchExtensionPhysicsCollisionWallContactTableRow",
    "WorkbenchExtensionPhysicsCollisionCcdWarningTableRow",
    "WorkbenchExtensionPhysicsCollisionOutputRow",
];
const PHYSICS_COLLISION_ROW_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.physics_collision.player_capsule_row.select",
        "WorkbenchExtensionPhysicsCollisionPlayerCapsuleRow",
    ),
    action(
        "workbench.extension.physics_collision.material_ice_row.select",
        "WorkbenchExtensionPhysicsCollisionMaterialIceRow",
    ),
    action(
        "workbench.extension.physics_collision.contact_wall_row.select",
        "WorkbenchExtensionPhysicsCollisionContactWallRow",
    ),
    action(
        "workbench.extension.physics_collision.player_capsule_table_row.select",
        "WorkbenchExtensionPhysicsCollisionPlayerCapsuleTableRow",
    ),
    action(
        "workbench.extension.physics_collision.ice_material_table_row.select",
        "WorkbenchExtensionPhysicsCollisionIceMaterialTableRow",
    ),
    action(
        "workbench.extension.physics_collision.wall_contact_table_row.select",
        "WorkbenchExtensionPhysicsCollisionWallContactTableRow",
    ),
    action(
        "workbench.extension.physics_collision.ccd_warning_table_row.select",
        "WorkbenchExtensionPhysicsCollisionCcdWarningTableRow",
    ),
    action(
        "workbench.extension.physics_collision.output.select",
        "WorkbenchExtensionPhysicsCollisionOutputRow",
    ),
];
const PHYSICS_COLLISION_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsPhysicsCollisionButton",
    "WorkbenchExtensionPhysicsCollisionSimulateButton",
    "WorkbenchExtensionPhysicsCollisionValidateButton",
];
const PHYSICS_COLLISION_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.physics_collision.open",
        "WorkbenchAssetsPhysicsCollisionButton",
    ),
    action(
        "workbench.extension.physics_collision.simulate.invoke",
        "WorkbenchExtensionPhysicsCollisionSimulateButton",
    ),
    action(
        "workbench.extension.physics_collision.validate.invoke",
        "WorkbenchExtensionPhysicsCollisionValidateButton",
    ),
];
const PHYSICS_COLLISION_FIELD_ACTIONS: &[&str] = &[
    "workbench.extension.physics_collision.profile.edit",
    "workbench.extension.physics_collision.profile.commit",
    "workbench.extension.physics_collision.solver.edit",
    "workbench.extension.physics_collision.solver.commit",
    "workbench.extension.physics_collision.mass.edit",
    "workbench.extension.physics_collision.mass.commit",
];

pub(super) const PHYSICS_COLLISION_NAVIGATION_SPEC: ExtensionNavigationSpec = spec(
    "workbench.extension.physics_collision.open",
    "WorkbenchExtensionPhysicsCollisionWorkspace",
    PHYSICS_COLLISION_TAB_CONTROLS,
    PHYSICS_COLLISION_TAB_ACTIONS,
    PHYSICS_COLLISION_ROW_CONTROLS,
    PHYSICS_COLLISION_ROW_ACTIONS,
    PHYSICS_COLLISION_COMMAND_CONTROLS,
    PHYSICS_COLLISION_COMMAND_ACTIONS,
    PHYSICS_COLLISION_FIELD_ACTIONS,
);
