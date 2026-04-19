use crate::binding::{UiBindingParseError, UiEventBinding};

#[derive(Clone, Debug, Default)]
pub struct UiBindingCodec;

impl UiBindingCodec {
    pub fn format(binding: &UiEventBinding) -> String {
        binding.native_binding()
    }

    pub fn parse(input: &str) -> Result<UiEventBinding, UiBindingParseError> {
        UiEventBinding::parse_native_binding(input)
    }
}
