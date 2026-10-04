use super::transmission_step_range;

#[test]
fn render_transmission_steps_partition_commands_without_overlap() {
    let ranges = (0..3)
        .filter_map(|step| transmission_step_range(10, 3, step))
        .collect::<Vec<_>>();

    assert_eq!(ranges, vec![0..4, 4..7, 7..10]);
}

#[test]
fn render_transmission_steps_do_not_emit_empty_ranges() {
    let ranges = (0..4)
        .filter_map(|step| transmission_step_range(2, 4, step))
        .collect::<Vec<_>>();

    assert_eq!(ranges, vec![0..1, 1..2]);
    assert_eq!(transmission_step_range(0, 4, 0), None);
    assert_eq!(transmission_step_range(2, 0, 0), None);
}
