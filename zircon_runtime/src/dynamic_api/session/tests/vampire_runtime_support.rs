use std::{env, path::Path};

use crate::core::framework::animation::AnimationParameterValue;
use crate::core::math::{Transform, Vec3};
use zircon_runtime_interface::{
    ZrByteSlice, ZrRuntimeEventV1, ZrRuntimeFrameRequestV1, ZrRuntimeViewportHandle,
    ZIRCON_RUNTIME_ABI_VERSION_V1, ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
    ZR_RUNTIME_BUTTON_STATE_RELEASED_V1, ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
};

use super::super::{RuntimeDynamicSession, RuntimeProjectConfig};

pub(super) fn vampire_project_config() -> RuntimeProjectConfig {
    let root = env::var_os("ZR_VAMPIRE_PROJECT_ROOT")
        .map(Into::into)
        .unwrap_or_else(|| {
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .parent()
                .expect("workspace root")
                .join("examples")
                .join("vampire")
        });
    let root = root.to_string_lossy();
    RuntimeProjectConfig::from_abi_slice(ZrByteSlice {
        data: root.as_ptr(),
        len: root.len(),
    })
    .unwrap()
    .unwrap()
}

pub(super) fn script_binding_number(
    session: &RuntimeDynamicSession,
    entity: u64,
    property: &str,
) -> Option<f64> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, "script.bindings")?
            .as_array()?
            .iter()
            .find_map(|binding| binding.get("properties")?.get(property)?.as_f64())
    })
}

pub(super) fn script_property_entities(
    session: &RuntimeDynamicSession,
    property: &str,
    expected: &str,
) -> Vec<u64> {
    session.level.with_world(|world| {
        world
            .node_records()
            .into_iter()
            .filter(|node| {
                world
                    .dynamic_component(node.id, "script.bindings")
                    .is_some_and(|bindings| {
                        script_binding_property_matches(bindings, property, expected)
                    })
            })
            .map(|node| node.id)
            .collect()
    })
}

fn script_binding_property_matches(
    bindings: &serde_json::Value,
    property: &str,
    expected: &str,
) -> bool {
    bindings.as_array().is_some_and(|bindings| {
        bindings.iter().any(|binding| {
            binding
                .get("enabled")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(true)
                && binding
                    .get("properties")
                    .and_then(|properties| properties.get(property))
                    .and_then(serde_json::Value::as_str)
                    == Some(expected)
        })
    })
}

pub(super) fn set_entity_position(session: &RuntimeDynamicSession, entity: u64, position: Vec3) {
    session.level.with_world_mut(|world| {
        let mut transform = world.world_transform(entity).unwrap_or_default();
        transform.translation = position;
        world
            .update_transform(entity, transform)
            .expect("test entity transform should be mutable");
    });
}

pub(super) fn set_script_binding_number(
    session: &RuntimeDynamicSession,
    entity: u64,
    property: &str,
    value: f64,
) {
    session.level.with_world_mut(|world| {
        let mut bindings = world
            .dynamic_component(entity, "script.bindings")
            .cloned()
            .unwrap_or_else(empty_vampire_script_bindings);
        let binding = bindings
            .as_array_mut()
            .and_then(|bindings| bindings.first_mut())
            .expect("test script bindings should contain one binding");
        let properties = binding
            .get_mut("properties")
            .and_then(serde_json::Value::as_object_mut)
            .expect("test script binding should contain properties");
        properties.insert(property.to_string(), serde_json::json!(value));
        world
            .set_dynamic_component(entity, "script.bindings", bindings)
            .expect("test script bindings should be writable");
    });
}

pub(super) fn remove_script_entities_by_role_except(
    session: &RuntimeDynamicSession,
    role: &str,
    keep: Option<u64>,
) {
    let entities = script_property_entities(session, "role", role);
    session.level.with_world_mut(|world| {
        for entity in entities {
            if Some(entity) != keep {
                world.remove_entity(entity);
            }
        }
    });
}

fn empty_vampire_script_bindings() -> serde_json::Value {
    serde_json::json!([{
        "package": "vampire_game",
        "module": "main",
        "enabled": true,
        "properties": {}
    }])
}

pub(super) fn dynamic_component_i64(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<i64> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, component_id)
            .and_then(serde_json::Value::as_i64)
    })
}

pub(super) fn dynamic_component_value(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<serde_json::Value> {
    session
        .level
        .with_world(|world| world.dynamic_component(entity, component_id).cloned())
}

pub(super) fn dynamic_component_string(
    session: &RuntimeDynamicSession,
    entity: u64,
    component_id: &str,
) -> Option<String> {
    session.level.with_world(|world| {
        world
            .dynamic_component(entity, component_id)
            .and_then(serde_json::Value::as_str)
            .map(str::to_string)
    })
}

pub(super) fn start_vampire_game(session: &mut RuntimeDynamicSession) {
    session.tick_frame().unwrap();
    click_vampire_menu_button(session, 640, 360);
    session.tick_frame().unwrap();
    session.tick_frame().unwrap();
    assert_eq!(
        dynamic_component_string(session, 2, "vampire.run_state").as_deref(),
        Some("playing"),
        "vampire test helper should enter gameplay before assertions"
    );
}

pub(super) fn click_vampire_menu_button(
    session: &mut RuntimeDynamicSession,
    width: u32,
    height: u32,
) {
    let resized = session.handle_event(ZrRuntimeEventV1::viewport_resized(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        zircon_runtime_interface::ZrRuntimeViewportSizeV1::new(width.max(1), height.max(1)),
    ));
    assert!(resized.is_ok(), "{resized:?}");

    let pointer_x = width as f32 * 0.5;
    let pointer_y = height as f32 * 0.5 + 68.0;
    let moved = session.handle_event(ZrRuntimeEventV1::pointer_moved(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        pointer_x,
        pointer_y,
    ));
    assert!(moved.is_ok(), "{moved:?}");
    let pressed = session.handle_event(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        ZR_RUNTIME_BUTTON_STATE_PRESSED_V1,
        pointer_x,
        pointer_y,
    ));
    assert!(pressed.is_ok(), "{pressed:?}");
    let released = session.handle_event(ZrRuntimeEventV1::mouse_button(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        ZR_RUNTIME_MOUSE_BUTTON_LEFT_V1,
        ZR_RUNTIME_BUTTON_STATE_RELEASED_V1,
        pointer_x,
        pointer_y,
    ));
    assert!(released.is_ok(), "{released:?}");
}

pub(super) fn world_hud_bar(
    session: &RuntimeDynamicSession,
    entity: u64,
) -> Option<serde_json::Value> {
    dynamic_component_value(session, entity, "render.world_hud_bars")
        .and_then(|value| {
            value
                .get("bars")
                .and_then(serde_json::Value::as_array)
                .cloned()
        })
        .and_then(|bars| bars.first().cloned())
}

pub(super) fn assert_world_hud_bar_tracks_position(
    bar: &serde_json::Value,
    expected_position: Vec3,
    message: &str,
) {
    let position = bar
        .get("position")
        .and_then(serde_json::Value::as_array)
        .expect("world HUD bar must carry a position");
    let actual = Vec3::new(
        position
            .first()
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
        position
            .get(1)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
        position
            .get(2)
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::NAN) as f32,
    );
    let delta = actual - expected_position;
    assert!(
        delta.length() <= 0.001,
        "{message}: actual={actual:?} expected={expected_position:?} bar={bar:?}"
    );
    assert!(
        bar.get("ratio")
            .and_then(serde_json::Value::as_f64)
            .is_some_and(|ratio| (0.0..=1.0).contains(&ratio)),
        "{message}: world HUD bar must carry a normalized ratio, bar={bar:?}"
    );
}

pub(super) fn animation_state_machine_parameters(
    session: &RuntimeDynamicSession,
    entity: u64,
) -> std::collections::BTreeMap<String, AnimationParameterValue> {
    session
        .level
        .with_world(|world| {
            world
                .animation_state_machine_player(entity)
                .map(|player| player.parameters.clone())
        })
        .unwrap_or_default()
}

pub(super) fn vampire_actor_node_local_transforms(
    session: &RuntimeDynamicSession,
) -> std::collections::BTreeMap<u64, Transform> {
    session.level.with_world(|world| {
        [202, 203, 204, 205, 206, 207]
            .into_iter()
            .filter_map(|entity| world.find_node(entity).map(|node| (entity, node.transform)))
            .collect()
    })
}

pub(super) fn entity_position(session: &RuntimeDynamicSession, entity: u64) -> Vec3 {
    session
        .level
        .with_world(|world| world.world_transform(entity).unwrap().translation)
}

pub(super) fn assert_vec3_close(actual: Vec3, expected: Vec3) {
    let delta = actual - expected;
    assert!(
        delta.length() <= 0.001,
        "expected vector close to {expected:?}, got {actual:?}"
    );
}

pub(super) fn planar_distance(a: Vec3, b: Vec3) -> f32 {
    let delta = a - b;
    Vec3::new(delta.x, 0.0, delta.z).length()
}

pub(super) fn count_hud_panel_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let y_start = 16usize.min(height);
    let y_end = 80usize.min(height);
    let x_start = 16usize.min(width);
    let x_end = 260usize.min(width);
    let mut count = 0;
    for y in y_start..y_end {
        for x in x_start..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            if pixel[0] <= 70 && pixel[1] <= 90 && pixel[2] <= 105 && pixel[3] >= 180 {
                count += 1;
            }
        }
    }
    count
}

pub(super) fn count_world_hud_bar_pixels(rgba: &[u8], width: u32, height: u32) -> usize {
    let width = width as usize;
    let height = height as usize;
    let mut count = 0;
    for y in 0..height {
        for x in 0..width {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            let red_bar =
                pixel[0] >= 170 && (90..=170).contains(&pixel[1]) && (95..=190).contains(&pixel[2]);
            let green_bar = (110..=200).contains(&pixel[0]) && pixel[1] >= 150 && pixel[2] <= 140;
            let blue_bar = pixel[0] <= 120 && pixel[1] >= 120 && pixel[2] >= 160;
            let purple_slot = pixel[0] >= 120 && pixel[1] <= 150 && pixel[2] >= 130;
            if pixel[3] >= 180 && (red_bar || green_bar || blue_bar || purple_slot) {
                count += 1;
            }
        }
    }
    count
}

pub(super) fn capture_vampire_frame_for_env(session: &mut RuntimeDynamicSession, env_var: &str) {
    if std::env::var(env_var).is_err() {
        return;
    }
    let frame = session
        .capture_frame(ZrRuntimeFrameRequestV1::new(
            ZIRCON_RUNTIME_ABI_VERSION_V1,
            ZrRuntimeViewportHandle::new(1),
            vampire_capture_viewport_size(),
        ))
        .unwrap();
    let rgba = if frame.rgba.data.is_null() || frame.rgba.len == 0 {
        &[]
    } else {
        unsafe { std::slice::from_raw_parts(frame.rgba.data.cast_const(), frame.rgba.len) }
    };
    export_vampire_capture_frame_if_requested(env_var, rgba, frame.width, frame.height);
}

pub(super) fn export_vampire_capture_frame_if_requested(
    env_var: &str,
    rgba: &[u8],
    width: u32,
    height: u32,
) {
    let Ok(path) = std::env::var(env_var) else {
        return;
    };
    let Some(image) = image::RgbaImage::from_raw(width, height, rgba.to_vec()) else {
        panic!("captured vampire frame rgba buffer does not match {width}x{height}");
    };
    image
        .save_with_format(path, image::ImageFormat::Png)
        .expect("failed to export vampire capture frame png");
}

pub(super) fn vampire_capture_viewport_size() -> zircon_runtime_interface::ZrRuntimeViewportSizeV1 {
    let width = std::env::var("ZR_VAMPIRE_CAPTURE_WIDTH")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(640);
    let height = std::env::var("ZR_VAMPIRE_CAPTURE_HEIGHT")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .unwrap_or(360);
    zircon_runtime_interface::ZrRuntimeViewportSizeV1::new(width.max(1), height.max(1))
}

pub(super) fn vampire_capture_tick_count() -> usize {
    std::env::var("ZR_VAMPIRE_CAPTURE_TICKS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
}

pub(super) fn vampire_diagnostic_tick_count() -> usize {
    std::env::var("ZR_VAMPIRE_DIAGNOSTIC_TICKS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(12)
}

pub(super) fn diagnostic_current(
    diagnostics: &crate::core::diagnostics::RuntimeDiagnosticsSnapshot,
    path: &str,
) -> Option<f64> {
    diagnostics
        .store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
        .and_then(|series| series.current)
}

pub(super) fn diagnostic_series<'a>(
    diagnostics: &'a crate::core::diagnostics::RuntimeDiagnosticsSnapshot,
    path: &str,
) -> Option<&'a crate::core::diagnostics::DiagnosticSeriesSnapshot> {
    diagnostics
        .store
        .series
        .iter()
        .find(|series| series.path.as_str() == path)
}

pub(super) fn small_headless_frame_request() -> ZrRuntimeFrameRequestV1 {
    ZrRuntimeFrameRequestV1::new(
        ZIRCON_RUNTIME_ABI_VERSION_V1,
        ZrRuntimeViewportHandle::new(1),
        zircon_runtime_interface::ZrRuntimeViewportSizeV1::new(64, 48),
    )
}

pub(super) fn summarize_hud_region(rgba: &[u8], width: u32, height: u32) -> String {
    let width = width as usize;
    let height = height as usize;
    let sample_points = [
        (20, 20),
        (80, 24),
        (220, 50),
        (20, height.saturating_sub(24)),
    ];
    let samples = sample_points
        .into_iter()
        .filter_map(|(x, y)| {
            let index = (y * width + x) * 4;
            rgba.get(index..index + 4)
                .map(|pixel| format!("({x},{y})={:?}", pixel))
        })
        .collect::<Vec<_>>();
    let mut min_rgb = [u8::MAX; 3];
    let mut max_rgb = [u8::MIN; 3];
    let mut opaque = 0usize;
    let y_end = 80usize.min(height);
    let x_end = 260usize.min(width);
    for y in 16usize.min(height)..y_end {
        for x in 16usize.min(width)..x_end {
            let index = (y * width + x) * 4;
            let Some(pixel) = rgba.get(index..index + 4) else {
                continue;
            };
            for channel in 0..3 {
                min_rgb[channel] = min_rgb[channel].min(pixel[channel]);
                max_rgb[channel] = max_rgb[channel].max(pixel[channel]);
            }
            if pixel[3] >= 180 {
                opaque += 1;
            }
        }
    }
    format!(
        "samples=[{}] min_rgb={min_rgb:?} max_rgb={max_rgb:?} opaque={opaque}",
        samples.join(", ")
    )
}
