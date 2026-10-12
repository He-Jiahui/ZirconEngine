//! 原生手柄震动的容量边界与创建顺序测试。
//!
//! 父模块仅在 test 和 gamepad-gilrs 同时启用时挂载；测试调用纯准入函数或读取源码，不启动震动后端。

use super::*;

#[test]
/// 验证每个手柄允许保留的原生震动效果上限，以及达到上限时的稳定错误标识。
///
/// # 使用时机
/// 在 gamepad-gilrs 条件下调用 [admit_rumble_effect]，覆盖 [RUMBLE_EFFECTS_MAX_PER_GAMEPAD] 的最后可接受和第一个拒绝计数；不创建设备、句柄或计时器。
///
/// # 返回与状态
/// 测试返回 ()。容量减一返回 Ok(())，容量本身返回 runtime_gamepad_rumble_effect_limit_reached；准入检查不改变效果表。
///
/// # 参考
/// [admit_rumble_effect]、[RUMBLE_EFFECTS_MAX_PER_GAMEPAD]
fn rumble_effect_admission_has_a_fixed_per_gamepad_limit() {
    assert_eq!(RUMBLE_EFFECTS_MAX_PER_GAMEPAD, 32);
    assert_eq!(
        admit_rumble_effect(RUMBLE_EFFECTS_MAX_PER_GAMEPAD - 1),
        Ok(())
    );
    assert_eq!(
        admit_rumble_effect(RUMBLE_EFFECTS_MAX_PER_GAMEPAD),
        Err("runtime_gamepad_rumble_effect_limit_reached")
    );
}

#[test]
/// 通过源码守卫确认震动 Add 先完成容量准入，再创建、播放并发布运行中效果。
///
/// # 使用时机
/// 读取生产 rumble.rs 测试模块之前的文本并去空白，检查 [RuntimeEntryApp::apply_runtime_gamepad_rumble_request] 调用顺序和清理 marker。
///
/// # 返回与限制
/// 测试返回 ()；通过说明源码仍表达准入→finish→play→RunningRumbleEffect 入表，并保留 [clear_finished_rumble_effects] 与 [clear_gamepad_rumble_effects]。不验证平台返回值、设备副作用、时钟或并发。
///
/// # 参考
/// [RuntimeEntryApp::apply_runtime_gamepad_rumble_request]、[clear_finished_rumble_effects]、[clear_gamepad_rumble_effects]
fn rumble_add_admits_before_backend_creation_and_publish() {
    let source = include_str!("../rumble.rs")
        .split_once("\n#[cfg(all(test, feature = \"gamepad-gilrs\"))]")
        .map(|(production, _)| production)
        .expect("rumble production source precedes its test module");
    let source = source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect::<String>();

    let admission = source
        .find("admit_rumble_effect(active_effect_count)?;")
        .expect("rumble Add checks the per-gamepad hard limit");
    let finish = source
        .find(".finish(gamepads)")
        .expect("rumble Add creates a backend effect only after admission");
    let play = source
        .find("effect.play().map_err(rumble_force_feedback_error)?;")
        .expect("rumble Add plays an admitted effect");
    let publish = source
        .find(".push(RunningRumbleEffect{")
        .expect("rumble Add publishes the running effect after play succeeds");

    assert!(admission < finish && finish < play && play < publish);
    assert!(
        source.contains("ZrRuntimeGamepadRumbleRequestKindV1::Stop=>{stop_gamepad_rumble_effects(")
    );
    assert!(source.contains("effects.retain(|effect|effect.deadline>now);"));
    assert!(source.contains("for effect in effects.drain(..){"));
}
