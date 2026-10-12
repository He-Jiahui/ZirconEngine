//! 验证默认输入管理器的帧事件保留、相邻指针合并和独立原始记录周期。
//!
//! # 使用时机
//! 由父测试模块挂载，测试框架调用其中的同步用例；每个用例构造自己的管理器，
//! 帧窗口、快照与排空结果均限于该用例，不依赖平台事件循环。
//!
//! # 状态与生命周期
//! 帧事件队列可以合并相邻指针样本，原始记录队列则按独立的启用状态和容量保留事件。
//! 本模块分别检查两条路径的边界，样本数循环不测量墙钟时间或真实轮询频率。
//!
//! # 参考
//! [crate::input::DefaultInputManager]、[crate::core::framework::input::InputManager]、
//! [crate::input::InputEventRecordingConfig]

use super::*;

// 指针高频事件只在相邻同类项合并；测试确认按钮、触摸和失焦仍是有序边界。
/// 以三档样本数标签循环建立帧窗口，检查光标位置取相邻样本末值、相对移动累加及队列合并计数。
///
/// # 执行边界
/// 每档循环 600 次；events_per_second 只决定每个窗口提交的样本数，代码不等待或测量墙钟时间，因此不构成时延或吞吐基准。
///
/// # 调用与生命周期
/// 管理器仅存活于本用例；每个窗口先断言队列状态，再排空并核对保留事件。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 125、500、1000 是每个窗口的样本数；每档执行 600 次循环，每次分别开启绝对位置与相对移动两个窗口。断言检查合并后事件和计数，不能据此换算真实轮询频率。
///
/// # 参考
/// [crate::core::framework::input::InputManager::begin_frame]、[crate::core::framework::input::InputManager::submit_event]、[crate::core::framework::input::InputManager::event_queue_status]、[crate::core::framework::input::InputManager::drain_events]
#[test]
fn pointer_event_streams_are_frame_bounded_at_common_polling_rates() {
    for events_per_second in [125, 500, 1_000] {
        let input = DefaultInputManager::default();
        for _second in 0..600 {
            input.begin_frame();
            for sample in 0..events_per_second {
                input.submit_event(InputEvent::CursorMoved {
                    x: sample as f32,
                    y: -(sample as f32),
                });
            }
            assert_eq!(input.event_queue_status().retained_events, 1);
            assert_eq!(
                input.event_queue_status().coalesced_events,
                (events_per_second - 1) as u64
            );
            assert_eq!(
                input.drain_events(),
                vec![InputEvent::CursorMoved {
                    x: (events_per_second - 1) as f32,
                    y: -((events_per_second - 1) as f32),
                }]
            );

            input.begin_frame();
            for _ in 0..events_per_second {
                input.submit_event(InputEvent::MouseMotion {
                    delta_x: 0.25,
                    delta_y: -0.5,
                });
            }
            assert_eq!(input.event_queue_status().retained_events, 1);
            assert_eq!(
                input.event_queue_status().coalesced_events,
                (events_per_second - 1) as u64
            );
            assert_eq!(
                input.drain_events(),
                vec![InputEvent::MouseMotion {
                    delta_x: events_per_second as f32 * 0.25,
                    delta_y: events_per_second as f32 * -0.5,
                }]
            );
        }
    }
}

/// 验证同一帧中的按钮边沿阻断光标合并，且触摸开始、移动和结束事件仍按提交顺序排空。
///
/// # 调用与生命周期
/// 管理器在本用例内创建，完整事件序列只排空一次。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 断言比较一次排空得到的完整事件序列：按钮边沿分隔两段光标合并，触摸阶段维持提交顺序。它没有读取触摸快照或执行宿主操作。
///
/// # 参考
/// [crate::core::framework::input::InputManager::submit_event]、[crate::core::framework::input::InputManager::drain_events]
#[test]
fn pointer_coalescing_preserves_button_and_touch_edges_in_order() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 4.0 });
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
    input.submit_event(InputEvent::CursorMoved { x: 5.0, y: 6.0 });
    input.submit_event(InputEvent::CursorMoved { x: 7.0, y: 8.0 });
    input.submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Started,
        x: 10.0,
        y: 11.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Moved,
        x: 12.0,
        y: 13.0,
    });
    input.submit_event(InputEvent::Touch {
        id: 9,
        phase: TouchPhase::Ended,
        x: 14.0,
        y: 15.0,
    });

    assert_eq!(
        input.drain_events(),
        vec![
            InputEvent::CursorMoved { x: 3.0, y: 4.0 },
            InputEvent::ButtonPressed(InputButton::MouseLeft),
            InputEvent::CursorMoved { x: 7.0, y: 8.0 },
            InputEvent::ButtonReleased(InputButton::MouseLeft),
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Started,
                x: 10.0,
                y: 11.0,
            },
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Moved,
                x: 12.0,
                y: 13.0,
            },
            InputEvent::Touch {
                id: 9,
                phase: TouchPhase::Ended,
                x: 14.0,
                y: 15.0,
            },
        ]
    );
}

/// 验证 FocusLost 将光标移动样本分成两个相邻合并区段，并保留失焦事件的队列顺序。
///
/// # 调用与生命周期
/// 所有输入属于一个新建管理器的一帧。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 断言仅检查失焦前后两段光标事件及中间的失焦事件；按键、设备、IME 和光标宿主的失焦清理不由本用例断言。
///
/// # 参考
/// [crate::core::framework::input::InputManager::submit_event]、[crate::core::framework::input::InputManager::drain_events]
#[test]
fn focus_loss_is_a_barrier_to_pointer_event_coalescing() {
    let input = DefaultInputManager::default();
    input.begin_frame();
    input.submit_event(InputEvent::CursorMoved { x: 1.0, y: 2.0 });
    input.submit_event(InputEvent::CursorMoved { x: 3.0, y: 4.0 });
    input.submit_event(InputEvent::FocusLost);
    input.submit_event(InputEvent::CursorMoved { x: 5.0, y: 6.0 });
    input.submit_event(InputEvent::CursorMoved { x: 7.0, y: 8.0 });

    assert_eq!(
        input.drain_events(),
        vec![
            InputEvent::CursorMoved { x: 3.0, y: 4.0 },
            InputEvent::FocusLost,
            InputEvent::CursorMoved { x: 7.0, y: 8.0 },
        ]
    );
}

/// 验证帧边界丢弃尚未排空的瞬时按钮事件，同时保留已更新的按住状态。
///
/// # 调用与生命周期
/// 本用例通过 DefaultInputManager 的 begin_frame 覆盖实现跨越一次帧边界。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 事件在初始窗口提交后只跨越一次帧边界；断言检查瞬时队列已空和按钮仍按住，不读取该帧的 just_pressed，也不检查原始记录队列。
///
/// # 参考
/// [crate::input::DefaultInputManager]、[crate::core::framework::input::InputManager::begin_frame]、[crate::core::framework::input::InputManager::drain_events]、[crate::core::framework::input::InputManager::frame_snapshot]
#[test]
fn begin_frame_discards_undrained_transient_events() {
    let input = DefaultInputManager::default();
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));

    input.begin_frame();

    assert!(input.drain_events().is_empty());
    assert!(input
        .frame_snapshot()
        .buttons
        .pressed(&InputButton::MouseLeft));
}

/// 验证原始记录默认关闭、启用后按容量保留最新事件并报告丢弃数，以及排空和禁用时的计数生命周期。
///
/// # 调用与生命周期
/// 记录周期依次经历默认关闭、容量 3 启用、排空和重新禁用。
///
/// # 参数
/// 无显式参数；测试框架调用此入口，夹具与提交序列由函数内部建立。
///
/// # 返回值
/// 正常返回 `()` 表示本用例的断言均通过；断言或 unwrap 失败会 panic，由测试框架报告失败，不返回业务数据。
///
/// # 验证范围
/// 启用容量 3 后提交五个事件，原始记录保留序号 3、4、5 并累计淘汰 2 个；独立帧队列保留合并后的末值。排空保留淘汰数，重新禁用重置记录计数，不把两条队列视为等价。
///
/// # 参考
/// [crate::core::framework::input::InputManager::set_event_recording_config]、[crate::core::framework::input::InputManager::event_recording_status]、[crate::core::framework::input::InputManager::drain_event_records]、[crate::core::framework::input::InputManager::drain_events]
#[test]
fn recording_is_opt_in_bounded_and_reports_discarded_raw_records() {
    let input = DefaultInputManager::default();
    for sample in 0..1_000 {
        input.submit_event(InputEvent::CursorMoved {
            x: sample as f32,
            y: 0.0,
        });
    }

    assert!(input.drain_event_records().is_empty());
    assert_eq!(input.event_recording_status().retained_records, 0);
    assert!(!input.event_recording_status().enabled);

    input.set_event_recording_config(crate::input::InputEventRecordingConfig::enabled(3));
    for sample in 0..5 {
        input.submit_event(InputEvent::CursorMoved {
            x: sample as f32,
            y: 1.0,
        });
    }

    let status = input.event_recording_status();
    assert!(status.enabled);
    assert_eq!(status.capacity, 3);
    assert_eq!(status.retained_records, 3);
    assert_eq!(status.discarded_records, 2);
    assert_eq!(
        input.drain_events(),
        vec![InputEvent::CursorMoved { x: 4.0, y: 1.0 }]
    );

    let records = input.drain_event_records();
    assert_eq!(
        records
            .iter()
            .map(|record| record.sequence)
            .collect::<Vec<_>>(),
        vec![3, 4, 5]
    );
    assert_eq!(
        records
            .iter()
            .map(|record| record.event.clone())
            .collect::<Vec<_>>(),
        vec![
            InputEvent::CursorMoved { x: 2.0, y: 1.0 },
            InputEvent::CursorMoved { x: 3.0, y: 1.0 },
            InputEvent::CursorMoved { x: 4.0, y: 1.0 },
        ]
    );
    let drained_status = input.event_recording_status();
    assert_eq!(drained_status.retained_records, 0);
    assert_eq!(drained_status.discarded_records, 2);

    input.set_event_recording_config(crate::input::InputEventRecordingConfig::disabled());
    input.submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
    let disabled_status = input.event_recording_status();
    assert!(!disabled_status.enabled);
    assert_eq!(disabled_status.retained_records, 0);
    assert_eq!(disabled_status.discarded_records, 0);
    assert!(input.drain_event_records().is_empty());
}
