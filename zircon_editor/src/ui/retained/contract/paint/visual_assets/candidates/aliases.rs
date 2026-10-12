pub(super) fn shell_icon_alias(icon_name: &str) -> Option<&'static str> {
    match semantic_icon_key(icon_name).as_str() {
        "add" | "plus" => Some("zircon_editor_shell/controls/add.svg"),
        "add-circle" | "add-circle-outline" | "addcircle" => {
            Some("editor_pages/inspector/properties/add-component.svg")
        }
        "new" | "file" | "file-new" | "new-file" => {
            Some("zircon_editor_shell/toolbar/file-new.svg")
        }
        "folder" | "open" | "folder-open" => Some("zircon_editor_shell/toolbar/folder-open.svg"),
        "save" | "save-all" => Some("zircon_editor_shell/toolbar/save.svg"),
        "compile" | "build" => Some("zircon_editor_shell/toolbar/compile.svg"),
        "flash" | "flash-outline" => Some("zircon_editor_shell/toolbar/compile.svg"),
        "settings" | "gear" | "cog" => Some("zircon_editor_shell/activity/settings.svg"),
        "search" | "find" | "magnifier" | "magnifying-glass" | "magnifyingglass" => {
            Some("zircon_editor_shell/controls/search.svg")
        }
        "filter" => Some("zircon_editor_shell/scene/filter.svg"),
        "checkmark" | "check-mark" | "tick" => Some("zircon_editor_shell/controls/check.svg"),
        "trash" | "delete" | "remove" => Some("zircon_editor_shell/controls/delete.svg"),
        "copy" | "duplicate" => Some("zircon_editor_shell/controls/copy.svg"),
        "cancel" | "close" => Some("editor_pages/workbench/tabs/close-tab.svg"),
        "disabled" | "unavailable" => Some("zircon_editor_shell/status/disabled.svg"),
        "eye" | "visible" => Some("zircon_editor_shell/scene/eye.svg"),
        "eye-off" | "eyeoff" | "hidden" => Some("zircon_editor_shell/scene/eye-off.svg"),
        "lock" | "locked" => Some("zircon_editor_shell/scene/lock.svg"),
        "more" | "more-vertical" | "overflow" | "kebab" => {
            Some("zircon_editor_shell/toolbar/more-vertical.svg")
        }
        "more-horizontal" | "ellipsis" | "overflow-horizontal" => {
            Some("zircon_editor_shell/toolbar/more-horizontal.svg")
        }
        "dropdown" | "chevron-down" | "chevrondown" => {
            Some("zircon_editor_shell/toolbar/dropdown.svg")
        }
        "chevron-right" | "chevronright" | "disclosure" => {
            Some("zircon_editor_shell/toolbar/chevron-right.svg")
        }
        "snap" | "magnet" => Some("zircon_editor_shell/viewport/magnet.svg"),
        "navigate" | "navigate-outline" => Some("ionicons/locate-outline.svg"),
        "globe" | "world" => Some("zircon_editor_shell/viewport/globe.svg"),
        "target" | "crosshair" => Some("zircon_editor_shell/viewport/crosshair.svg"),
        "select" | "cursor" => Some("zircon_editor_shell/toolbar/select.svg"),
        "move" | "translate" => Some("zircon_editor_shell/toolbar/move.svg"),
        "rotate-ccw" | "reset" => Some("zircon_editor_shell/toolbar/undo.svg"),
        "rotate" => Some("zircon_editor_shell/toolbar/rotate.svg"),
        "scale" | "resize" => Some("zircon_editor_shell/toolbar/scale.svg"),
        "grid" | "layout" | "layout-grid" => Some("zircon_editor_shell/toolbar/layout-grid.svg"),
        "layers" => Some("ionicons/layers-outline.svg"),
        "checkbox" => Some("zircon_editor_shell/controls/checkbox.svg"),
        "radio" => Some("zircon_editor_shell/controls/radio.svg"),
        "table" | "columns" => Some("zircon_editor_shell/controls/table.svg"),
        "list" => Some("zircon_editor_shell/controls/list.svg"),
        "info" => Some("zircon_editor_shell/status/info.svg"),
        "warning" | "warn" | "alert" => Some("zircon_editor_shell/status/warning.svg"),
        "error" => Some("zircon_editor_shell/status/error.svg"),
        "success" | "check" | "ok" => Some("zircon_editor_shell/status/success.svg"),
        "play" | "scene" => Some("zircon_editor_shell/activity/play.svg"),
        "cube" | "entity" | "box" => Some("zircon_editor_shell/activity/cube.svg"),
        "component" => Some("zircon_engine_style/scene/component.svg"),
        "capsule" => Some("zircon_engine_style/scene/collider.svg"),
        "circle" => Some("ionicons/ellipse-outline.svg"),
        "mesh" | "mesh-renderer" => Some("zircon_editor_shell/inspector/mesh-renderer.svg"),
        "material" => Some("zircon_editor_shell/inspector/material.svg"),
        "root" => Some("zircon_editor_shell/scene/root.svg"),
        "environment" | "sky" => Some("zircon_editor_shell/scene/sky.svg"),
        "level" | "geometry" => Some("zircon_editor_shell/scene/geometry.svg"),
        "props" => Some("zircon_editor_shell/scene/props.svg"),
        "player-start" | "playerstart" => Some("zircon_editor_shell/scene/player-start.svg"),
        "audio-zone" | "audiozone" => Some("zircon_editor_shell/scene/audio-zone.svg"),
        "graph" | "node-graph" => Some("zircon_editor_shell/activity/node-graph.svg"),
        "image" => Some("zircon_editor_shell/activity/image.svg"),
        "audio" | "speaker" => Some("zircon_editor_shell/activity/audio.svg"),
        "code" => Some("zircon_editor_shell/activity/code.svg"),
        "camera" => Some("zircon_engine_style/scene/camera.svg"),
        "light" => Some("zircon_editor_shell/scene/light.svg"),
        "sun" => Some("zircon_editor_shell/toolbar/sun.svg"),
        "link" => Some("zircon_editor_shell/inspector/link.svg"),
        "edit" => Some("editor_pages/inspector/properties/override-property.svg"),
        "pin" | "pin-tab" => Some("editor_pages/workbench/tabs/pin-tab.svg"),
        "history" => Some("editor_pages/workbench/menu/undo-history.svg"),
        "activity" => Some("zircon_engine_style/runtime/profiler.svg"),
        "cpu" => Some("editor_pages/console_profiler/profiling/cpu.svg"),
        "chart" => Some("editor_pages/console_profiler/profiling/frame-graph.svg"),
        "timeline" => Some("editor_pages/animation_timeline/curves/dope-sheet.svg"),
        "monitor" | "renderer" => Some("editor_pages/inspector/sections/rendering.svg"),
        "package" => Some("zircon_editor_shell/toolbar/package.svg"),
        "puzzle" => Some("zircon_engine_style/build/plugin.svg"),
        "cloud" | "cloud-outline" => Some("editor_pages/build_plugins/deploy/cloud-upload.svg"),
        "sparkles" => Some("editor_pages/asset_browser/asset_types/particle-system.svg"),
        "route" => Some("zircon_engine_style/scene/navmesh.svg"),
        "database" => Some("zircon_engine_style/graph/blackboard.svg"),
        "button" => Some("editor_pages/ui_layout_editor/widgets/button.svg"),
        "tree" => Some("zircon_engine_style/graph/behavior-tree.svg"),
        "leaf" => Some("zircon_editor_shell/scene/props.svg"),
        "user" => Some("zircon_editor_shell/scene/player-start.svg"),
        "users" => Some("zircon_engine_style/scene/entity.svg"),
        "flag" => Some("editor_pages/workbench/status/notification.svg"),
        "asset-texture" => Some("editor_pages/asset_browser/asset_types/texture.svg"),
        "asset-material" => Some("editor_pages/asset_browser/asset_types/material.svg"),
        "asset-scene" => Some("editor_pages/asset_browser/asset_types/scene-file.svg"),
        "asset-shader" => Some("editor_pages/asset_browser/asset_types/shader.svg"),
        "asset-mesh" => Some("editor_pages/asset_browser/asset_types/mesh.svg"),
        "asset-script" => Some("editor_pages/asset_browser/asset_types/script-file.svg"),
        "asset-ui-layout" => Some("editor_pages/ui_layout_editor/layout/canvas.svg"),
        "asset-ui-widget" => Some("editor_pages/ui_layout_editor/widgets/widget.svg"),
        "asset-ui-style" => Some("editor_pages/inspector/sections/ui.svg"),
        "asset-audio" => Some("editor_pages/asset_browser/asset_types/audio-clip.svg"),
        "asset-font" => Some("editor_pages/asset_browser/asset_types/font.svg"),
        "asset-prefab" => Some("editor_pages/asset_browser/asset_types/prefab.svg"),
        "asset-animation-clip" => Some("editor_pages/asset_browser/asset_types/animation-clip.svg"),
        "asset-tilemap" => Some("zircon_engine_style/assets/tilemap.svg"),
        "terrain" => Some("zircon_engine_style/scene/terrain.svg"),
        "branch" => Some("editor_pages/build_plugins/source_control/branch.svg"),
        "prefab" => Some("editor_pages/asset_browser/asset_types/prefab.svg"),
        "trigger-volume" => Some("zircon_engine_style/scene/trigger-volume.svg"),
        "collider" => Some("zircon_engine_style/scene/collider.svg"),
        "physics" => Some("editor_pages/inspector/sections/physics.svg"),
        "navmesh" => Some("zircon_engine_style/scene/navmesh.svg"),
        "gamepad" => Some("zircon_editor_shell/toolbar/gamepad.svg"),
        "source-control" => Some("zircon_engine_style/build/source-control.svg"),
        "test" => Some("zircon_engine_style/build/test.svg"),
        "plugin" => Some("zircon_engine_style/build/plugin.svg"),
        "sequence" => Some("editor_pages/graph_editor/execution/sequence.svg"),
        "animation-clip" => Some("editor_pages/asset_browser/asset_types/animation-clip.svg"),
        "curve-bezier" => Some("editor_pages/animation_timeline/curves/curve-bezier.svg"),
        "skeleton" => Some("zircon_engine_style/scene/skeleton.svg"),
        "bone" => Some("zircon_engine_style/scene/bone.svg"),
        "constraint" => Some("editor_pages/ui_layout_editor/constraints/constraint.svg"),
        "animation" => Some("editor_pages/inspector/sections/animation.svg"),
        "archive" => Some("editor_pages/build_plugins/package/archive.svg"),
        "shader" => Some("editor_pages/asset_browser/asset_types/shader.svg"),
        "rendering" => Some("editor_pages/inspector/sections/rendering.svg"),
        "diagnostics" => Some("zircon_engine_style/runtime/diagnostics.svg"),
        "profiler" => Some("zircon_engine_style/runtime/profiler.svg"),
        "frame-time" => Some("editor_pages/console_profiler/profiling/frame-time.svg"),
        "font" => Some("editor_pages/asset_browser/asset_types/font.svg"),
        "ui" => Some("editor_pages/inspector/sections/ui.svg"),
        _ => None,
    }
}

fn semantic_icon_key(icon_name: &str) -> String {
    let normalized = icon_name
        .trim()
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_ascii_lowercase();
    let file_name = normalized.rsplit('/').next().unwrap_or(normalized.as_str());
    file_name
        .strip_suffix(".svg")
        .or_else(|| file_name.strip_suffix(".png"))
        .unwrap_or(file_name)
        .to_string()
}
