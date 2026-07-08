use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const EMPTY_MESSAGE: &str = "No commands found";
const SEARCH_ICON: &str = "search";
const SEARCH_PLACEHOLDER: &str = "Search commands";

pub(super) struct CommandPaletteSearchText<'a> {
    pub(super) value: &'a str,
    pub(super) placeholder: bool,
}

pub(super) fn command_palette_text_style() -> UiTextRunPaintStyle {
    UiTextRunPaintStyle::default()
}

pub(super) fn command_palette_empty_message() -> &'static str {
    EMPTY_MESSAGE
}

pub(super) fn command_palette_search_icon() -> &'static str {
    SEARCH_ICON
}

pub(super) fn command_palette_search_text(query: &str) -> CommandPaletteSearchText<'_> {
    if query.trim().is_empty() {
        CommandPaletteSearchText {
            value: SEARCH_PLACEHOLDER,
            placeholder: true,
        }
    } else {
        CommandPaletteSearchText {
            value: query,
            placeholder: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn command_palette_search_text_uses_placeholder_only_for_empty_queries() {
        let empty = command_palette_search_text("   ");
        assert_eq!(empty.value, SEARCH_PLACEHOLDER);
        assert!(empty.placeholder);

        let query = command_palette_search_text("lights");
        assert_eq!(query.value, "lights");
        assert!(!query.placeholder);
    }
}
