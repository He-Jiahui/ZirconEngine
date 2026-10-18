use super::ExtensionModuleFeedback;

pub(super) fn feedback(action_id: &str) -> Option<ExtensionModuleFeedback> {
    let feedback = match action_id {
        "workbench.extension.collision_proxy.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.collision_proxy.bake.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.collision_proxy.test_contacts.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.collision_proxy.decimator_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.collision_proxy.channel_mask_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionCollisionProxyOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.physics_collision.open" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.physics_collision.simulate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.physics_collision.validate.invoke" => ExtensionModuleFeedback {
            output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
            status_text: "Not implemented (preview)",
            output_text: "Not implemented (preview)",
        },
        "workbench.extension.physics_collision.player_capsule_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        "workbench.extension.physics_collision.ccd_warning_table_row.select" => {
            ExtensionModuleFeedback {
                output_control_id: "WorkbenchExtensionPhysicsCollisionOutputRow",
                status_text: "Not implemented (preview)",
                output_text: "Not implemented (preview)",
            }
        }
        _ => return None,
    };
    Some(feedback)
}
