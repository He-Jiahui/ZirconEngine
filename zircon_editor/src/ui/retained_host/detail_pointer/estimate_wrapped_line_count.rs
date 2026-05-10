pub(super) fn estimate_wrapped_line_count(text: &str, width: f32, char_width: f32) -> usize {
    let columns = (width / char_width).floor().max(1.0) as usize;
    text.split('\n')
        .map(|line| {
            let count = line.chars().count();
            if count == 0 {
                1
            } else {
                count.div_ceil(columns).max(1)
            }
        })
        .sum::<usize>()
        .max(1)
}
