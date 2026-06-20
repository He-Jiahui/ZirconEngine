pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn is_date_time_picker(
    component_role: &str,
    role: &str,
) -> bool {
    super::super::matches_any_role(
        component_role,
        role,
        &[
            "mui-x-date-time-pickers",
            "DateTimePickers",
            "DatePicker",
            "TimePicker",
        ],
    )
}
