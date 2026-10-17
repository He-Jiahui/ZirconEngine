//! 从键盘命令构造拥有所有权的有序编辑意图。
//! 这里只计算源字节偏移与动作次序；文本状态机负责实际应用、边界归一和只读约束。

use zircon_runtime_interface::ui::{
    dispatch::{UiKeyboardInputEvent, UiKeyboardInputState},
    surface::{UiEditableTextState, UiTextEditAction},
};

use crate::ui::text::{
    line_end_boundary, line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary,
};

/// 键盘编辑产生的一至两个有序动作，无需为每个按键分配容器。
/// 整词/整行删除先选择范围再删除，消费方须保持顺序并作为一条编辑意图提交。
///
/// # 所有权与生命周期
/// 首项始终存在，次项可选；动作不借用输入状态，交给调用方一次消费。
/// 构造并未修改文本；是否提交内容变化仍由后续状态机与文档事务决定。
pub(in crate::ui::surface::input) struct KeyboardTextEditActions {
    first: UiTextEditAction,
    second: Option<UiTextEditAction>,
}

impl IntoIterator for KeyboardTextEditActions {
    type Item = UiTextEditAction;
    type IntoIter = std::iter::Chain<
        std::iter::Once<UiTextEditAction>,
        std::option::IntoIter<UiTextEditAction>,
    >;

    /// 移动动作所有权并保持首项在前、可选次项在后，不克隆或执行动作。
    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self.first).chain(self.second)
    }
}

/// 把命令键转换为使用源字节边界的文字编辑意图；字符输入、历史和剪贴板已由调用方优先处理。
/// 安全文本绕开词边界导航，避免掩码交互暴露真实词结构。
///
/// # 参数
/// 借用本次事件与编辑快照；安全类别由调用方读取节点，偏移使用源文本的 UTF-8 字节单位。
/// # 返回与调用时机
/// 按下或重复态返回本层识别的动作序列；空值让路由继续尝试其他键盘行为。
/// 调用方已优先处理历史、剪贴板及文字负载，并须按序将整条序列一次提交。
/// # 约束与参考
/// 逻辑键未匹配后才进入物理码回退；同级可编辑路由与文本状态机负责落地及错误处理。
pub(in crate::ui::surface::input) fn keyboard_text_edit_actions(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
    secure: bool,
) -> Option<KeyboardTextEditActions> {
    if !matches!(
        keyboard.state,
        UiKeyboardInputState::Pressed | UiKeyboardInputState::Repeated
    ) {
        return None;
    }

    let extend_selection = keyboard.metadata.modifiers.shift;
    let secure_line_navigation = secure && keyboard.metadata.modifiers.control;
    let secure_line_deletion = secure_line_navigation
        && !keyboard.metadata.modifiers.alt
        && !keyboard.metadata.modifiers.shift;
    let word_navigation = !secure && keyboard.metadata.modifiers.control;
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = secure_line_navigation
        || (keyboard.metadata.modifiers.super_key
            && !keyboard.metadata.modifiers.control
            && !keyboard.metadata.modifiers.alt);
    match keyboard.logical_key.as_str() {
        // 空逻辑键交给物理键回退；全选必须由逻辑 A 或回退中的物理 A 明确识别。
        key if keyboard_requests_select_all(keyboard, key) => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        "Backspace" if secure_line_deletion => Some(delete_to_line_start_actions(state)),
        "Delete" if secure_line_deletion => Some(delete_to_line_end_actions(state)),
        "Backspace" if word_navigation => Some(delete_previous_word_actions(state)),
        "Delete" if word_navigation => Some(delete_next_word_actions(state)),
        "Backspace" => Some(single_action(UiTextEditAction::Backspace)),
        "Delete" => Some(single_action(UiTextEditAction::Delete)),
        "Escape" => Some(escape_actions(state)),
        "ArrowLeft" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowRight" if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        "ArrowLeft" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowRight" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        "ArrowUp" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        "ArrowDown" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        "Home" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        "End" => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => keyboard_text_edit_actions_from_key_code(
            keyboard,
            state,
            extend_selection,
            word_navigation,
            secure_line_navigation,
            secure_line_deletion,
        ),
    }
}

/// 复用上层已算出的选择与安全策略，仅在逻辑映射未命中后解释物理码。
/// 未知物理码返回空值；此处仍只构造意图，不改变传入快照。
fn keyboard_text_edit_actions_from_key_code(
    keyboard: &UiKeyboardInputEvent,
    state: &UiEditableTextState,
    extend_selection: bool,
    word_navigation: bool,
    secure_line_navigation: bool,
    secure_line_deletion: bool,
) -> Option<KeyboardTextEditActions> {
    let document_navigation =
        keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key;
    let hard_line_navigation = secure_line_navigation
        || (keyboard.metadata.modifiers.super_key
            && !keyboard.metadata.modifiers.control
            && !keyboard.metadata.modifiers.alt);
    match keyboard.key_code {
        65 | 97 if keyboard_requests_select_all(keyboard, "a") => {
            Some(single_action(UiTextEditAction::SetSelection {
                anchor: 0,
                focus: state.text.len(),
            }))
        }
        8 if secure_line_deletion => Some(delete_to_line_start_actions(state)),
        46 if secure_line_deletion => Some(delete_to_line_end_actions(state)),
        8 if word_navigation => Some(delete_previous_word_actions(state)),
        46 if word_navigation => Some(delete_next_word_actions(state)),
        8 => Some(single_action(UiTextEditAction::Backspace)),
        46 => Some(single_action(UiTextEditAction::Delete)),
        27 => Some(escape_actions(state)),
        37 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_start_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        39 if hard_line_navigation => Some(single_action(UiTextEditAction::MoveCaret {
            offset: line_end_boundary(&state.text, state.caret.offset),
            extend_selection,
        })),
        37 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        39 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_text_boundary(&state.text, state.caret.offset, word_navigation),
            extend_selection,
        })),
        38 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: previous_line_offset(state, document_navigation),
            extend_selection,
        })),
        40 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: next_line_offset(state, document_navigation),
            extend_selection,
        })),
        36 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: home_offset(state, document_navigation),
            extend_selection,
        })),
        35 => Some(single_action(UiTextEditAction::MoveCaret {
            offset: end_offset(state, document_navigation),
            extend_selection,
        })),
        _ => None,
    }
}

// 已有非空选区时直接删除；否则先选择目标范围，再让删除动作消费该选区。
fn delete_previous_word_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Backspace);
    }
    let caret = state.caret.offset.min(state.text.len());
    let start = previous_text_boundary(&state.text, caret, true);
    if start == caret {
        single_action(UiTextEditAction::Backspace)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: start,
                focus: caret,
            },
            UiTextEditAction::Backspace,
        )
    }
}

// 与向前删除保持相同次序，词边界缺失时仍交给普通删除处理文档端点。
fn delete_next_word_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Delete);
    }
    let caret = state.caret.offset.min(state.text.len());
    let end = next_text_boundary(&state.text, caret, true);
    if end == caret {
        single_action(UiTextEditAction::Delete)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: caret,
                focus: end,
            },
            UiTextEditAction::Delete,
        )
    }
}

// 安全文本使用硬行范围，不查询被掩码文本的词结构；已有选区优先。
fn delete_to_line_start_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Backspace);
    }
    let caret = state.caret.offset.min(state.text.len());
    let start = line_start_boundary(&state.text, caret);
    if start == caret {
        single_action(UiTextEditAction::Backspace)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: start,
                focus: caret,
            },
            UiTextEditAction::Backspace,
        )
    }
}

// 保留已有选区；无选区时以当前硬行末尾生成选择与删除的有序组合。
fn delete_to_line_end_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if has_active_selection(state) {
        return single_action(UiTextEditAction::Delete);
    }
    let caret = state.caret.offset.min(state.text.len());
    let end = line_end_boundary(&state.text, caret);
    if end == caret {
        single_action(UiTextEditAction::Delete)
    } else {
        double_action(
            UiTextEditAction::SetSelection {
                anchor: caret,
                focus: end,
            },
            UiTextEditAction::Delete,
        )
    }
}

/// 有组合态时发出取消意图，否则发出当前位置的不延伸移动意图。
fn escape_actions(state: &UiEditableTextState) -> KeyboardTextEditActions {
    if state.composition.is_some() {
        single_action(UiTextEditAction::CancelComposition)
    } else {
        single_action(UiTextEditAction::MoveCaret {
            offset: state.caret.offset,
            extend_selection: false,
        })
    }
}

fn single_action(action: UiTextEditAction) -> KeyboardTextEditActions {
    KeyboardTextEditActions {
        first: action,
        second: None,
    }
}

fn double_action(first: UiTextEditAction, second: UiTextEditAction) -> KeyboardTextEditActions {
    KeyboardTextEditActions {
        first,
        second: Some(second),
    }
}

fn has_active_selection(state: &UiEditableTextState) -> bool {
    state
        .selection
        .as_ref()
        .is_some_and(|selection| selection.anchor != selection.focus)
}

fn home_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        line_start_boundary(&state.text, state.caret.offset)
    }
}

fn end_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        line_end_boundary(&state.text, state.caret.offset)
    }
}

// 普通垂直移动按字素列对齐；没有前一行时回到文档开头。
fn previous_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        0
    } else {
        previous_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(0)
    }
}

// 没有后一行时回到文档末尾，列对齐规则由共享文本边界工具决定。
fn next_line_offset(state: &UiEditableTextState, document_navigation: bool) -> usize {
    if document_navigation {
        state.text.len()
    } else {
        next_line_same_column_boundary(&state.text, state.caret.offset).unwrap_or(state.text.len())
    }
}

// 物理回退先匹配 A 的键码后才传入此标识，不能把任意命令键当作全选。
fn keyboard_requests_select_all(keyboard: &UiKeyboardInputEvent, logical_key: &str) -> bool {
    (keyboard.metadata.modifiers.control || keyboard.metadata.modifiers.super_key)
        && !keyboard.metadata.modifiers.alt
        && matches!(logical_key, "a" | "A")
}

// 共享工具没有前一边界时，以文档首端保持返回值为有效的源字节偏移。
fn previous_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        previous_word_boundary(text, offset).unwrap_or(0)
    } else {
        previous_grapheme_boundary(text, offset).unwrap_or(0)
    }
}

// 向后导航的缺省端点是源文本字节长度，与所有编辑意图的偏移单位一致。
fn next_text_boundary(text: &str, offset: usize, word_navigation: bool) -> usize {
    if word_navigation {
        next_word_boundary(text, offset).unwrap_or(text.len())
    } else {
        next_grapheme_boundary(text, offset).unwrap_or(text.len())
    }
}

#[cfg(test)]
#[path = "tests/edit_actions.rs"]
mod tests;
