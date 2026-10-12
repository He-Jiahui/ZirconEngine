use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.collision_proxy.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Collision proxy opened",
            output_text: "Native extension workspace opened for Proxy_RockCliff",
        },
        "workbench.extension.collision_proxy.bake.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Collision proxy bake queued",
            output_text: "Bake queued   Proxy_RockCliff   18 proxies",
        },
        "workbench.extension.collision_proxy.test_contacts.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Collision contact test queued",
            output_text: "Contact test queued   WorldStatic   9 channels",
        },
        "workbench.extension.collision_proxy.decimator_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
                status_text: "Collision decimator selected",
                output_text: "Selected Decimator   Proxy   42 percent",
            }
        }
        "workbench.extension.collision_proxy.channel_mask_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
                status_text: "Collision channel mask selected",
                output_text: "Selected Channel Mask   WorldStatic   Player block",
            }
        }
        "workbench.extension.physics_collision.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Physics collision opened",
            output_text: "Native extension workspace opened for Body_PlayerCapsule",
        },
        "workbench.extension.physics_collision.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Physics simulation queued",
            output_text: "Simulation queued   124 bodies   32 contacts",
        },
        "workbench.extension.physics_collision.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Physics validation queued",
            output_text: "Validation queued   Character profile   1 warning",
        },
        "workbench.extension.physics_collision.player_capsule_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
                status_text: "Physics body selected",
                output_text: "Selected Player Capsule   Dynamic body   82 kg",
            }
        }
        "workbench.extension.physics_collision.ccd_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
                status_text: "Physics warning selected",
                output_text: "Selected CCD Warning   Projectile layer missing sweep",
            }
        }
        _ => return None,
    };
    Some(feedback)
}
