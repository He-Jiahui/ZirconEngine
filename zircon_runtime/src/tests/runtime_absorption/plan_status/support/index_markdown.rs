pub(in crate::tests::runtime_absorption::plan_status) fn runtime_index_row_for<'a>(
    index_source: &'a str,
    filename: &str,
) -> &'a str {
    index_source
        .lines()
        .find(|line| {
            let cells = markdown_table_cells(line);
            cells.len() >= 2 && first_backtick_value(cells[1]) == Some(filename)
        })
        .unwrap_or_else(|| panic!("runtime index should include subplan row for `{filename}`"))
}

pub(in crate::tests::runtime_absorption::plan_status) fn runtime_index_problem_row_for<'a>(
    index_source: &'a str,
    problem_id: &str,
    label: &str,
) -> &'a str {
    index_source
        .lines()
        .find(|line| {
            let cells = markdown_table_cells(line);
            cells.first().copied() == Some(problem_id)
        })
        .unwrap_or_else(|| panic!("Runtime index should keep the {problem_id} {label} problem row"))
}

pub(in crate::tests::runtime_absorption::plan_status) fn index_section_between<'a>(
    source: &'a str,
    start_anchor: &str,
    end_anchor: &str,
) -> &'a str {
    let start = source
        .find(start_anchor)
        .unwrap_or_else(|| panic!("runtime index should include section `{start_anchor}`"));
    let section = &source[start..];
    section
        .find(end_anchor)
        .map(|end| &section[..end])
        .unwrap_or(section)
}

pub(in crate::tests::runtime_absorption::plan_status) fn first_backtick_value(
    source: &str,
) -> Option<&str> {
    let (_, tail) = source.split_once('`')?;
    let (value, _) = tail.split_once('`')?;
    Some(value)
}

pub(in crate::tests::runtime_absorption::plan_status) fn leading_plan_id(
    source: &str,
) -> Option<&str> {
    let bytes = source.as_bytes();
    if bytes.len() >= 2 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() {
        Some(&source[..2])
    } else {
        None
    }
}

pub(in crate::tests::runtime_absorption::plan_status) fn referenced_plan_ids(
    source: &str,
) -> Vec<&str> {
    let bytes = source.as_bytes();
    let mut ids = Vec::new();

    for start in 0..bytes.len().saturating_sub(1) {
        let candidate = &bytes[start..start + 2];
        if candidate.iter().all(u8::is_ascii_digit) {
            let id = std::str::from_utf8(candidate)
                .expect("ASCII plan id candidate should be valid utf-8");
            if !ids.contains(&id) {
                ids.push(id);
            }
        }
    }

    ids
}

pub(in crate::tests::runtime_absorption::plan_status) fn markdown_table_cells(
    row: &str,
) -> Vec<&str> {
    row.trim()
        .trim_matches('|')
        .split('|')
        .map(str::trim)
        .collect()
}
