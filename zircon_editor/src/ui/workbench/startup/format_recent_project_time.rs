/// 使用上层同轮时钟生成相对时间；未知和未来时间都有显示回退，不用于事件排序。
pub(crate) fn format_recent_project_time(last_opened_unix_ms: u64, now_unix_ms: u64) -> String {
    if last_opened_unix_ms == 0 {
        return "Unknown".to_string();
    }
    let delta_seconds = now_unix_ms.saturating_sub(last_opened_unix_ms) / 1_000;
    if delta_seconds < 60 {
        "Just now".to_string()
    } else if delta_seconds < 60 * 60 {
        elapsed_count_label(delta_seconds / 60, 'm')
    } else if delta_seconds < 60 * 60 * 24 {
        elapsed_count_label(delta_seconds / (60 * 60), 'h')
    } else {
        elapsed_count_label(delta_seconds / (60 * 60 * 24), 'd')
    }
}

fn elapsed_count_label(mut count: u64, unit: char) -> String {
    let mut digits = [0_u8; 20];
    let mut start = digits.len();
    loop {
        start -= 1;
        digits[start] = b'0' + (count % 10) as u8;
        count /= 10;
        if count == 0 {
            break;
        }
    }
    let digit_count = digits.len() - start;
    let mut label = String::with_capacity(digit_count + "m ago".len());
    for digit in &digits[start..] {
        label.push(char::from(*digit));
    }
    label.push(unit);
    label.push_str(" ago");
    label
}

#[cfg(test)]
#[path = "tests/format_recent_project_time_performance_tests.rs"]
mod performance_tests;
