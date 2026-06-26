pub(super) fn shell_icon_alias(icon_name: &str) -> Option<&'static str> {
    match semantic_icon_key(icon_name).as_str() {
        "add" | "plus" => Some("zircon_editor_shell/controls/add.svg"),
        "new" | "file" | "file-new" | "new-file" => {
            Some("zircon_editor_shell/toolbar/file-new.svg")
        }
        "folder" | "open" | "folder-open" => Some("zircon_editor_shell/toolbar/folder-open.svg"),
        "save" | "save-all" => Some("zircon_editor_shell/toolbar/save.svg"),
        "compile" | "build" => Some("zircon_editor_shell/toolbar/compile.svg"),
        "settings" | "gear" | "cog" => Some("zircon_editor_shell/activity/settings.svg"),
        "filter" => Some("zircon_editor_shell/scene/filter.svg"),
        "checkmark" | "check-mark" | "tick" => Some("zircon_editor_shell/controls/check.svg"),
        "trash" | "delete" | "remove" => Some("zircon_editor_shell/controls/delete.svg"),
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
        "globe" | "world" => Some("zircon_editor_shell/viewport/globe.svg"),
        "target" | "crosshair" => Some("zircon_editor_shell/viewport/crosshair.svg"),
        "select" | "cursor" => Some("zircon_editor_shell/toolbar/select.svg"),
        "move" | "translate" => Some("zircon_editor_shell/toolbar/move.svg"),
        "rotate" => Some("zircon_editor_shell/toolbar/rotate.svg"),
        "scale" | "resize" => Some("zircon_editor_shell/toolbar/scale.svg"),
        "grid" | "layout" | "layout-grid" => Some("zircon_editor_shell/toolbar/layout-grid.svg"),
        "checkbox" => Some("zircon_editor_shell/controls/checkbox.svg"),
        "radio" => Some("zircon_editor_shell/controls/radio.svg"),
        "table" | "columns" => Some("zircon_editor_shell/controls/table.svg"),
        "list" => Some("zircon_editor_shell/controls/list.svg"),
        "info" => Some("zircon_editor_shell/status/info.svg"),
        "warning" | "warn" => Some("zircon_editor_shell/status/warning.svg"),
        "error" => Some("zircon_editor_shell/status/error.svg"),
        "success" | "check" | "ok" => Some("zircon_editor_shell/status/success.svg"),
        "play" | "scene" => Some("zircon_editor_shell/activity/play.svg"),
        "cube" | "entity" => Some("zircon_editor_shell/activity/cube.svg"),
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
        "audio" => Some("zircon_editor_shell/activity/audio.svg"),
        "code" => Some("zircon_editor_shell/activity/code.svg"),
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
