use std::collections::BTreeMap;

use crate::core::commands::EditorCommandRegistry;
use zircon_runtime_interface::ui::binding::UiEventKind;

use super::support::{collect_zui_document_files, editor_asset_root, load_zui_document};

const FROZEN_NON_COMMAND_ROUTE_BINDING_COUNT: usize = 1_464;
const FROZEN_NON_COMMAND_ROUTE_BINDING_HASH: u64 = 0xff00_9acf_d490_db55;
const FNV_1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

struct CanonicalCommandActionBinding {
    path: &'static str,
    binding_id: &'static str,
    command: &'static str,
}

const CANONICAL_COMMAND_ACTION_BINDINGS: &[CanonicalCommandActionBinding] = &[
    CanonicalCommandActionBinding {
        path: "assets_activity.zui",
        binding_id: "AssetSurface/OpenAssetBrowser",
        command: "view.asset_browser.open",
    },
    CanonicalCommandActionBinding {
        path: "host/asset_surface_controls.zui",
        binding_id: "AssetSurface/OpenAssetBrowser",
        command: "view.asset_browser.open",
    },
    CanonicalCommandActionBinding {
        path: "host/inspector_surface_controls.zui",
        binding_id: "InspectorView/DeleteSelected",
        command: "scene.node.delete_selected",
    },
    CanonicalCommandActionBinding {
        path: "host/scene_viewport_toolbar.zui",
        binding_id: "ViewportToolbar/EnterPlayMode",
        command: "runtime.play_mode.enter",
    },
    CanonicalCommandActionBinding {
        path: "host/scene_viewport_toolbar.zui",
        binding_id: "ViewportToolbar/ExitPlayMode",
        command: "runtime.play_mode.exit",
    },
    CanonicalCommandActionBinding {
        path: "components/workbench/shell/workbench_top_toolbar.zui",
        binding_id: "AssetSurface/OpenAssetBrowser",
        command: "view.asset_browser.open",
    },
    CanonicalCommandActionBinding {
        path: "components/workbench/shell/workbench_top_toolbar.zui",
        binding_id: "MenuAction/OpenProject",
        command: "file.project.open",
    },
    CanonicalCommandActionBinding {
        path: "components/workbench/shell/workbench_top_toolbar.zui",
        binding_id: "MenuAction/SaveProject",
        command: "file.project.save",
    },
    CanonicalCommandActionBinding {
        path: "components/workbench/shell/workbench_top_toolbar.zui",
        binding_id: "Run/Play",
        command: "runtime.play_mode.enter",
    },
    CanonicalCommandActionBinding {
        path: "components/workbench/shell/workbench_top_toolbar.zui",
        binding_id: "ViewportToolbar/ExitPlayMode",
        command: "runtime.play_mode.exit",
    },
    CanonicalCommandActionBinding {
        path: "host/workbench_shell.zui",
        binding_id: "WorkbenchMenuBar/OpenProject",
        command: "file.project.open",
    },
    CanonicalCommandActionBinding {
        path: "host/workbench_shell.zui",
        binding_id: "WorkbenchMenuBar/SaveProject",
        command: "file.project.save",
    },
];

const REMOVED_COMMAND_ROUTE_ALIASES: &[(&str, &str)] = &[
    (
        "workbench.asset.open_asset_browser",
        "view.asset_browser.open",
    ),
    ("workbench.play_mode.exit", "runtime.play_mode.exit"),
    ("workbench.project.open", "file.project.open"),
    ("workbench.project.save", "file.project.save"),
    ("workbench.run.play", "runtime.play_mode.enter"),
];

#[test]
fn editor_command_event_bindings_use_registered_action_identity_only() {
    let registry = EditorCommandRegistry::default_workbench();
    let editor_root = editor_asset_root().join("ui/editor");
    let removed_aliases = removed_command_route_aliases(&registry);
    let mut observed_canonical_actions = BTreeMap::new();
    let mut non_command_routes = Vec::new();
    let mut checked_bindings = 0usize;
    let mut checked_command_actions = 0usize;
    let mut offenders = Vec::new();

    for path in collect_zui_document_files(&editor_root) {
        let relative_path = path
            .strip_prefix(&editor_root)
            .expect("collected editor asset must remain under the editor root")
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let document = load_zui_document(&path);
        for (node_id, node) in &document.nodes {
            for (binding_index, binding) in node.events.iter().enumerate() {
                checked_bindings += 1;
                let label = format!(
                    "{} node `{node_id}` event binding #{} id `{}`",
                    path.display(),
                    binding_index + 1,
                    binding.id
                );

                if binding.route.is_some() && binding.action.is_some() {
                    offenders.push(format!(
                        "{label} declares both UiBindingRef.route and UiBindingRef.action"
                    ));
                }

                if let Some(route) = binding.route.as_deref() {
                    if registry.command(route).is_some() {
                        offenders.push(format!(
                            "{label} routes registered editor command `{route}` instead of using UiActionRef.action"
                        ));
                    }
                    if let Some(command) = removed_aliases.get(route) {
                        offenders.push(format!(
                            "{label} uses removed editor-command route alias `{route}` for `{command}` instead of UiActionRef.action"
                        ));
                    } else {
                        non_command_routes.push((
                            relative_path.clone(),
                            node_id.clone(),
                            binding.id.clone(),
                            "binding.route",
                            route.to_string(),
                        ));
                    }
                }

                let Some(action) = binding.action.as_ref() else {
                    continue;
                };
                if action.route.is_some() && action.action.is_some() {
                    offenders.push(format!(
                        "{label} declares both UiActionRef.route and UiActionRef.action"
                    ));
                }
                if let Some(route) = action.route.as_deref() {
                    if registry.command(route).is_some() {
                        offenders.push(format!(
                            "{label} routes registered editor command `{route}` inside UiActionRef instead of using UiActionRef.action"
                        ));
                    }
                    if let Some(command) = removed_aliases.get(route) {
                        offenders.push(format!(
                            "{label} uses removed editor-command route alias `{route}` for `{command}` instead of UiActionRef.action"
                        ));
                    } else {
                        non_command_routes.push((
                            relative_path.clone(),
                            node_id.clone(),
                            binding.id.clone(),
                            "action.route",
                            route.to_string(),
                        ));
                    }
                }

                let Some(action_id) = action.action.as_deref() else {
                    continue;
                };
                if binding.event != UiEventKind::Click {
                    offenders.push(format!(
                        "{label} binds command action `{action_id}` to {:?}; typed editor command actions currently support Click only",
                        binding.event
                    ));
                }
                if !action.payload.is_empty() {
                    offenders.push(format!(
                        "{label} gives command action `{action_id}` a route payload"
                    ));
                }
                *observed_canonical_actions
                    .entry((
                        relative_path.clone(),
                        binding.id.clone(),
                        action_id.to_string(),
                    ))
                    .or_insert(0usize) += 1;
                checked_command_actions += 1;
                if registry.command(action_id).is_none() {
                    offenders.push(format!(
                        "{label} references unregistered editor command action `{action_id}`"
                    ));
                }
            }
        }
    }

    assert!(
        checked_bindings > 0,
        "editor .zui assets should declare event bindings"
    );
    assert!(
        checked_command_actions >= 12,
        "editor command governance must exercise the migrated console command actions"
    );
    non_command_routes.sort();
    assert_eq!(
        non_command_routes.len(),
        FROZEN_NON_COMMAND_ROUTE_BINDING_COUNT,
        "the non-command route allowlist changed; review the owner, path, binding identity, and route before refreshing the frozen inventory"
    );
    assert_eq!(
        stable_route_inventory_hash(&non_command_routes),
        FROZEN_NON_COMMAND_ROUTE_BINDING_HASH,
        "the non-command route allowlist identity changed; runtime navigation routes require an explicit owner review before the inventory may be refreshed"
    );
    let expected_canonical_actions = CANONICAL_COMMAND_ACTION_BINDINGS
        .iter()
        .map(|binding| {
            (
                (
                    binding.path.to_string(),
                    binding.binding_id.to_string(),
                    binding.command.to_string(),
                ),
                1usize,
            )
        })
        .collect::<BTreeMap<_, _>>();
    for expected in expected_canonical_actions.keys() {
        assert_eq!(
            observed_canonical_actions.get(expected),
            Some(&1),
            "canonical editor-command action binding changed: {expected:?}"
        );
    }
    assert!(
        offenders.is_empty(),
        "editor command event bindings must use one registered UiActionRef.action identity and must not retain editor-command route aliases: {offenders:#?}"
    );
}

fn stable_route_inventory_hash(routes: &[(String, String, String, &'static str, String)]) -> u64 {
    let mut hash = FNV_1A_64_OFFSET_BASIS;
    for route in routes {
        for field in [
            route.0.as_str(),
            route.1.as_str(),
            route.2.as_str(),
            route.3,
            route.4.as_str(),
        ] {
            hash = hash_bytes(hash, field.as_bytes());
            hash = hash_byte(hash, 0);
        }
        hash = hash_byte(hash, b'\n');
    }
    hash
}

fn hash_bytes(mut hash: u64, bytes: &[u8]) -> u64 {
    for byte in bytes {
        hash = hash_byte(hash, *byte);
    }
    hash
}

fn hash_byte(hash: u64, byte: u8) -> u64 {
    (hash ^ u64::from(byte)).wrapping_mul(FNV_1A_64_PRIME)
}

fn removed_command_route_aliases(
    registry: &EditorCommandRegistry,
) -> BTreeMap<&'static str, &'static str> {
    REMOVED_COMMAND_ROUTE_ALIASES
        .iter()
        .map(|(route, command)| {
            assert!(
                registry.command(command).is_some(),
                "removed route alias `{}` must resolve to registered command `{}`",
                route,
                command
            );
            (*route, *command)
        })
        .collect()
}
