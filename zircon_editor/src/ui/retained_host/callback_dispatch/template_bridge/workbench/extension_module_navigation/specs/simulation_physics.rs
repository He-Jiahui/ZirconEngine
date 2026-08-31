use super::types::{action, spec, ActionControl, ExtensionNavigationSpec};

const COLLISION_PROXY_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionCollisionProxyRockCliffRow",
    "WorkbenchExtensionCollisionProxyHullProxyARow",
    "WorkbenchExtensionCollisionProxyChannelPlayerRow",
    "WorkbenchExtensionCollisionProxySourceMeshTableRow",
    "WorkbenchExtensionCollisionProxyDecimatorTableRow",
    "WorkbenchExtensionCollisionProxyHullMergeTableRow",
    "WorkbenchExtensionCollisionProxyChannelMaskTableRow",
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
];
const COLLISION_PROXY_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsGameplayToolsMenu",
    "WorkbenchExtensionCollisionProxyBakeButton",
    "WorkbenchExtensionCollisionProxyTestContactsButton",
];
const COLLISION_PROXY_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.collision_proxy.open",
        "WorkbenchAssetsGameplayToolsMenu",
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
    COLLISION_PROXY_ROW_CONTROLS,
    COLLISION_PROXY_ROW_ACTIONS,
    COLLISION_PROXY_COMMAND_CONTROLS,
    COLLISION_PROXY_COMMAND_ACTIONS,
    COLLISION_PROXY_FIELD_ACTIONS,
);

const PHYSICS_COLLISION_ROW_CONTROLS: &[&str] = &[
    "WorkbenchExtensionPhysicsCollisionPlayerCapsuleRow",
    "WorkbenchExtensionPhysicsCollisionMaterialIceRow",
    "WorkbenchExtensionPhysicsCollisionContactWallRow",
    "WorkbenchExtensionPhysicsCollisionPlayerCapsuleTableRow",
    "WorkbenchExtensionPhysicsCollisionIceMaterialTableRow",
    "WorkbenchExtensionPhysicsCollisionWallContactTableRow",
    "WorkbenchExtensionPhysicsCollisionCcdWarningTableRow",
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
];
const PHYSICS_COLLISION_COMMAND_CONTROLS: &[&str] = &[
    "WorkbenchAssetsGameplayToolsMenu",
    "WorkbenchExtensionPhysicsCollisionSimulateButton",
    "WorkbenchExtensionPhysicsCollisionValidateButton",
];
const PHYSICS_COLLISION_COMMAND_ACTIONS: &[ActionControl] = &[
    action(
        "workbench.extension.physics_collision.open",
        "WorkbenchAssetsGameplayToolsMenu",
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
    PHYSICS_COLLISION_ROW_CONTROLS,
    PHYSICS_COLLISION_ROW_ACTIONS,
    PHYSICS_COLLISION_COMMAND_CONTROLS,
    PHYSICS_COLLISION_COMMAND_ACTIONS,
    PHYSICS_COLLISION_FIELD_ACTIONS,
);
