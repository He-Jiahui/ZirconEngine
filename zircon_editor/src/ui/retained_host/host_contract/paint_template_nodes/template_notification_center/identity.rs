//! 通知中心身份、标题和空态文案的边界；标题直接消费上游未读/省略计数，避免在每帧扫描记录。
//! 上游应提供已本地化的标题与空态文本；缺省英文只是宿主无内容时的fallback。

use super::super::super::data::TemplatePaneNodeData;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_notification_center(
    node: &TemplatePaneNodeData,
) -> bool {
    node.role.as_str() == "NotificationCenter"
        || node.component_role.as_str() == "notification-center"
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn header_text(
    node: &TemplatePaneNodeData,
) -> String {
    let title = non_empty(node.text.as_str()).unwrap_or("Notifications");
    notification_header_text(
        title,
        node.notification_unread_count,
        node.notification_overflow_count,
    )
}

fn notification_header_text(title: &str, unread_count: usize, overflow_count: usize) -> String {
    if unread_count == 0 && overflow_count == 0 {
        return title.to_string();
    }
    let (unread_digits, unread_start) = if unread_count == 0 {
        ([0; 20], 20)
    } else {
        decimal_digits(unread_count)
    };
    let (overflow_digits, overflow_start) = if overflow_count == 0 {
        ([0; 20], 20)
    } else {
        decimal_digits(overflow_count)
    };
    let unread_digits = &unread_digits[unread_start..];
    let overflow_digits = &overflow_digits[overflow_start..];
    let capacity = title.len()
        + usize::from(unread_count != 0) * (" (".len() + unread_digits.len() + ")".len())
        + usize::from(overflow_count != 0)
            * (" +".len() + overflow_digits.len() + " omitted".len());
    let mut header = String::with_capacity(capacity);
    header.push_str(title);
    if unread_count != 0 {
        header.push_str(" (");
        push_ascii_digits(&mut header, unread_digits);
        header.push(')');
    }
    if overflow_count != 0 {
        header.push_str(" +");
        push_ascii_digits(&mut header, overflow_digits);
        header.push_str(" omitted");
    }
    header
}

fn decimal_digits(mut value: usize) -> ([u8; 20], usize) {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (value % 10) as u8;
        value /= 10;
        if value == 0 {
            break;
        }
    }
    (digits, start)
}

fn push_ascii_digits(output: &mut String, digits: &[u8]) {
    for digit in digits {
        output.push(char::from(*digit));
    }
}

// TODO: [CR-EDITOR-PAINT-OVERLAY-0009] 缺省标题、空态与计数后缀由此处英文生成；
// 确认宿主本地化投影是否覆盖所有状态，特别是未读数和省略数变化后的标题。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn empty_text(
    node: &TemplatePaneNodeData,
) -> String {
    non_empty(node.value_text.as_str())
        .unwrap_or("No notifications")
        .to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    let value = value.trim();
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
