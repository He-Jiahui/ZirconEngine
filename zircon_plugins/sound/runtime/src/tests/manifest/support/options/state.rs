#[derive(Default)]
pub(super) struct PendingOptionManifest {
    pub(super) key: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) value_type: Option<String>,
    pub(super) default_value: Option<String>,
    pub(super) enum_values: Vec<String>,
    pub(super) required_capability: Option<String>,
}

impl PendingOptionManifest {
    pub(super) fn push_into(
        &mut self,
        options: &mut Vec<zircon_runtime::plugin::PluginOptionManifest>,
    ) {
        let Some(key) = self.key.take() else {
            self.enum_values.clear();
            return;
        };
        let mut option = zircon_runtime::plugin::PluginOptionManifest::new(
            key,
            self.display_name
                .take()
                .expect("sound option should declare display_name"),
            self.value_type
                .take()
                .expect("sound option should declare value_type"),
            self.default_value
                .take()
                .expect("sound option should declare default_value"),
        );
        let values = std::mem::take(&mut self.enum_values);
        if !values.is_empty() {
            option = option.with_enum_values(values);
        }
        if let Some(capability) = self.required_capability.take() {
            option = option.with_required_capability(capability);
        }
        options.push(option);
    }
}
