---
related_code:
  - zircon_hub/assets/brand/zircon-mark.svg
  - zircon_hub/assets/icons/nav/projects.svg
  - zircon_hub/assets/icons/actions/build-project.svg
  - zircon_hub/assets/icons/status/running.svg
  - zircon_hub/assets/icons/ui/search.svg
  - zircon_hub/assets/covers/project-elysium.svg
  - zircon_editor/assets/icons/ionicons/add-outline.svg
  - zircon_editor/assets/icons/zircon_editor_shell/toolbar/menu.svg
  - zircon_editor/assets/icons/zircon_engine_style/assets/mesh.svg
  - zircon_editor/assets/preview/editor-scifi-room.svg
implementation_files:
  - docs/zircon_editor/assets/icon-resource-audit.md
  - docs/zircon_editor/assets/icon-resource-audit.json
plan_sources:
  - user: 2026-05-18 continue evaluating and organizing icon resources
  - docs/superpowers/plans/2026-05-18-icon-resource-audit.md
tests:
  - Glob inventory counts for Hub and Editor SVG paths
  - Grep external-reference scan for href/image/http references
  - Grep non-ASCII scan for generated Zircon-owned SVG packs
  - python -m json.tool docs/zircon_editor/assets/icon-resource-audit.json
doc_type: module-detail
---

# Icon Resource Audit

## Scope

This audit covers every SVG under `zircon_hub/assets`, `zircon_editor/assets/icons`, and `zircon_editor/assets/preview`, including the existing Ionicons pack. It is an inventory only: no assets were moved, renamed, deleted, rewritten, or wired into UI code.

Audited totals: Hub `44`, Editor icons `142`, Editor preview `1`, total `187`.

Tooling identity policy: `path` is the canonical unique key for an audited icon record. `pack/category/name` is the human qualified name used for review, grouping, and collision discussion.

## Pack Summary

| Pack | Root | Count | Owner | Primary Use | Current Status | Notes |
| --- | --- | ---: | --- | --- | --- | --- |
| hub-brand | `zircon_hub/assets/brand` | 1 | zircon-owned | Hub brand mark | wired | Loaded by Hub UI media paths. |
| hub-icons-nav | `zircon_hub/assets/icons/nav` | 9 | zircon-owned | Hub navigation | wired | Names overlap with generic UI terms. |
| hub-icons-actions | `zircon_hub/assets/icons/actions` | 4 | zircon-owned | Hub action cards/buttons | wired | Action-specific names reduce collisions. |
| hub-icons-status | `zircon_hub/assets/icons/status` | 4 | zircon-owned | Hub status indicators | wired | Shares status stems with editor shell. |
| hub-icons-ui | `zircon_hub/assets/icons/ui` | 21 | zircon-owned | Hub chrome controls | wired | Generic UI stems need namespace care. |
| hub-covers | `zircon_hub/assets/covers` | 5 | zircon-owned | Hub project cover art | wired | Larger SVG illustrations, not toolbar icons. |
| editor-ionicons | `zircon_editor/assets/icons/ionicons` | 31 | third-party-legacy | Retained-host legacy/editor UI icons | mixed | 29 Ionicons have production, template, fixture, or test references; `menu-outline` and `ellipsis-horizontal-outline` are asset-only. |
| editor-shell | `zircon_editor/assets/icons/zircon_editor_shell` | 56 | zircon-owned | Editor shell chrome and panels | asset-only | Generated pack, not currently wired. |
| editor-engine-style | `zircon_editor/assets/icons/zircon_engine_style` | 55 | zircon-owned | Future engine-domain icon catalog | asset-only | Reference-informed generated pack. |
| editor-preview | `zircon_editor/assets/preview` | 1 | zircon-owned | Editor preview illustration | asset-only | Preview mock image only. |

## File-Level Inventory

### hub-brand

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/brand/zircon-mark.svg` | zircon-mark | brand | zircon-owned | wired | Hub brand asset. |

### hub-icons-nav

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/icons/nav/settings.svg` | settings | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/learn.svg` | learn | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/team.svg` | team | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/cloud.svg` | cloud | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/plugins.svg` | plugins | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/builds.svg` | builds | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/assets.svg` | assets | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/editor.svg` | editor | nav | zircon-owned | wired | Hub navigation icon. |
| `zircon_hub/assets/icons/nav/projects.svg` | projects | nav | zircon-owned | wired | Hub navigation icon. |

### hub-icons-actions

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/icons/actions/open-editor.svg` | open-editor | actions | zircon-owned | wired | Hub action icon. |
| `zircon_hub/assets/icons/actions/package-project.svg` | package-project | actions | zircon-owned | wired | Hub action icon. |
| `zircon_hub/assets/icons/actions/install-device.svg` | install-device | actions | zircon-owned | wired | Hub action icon. |
| `zircon_hub/assets/icons/actions/build-project.svg` | build-project | actions | zircon-owned | wired | Hub action icon. |

### hub-icons-status

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/icons/status/error.svg` | error | status | zircon-owned | wired | Hub status icon. |
| `zircon_hub/assets/icons/status/warning.svg` | warning | status | zircon-owned | wired | Hub status icon. |
| `zircon_hub/assets/icons/status/success.svg` | success | status | zircon-owned | wired | Hub status icon. |
| `zircon_hub/assets/icons/status/running.svg` | running | status | zircon-owned | wired | Hub status icon. |

### hub-icons-ui

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/icons/ui/edit.svg` | edit | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/alert.svg` | alert | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/collapse.svg` | collapse | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/refresh.svg` | refresh | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/more-vertical.svg` | more-vertical | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/close.svg` | close | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/maximize.svg` | maximize | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/minimize.svg` | minimize | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/settings.svg` | settings | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/help.svg` | help | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/bell.svg` | bell | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/sort.svg` | sort | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/folder.svg` | folder | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/list.svg` | list | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/grid.svg` | grid | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/import.svg` | import | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/plus.svg` | plus | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/chevron-right.svg` | chevron-right | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/chevron-left.svg` | chevron-left | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/chevron-down.svg` | chevron-down | ui | zircon-owned | wired | Hub UI control icon. |
| `zircon_hub/assets/icons/ui/search.svg` | search | ui | zircon-owned | wired | Hub UI control icon. |

### hub-covers

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_hub/assets/covers/project-neon-streets.svg` | project-neon-streets | covers | zircon-owned | wired | Hub cover illustration. |
| `zircon_hub/assets/covers/project-whispering-woods.svg` | project-whispering-woods | covers | zircon-owned | wired | Hub cover illustration. |
| `zircon_hub/assets/covers/project-sands-of-time.svg` | project-sands-of-time | covers | zircon-owned | wired | Hub cover illustration. |
| `zircon_hub/assets/covers/project-stellar-outpost.svg` | project-stellar-outpost | covers | zircon-owned | wired | Hub cover illustration. |
| `zircon_hub/assets/covers/project-elysium.svg` | project-elysium | covers | zircon-owned | wired | Hub cover illustration. |

### editor-ionicons

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_editor/assets/icons/ionicons/share-outline.svg` | share-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings or tests. |
| `zircon_editor/assets/icons/ionicons/save-outline.svg` | save-outline | ionicons | third-party-legacy | wired | Referenced by production templates, Rust mappings, or tests. |
| `zircon_editor/assets/icons/ionicons/refresh-outline.svg` | refresh-outline | ionicons | third-party-legacy | wired | Referenced by UI legacy test fixtures. |
| `zircon_editor/assets/icons/ionicons/git-network-outline.svg` | git-network-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/git-branch-outline.svg` | git-branch-outline | ionicons | third-party-legacy | wired | Referenced by UI legacy test fixtures. |
| `zircon_editor/assets/icons/ionicons/terminal-outline.svg` | terminal-outline | ionicons | third-party-legacy | wired | Referenced by production templates and Rust mappings. |
| `zircon_editor/assets/icons/ionicons/sync-outline.svg` | sync-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/scan-outline.svg` | scan-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/resize-outline.svg` | resize-outline | ionicons | third-party-legacy | wired | Referenced by production UI templates. |
| `zircon_editor/assets/icons/ionicons/remove-outline.svg` | remove-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/play-outline.svg` | play-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/options-outline.svg` | options-outline | ionicons | third-party-legacy | wired | Referenced by production templates, Rust mappings, and tests. |
| `zircon_editor/assets/icons/ionicons/move-outline.svg` | move-outline | ionicons | third-party-legacy | wired | Referenced by production UI templates. |
| `zircon_editor/assets/icons/ionicons/menu-outline.svg` | menu-outline | ionicons | third-party-legacy | asset-only | No non-audit references found. |
| `zircon_editor/assets/icons/ionicons/locate-outline.svg` | locate-outline | ionicons | third-party-legacy | wired | Referenced by production UI templates. |
| `zircon_editor/assets/icons/ionicons/list-outline.svg` | list-outline | ionicons | third-party-legacy | wired | Referenced by production UI templates. |
| `zircon_editor/assets/icons/ionicons/layers-outline.svg` | layers-outline | ionicons | third-party-legacy | wired | Referenced by production templates and Rust mappings. |
| `zircon_editor/assets/icons/ionicons/grid-outline.svg` | grid-outline | ionicons | third-party-legacy | wired | Referenced by production templates and Rust mappings. |
| `zircon_editor/assets/icons/ionicons/game-controller-outline.svg` | game-controller-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/folder-outline.svg` | folder-outline | ionicons | third-party-legacy | wired | Referenced by production UI templates. |
| `zircon_editor/assets/icons/ionicons/folder-open-outline.svg` | folder-open-outline | ionicons | third-party-legacy | wired | Referenced by production templates, Rust mappings, and tests. |
| `zircon_editor/assets/icons/ionicons/ellipsis-horizontal-outline.svg` | ellipsis-horizontal-outline | ionicons | third-party-legacy | asset-only | No non-audit references found. |
| `zircon_editor/assets/icons/ionicons/ellipse-outline.svg` | ellipse-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/cube-outline.svg` | cube-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings and tests. |
| `zircon_editor/assets/icons/ionicons/construct-outline.svg` | construct-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/color-fill-outline.svg` | color-fill-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/close-outline.svg` | close-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings and tests. |
| `zircon_editor/assets/icons/ionicons/chevron-forward-outline.svg` | chevron-forward-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings. |
| `zircon_editor/assets/icons/ionicons/chevron-back-outline.svg` | chevron-back-outline | ionicons | third-party-legacy | wired | Referenced by production Rust mappings and tests. |
| `zircon_editor/assets/icons/ionicons/albums-outline.svg` | albums-outline | ionicons | third-party-legacy | wired | Referenced by production templates and Rust mappings. |
| `zircon_editor/assets/icons/ionicons/add-outline.svg` | add-outline | ionicons | third-party-legacy | wired | Referenced by production templates and Rust mappings. |

### editor-shell

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/menu.svg` | menu | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/file-new.svg` | file-new | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/folder-open.svg` | folder-open | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/save.svg` | save | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/undo.svg` | undo | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/redo.svg` | redo | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/select.svg` | select | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/move.svg` | move | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/rotate.svg` | rotate | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/scale.svg` | scale | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/gamepad.svg` | gamepad | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/package.svg` | package | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/node-graph.svg` | node-graph | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/dropdown.svg` | dropdown | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/play.svg` | play | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/layout-grid.svg` | layout-grid | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/sun.svg` | sun | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/toolbar/more-vertical.svg` | more-vertical | toolbar | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/play.svg` | play | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/cube.svg` | cube | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/node-graph.svg` | node-graph | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/image.svg` | image | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/audio.svg` | audio | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/code.svg` | code | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/settings.svg` | settings | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/activity/help.svg` | help | activity | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/root.svg` | root | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/light.svg` | light | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/sky.svg` | sky | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/geometry.svg` | geometry | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/props.svg` | props | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/player-start.svg` | player-start | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/audio-zone.svg` | audio-zone | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/eye.svg` | eye | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/lock.svg` | lock | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/scene/filter.svg` | filter | scene | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/inspector/transform.svg` | transform | inspector | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/inspector/mesh-renderer.svg` | mesh-renderer | inspector | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/inspector/link.svg` | link | inspector | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/inspector/check.svg` | check | inspector | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/inspector/material.svg` | material | inspector | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/add.svg` | add | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/delete.svg` | delete | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/checkbox.svg` | checkbox | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/radio.svg` | radio | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/list.svg` | list | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/controls/table.svg` | table | controls | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/status/info.svg` | info | status | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/status/success.svg` | success | status | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/status/warning.svg` | warning | status | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/status/error.svg` | error | status | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/viewport/axis-gizmo.svg` | axis-gizmo | viewport | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/viewport/grid.svg` | grid | viewport | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/viewport/magnet.svg` | magnet | viewport | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/viewport/globe.svg` | globe | viewport | zircon-owned | asset-only | Editor shell generated icon. |
| `zircon_editor/assets/icons/zircon_editor_shell/viewport/crosshair.svg` | crosshair | viewport | zircon-owned | asset-only | Editor shell generated icon. |

### editor-engine-style

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_editor/assets/icons/zircon_engine_style/assets/mesh.svg` | mesh | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/material.svg` | material | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/texture.svg` | texture | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/shader.svg` | shader | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/animation-clip.svg` | animation-clip | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/audio-clip.svg` | audio-clip | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/scene-file.svg` | scene-file | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/prefab.svg` | prefab | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/script-file.svg` | script-file | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/font.svg` | font | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/tilemap.svg` | tilemap | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/assets/particle-system.svg` | particle-system | assets | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/entity.svg` | entity | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/component.svg` | component | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/camera.svg` | camera | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/directional-light.svg` | directional-light | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/point-light.svg` | point-light | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/spot-light.svg` | spot-light | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/terrain.svg` | terrain | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/navmesh.svg` | navmesh | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/collider.svg` | collider | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/rigid-body.svg` | rigid-body | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/trigger-volume.svg` | trigger-volume | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/skeleton.svg` | skeleton | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/scene/bone.svg` | bone | scene | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/blueprint.svg` | blueprint | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/event-node.svg` | event-node | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/function-node.svg` | function-node | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/variable.svg` | variable | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/state-machine.svg` | state-machine | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/transition.svg` | transition | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/behavior-tree.svg` | behavior-tree | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/blackboard.svg` | blackboard | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/graph/shader-graph.svg` | shader-graph | graph | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/pipette.svg` | pipette | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/paint.svg` | paint | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/measure.svg` | measure | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/focus-selection.svg` | focus-selection | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/isolate.svg` | isolate | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/pivot.svg` | pivot | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/align.svg` | align | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/tools/snap-grid.svg` | snap-grid | tools | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/play-in-editor.svg` | play-in-editor | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/simulate.svg` | simulate | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/pause.svg` | pause | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/stop.svg` | stop | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/profiler.svg` | profiler | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/runtime/diagnostics.svg` | diagnostics | runtime | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/plugin.svg` | plugin | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/module.svg` | module | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/package.svg` | package | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/deploy.svg` | deploy | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/source-control.svg` | source-control | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/test.svg` | test | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |
| `zircon_editor/assets/icons/zircon_engine_style/build/cloud-build.svg` | cloud-build | build | zircon-owned | asset-only | Reference-informed engine-domain icon. |

### editor-preview

| Path | Name | Category | Owner | Status | Audit Notes |
| --- | --- | --- | --- | --- | --- |
| `zircon_editor/assets/preview/editor-scifi-room.svg` | editor-scifi-room | preview | zircon-owned | asset-only | Editor preview illustration. |

## Duplicate And Collision Review

| Name | Paths | Classification | Recommendation |
| --- | --- | --- | --- |
| settings | `zircon_hub/assets/icons/nav/settings.svg`; `zircon_hub/assets/icons/ui/settings.svg`; `zircon_editor/assets/icons/zircon_editor_shell/activity/settings.svg` | intentional contextual duplicate | Keep category-qualified references. |
| play | `zircon_editor/assets/icons/zircon_editor_shell/toolbar/play.svg`; `zircon_editor/assets/icons/zircon_editor_shell/activity/play.svg` | same pack multi-context duplicate | Keep until shell icon policy distinguishes toolbar/action/activity roles. |
| grid | `zircon_hub/assets/icons/ui/grid.svg`; `zircon_editor/assets/icons/zircon_editor_shell/viewport/grid.svg` | cross-product generic duplicate | Keep namespaced by pack. |
| material | `zircon_editor/assets/icons/zircon_editor_shell/inspector/material.svg`; `zircon_editor/assets/icons/zircon_engine_style/assets/material.svg` | editor-domain semantic overlap | Keep category-qualified references. |
| package | `zircon_editor/assets/icons/zircon_editor_shell/toolbar/package.svg`; `zircon_editor/assets/icons/zircon_engine_style/build/package.svg` | editor-domain semantic overlap | Keep category-qualified references. |
| folder | `zircon_hub/assets/icons/ui/folder.svg`; `zircon_editor/assets/icons/ionicons/folder-outline.svg`; `zircon_editor/assets/icons/ionicons/folder-open-outline.svg`; `zircon_editor/assets/icons/zircon_editor_shell/toolbar/folder-open.svg` | exact and stem-family overlap | Keep Ionicons legacy isolated and prefer explicit open/closed names in new packs. |
| save | `zircon_editor/assets/icons/ionicons/save-outline.svg`; `zircon_editor/assets/icons/zircon_editor_shell/toolbar/save.svg` | third-party versus generated replacement candidate | Do not replace until UI wiring policy is approved. |
| add | `zircon_editor/assets/icons/ionicons/add-outline.svg`; `zircon_editor/assets/icons/zircon_editor_shell/controls/add.svg` | third-party versus generated replacement candidate | Do not replace until UI wiring policy is approved. |
| list | `zircon_hub/assets/icons/ui/list.svg`; `zircon_editor/assets/icons/ionicons/list-outline.svg`; `zircon_editor/assets/icons/zircon_editor_shell/controls/list.svg` | generic UI duplicate | Keep namespaced by pack. |
| help | `zircon_hub/assets/icons/ui/help.svg`; `zircon_editor/assets/icons/zircon_editor_shell/activity/help.svg` | cross-product generic duplicate | Keep namespaced by pack. |
| status names | `error`, `warning`, and `success` exist in Hub status and editor shell status packs | intentional shared status vocabulary | Maintain visual consistency when either pack is wired. |

## Gap Review

Future families not fully represented yet: asset import state variants, scene editing mode variants, source-control state variants, detailed build pipeline status, plugin lifecycle state icons, profiler panel-specific icons, debug overlay icons, and expanded Hub cloud/team state icons.

## Backlog

| Priority | Title | Paths | Reason | Recommended Action |
| --- | --- | --- | --- | --- |
| P0 | Keep audit manifest synchronized | `docs/zircon_editor/assets/icon-resource-audit.md`; `docs/zircon_editor/assets/icon-resource-audit.json` | Future asset additions can make counts stale. | Update both files whenever audited SVG scopes change. |
| P0 | Preserve generated SVG integrity checks | `zircon_hub/assets`; `zircon_editor/assets/icons/zircon_editor_shell`; `zircon_editor/assets/icons/zircon_engine_style`; `zircon_editor/assets/preview` | External refs or non-ASCII content would weaken asset portability. | Rerun the Grep scans during future asset-generation tasks. |
| P1 | Define pack naming collision policy | `zircon_hub/assets/icons`; `zircon_editor/assets/icons` | Generic names such as `grid`, `help`, and `settings` are safe only with pack-qualified lookup. | Document whether consumers must address icons by pack/category/name. |
| P1 | Decide Ionicons replacement policy | `zircon_editor/assets/icons/ionicons`; `zircon_editor/assets/icons/zircon_editor_shell` | Existing Ionicons are wired while generated replacements are asset-only. | Approve per-view replacement criteria before changing UI references. |
| P1 | Add machine validation for pack counts | `docs/zircon_editor/assets/icon-resource-audit.json` | Manual counts are easy to drift. | Add a future non-build validation script if this audit becomes recurring. |
| P2 | Wire generated editor shell icons | `zircon_editor/assets/icons/zircon_editor_shell` | Generated shell assets are ready but unused. | Map retained-host icon roles to shell pack names in a later UI plan. |
| P2 | Expand future domain icon families | `zircon_editor/assets/icons/zircon_engine_style` | Current catalog omits detailed states for import, build, plugin, profiler, and debug overlays. | Generate targeted state icons after UI requirements are fixed. |
| P2 | Extend Hub cloud/team states | `zircon_hub/assets/icons/nav`; `zircon_hub/assets/icons/ui` | Hub has base cloud/team nav icons but not status variants. | Add state-specific icons when Hub feature screens require them. |

## Validation Evidence

| Check | Scope | Result | Evidence |
| --- | --- | --- | --- |
| Glob count | `zircon_hub/assets/**/*.svg` | pass | Fresh Glob returned 44 Hub SVG paths, matching pack total 44. |
| Glob count | `zircon_editor/assets/icons/**/*.svg` | pass | Fresh Glob output aligns with supplied pack total 142 for editor icons. |
| Glob count | `zircon_editor/assets/preview/**/*.svg` | pass | Fresh Glob returned 1 preview SVG path. |
| Pack totals | all packs | pass | Pack counts sum to 187 and match Hub 44 + editor icons 142 + preview 1. |
| External reference scan | Hub assets, Editor icons, Editor preview | pass | Grep pattern `href=|url\(http|https://|<image` returned `no matches` for all three audited roots. |
| Generated-pack non-ASCII scan | Hub assets, editor shell, engine-style, preview | pass | Grep pattern `[^\x00-\x7F]` returned `no matches` for each generated Zircon-owned scope. |
| JSON parser | `docs/zircon_editor/assets/icon-resource-audit.json` | pass | `python -m json.tool docs/zircon_editor/assets/icon-resource-audit.json` parsed successfully. |
