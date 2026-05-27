use super::shared::*;
use zircon_runtime_interface::ui::component::UiPropSchema;

pub(super) fn descriptors() -> Vec<UiComponentDescriptor> {
    vec![
        click_away_listener(),
        portal(),
        no_ssr(),
        css_baseline(),
        scoped_css_baseline(),
        init_color_scheme_script(),
        use_media_query(),
    ]
}

fn click_away_listener() -> UiComponentDescriptor {
    add_props(
        composite(
            "ClickAwayListener",
            "Click Away Listener",
            UiComponentCategory::Container,
            "click-away-listener",
        ),
        [
            bool_prop("behavior_utility", true),
            bool_prop("disableReactTree", false),
            mui_enum_prop(
                "mouseEvent",
                "onClick",
                [
                    "false",
                    "onClick",
                    "onMouseDown",
                    "onMouseUp",
                    "onPointerDown",
                    "onPointerUp",
                ],
            ),
            mui_enum_prop(
                "touchEvent",
                "onTouchEnd",
                ["false", "onTouchEnd", "onTouchStart"],
            ),
        ],
    )
    .slot(UiSlotSchema::new("content").required(true))
    .event(UiComponentEventKind::ClosePopup)
}

fn portal() -> UiComponentDescriptor {
    add_props(
        composite("Portal", "Portal", UiComponentCategory::Container, "portal"),
        [
            bool_prop("behavior_utility", true),
            any_prop("container"),
            default_string_prop("container_id", ""),
            bool_prop("disablePortal", false),
            bool_prop("disable_portal", false),
        ],
    )
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn no_ssr() -> UiComponentDescriptor {
    add_props(
        composite("NoSsr", "No Ssr", UiComponentCategory::Container, "no-ssr"),
        [
            bool_prop("behavior_utility", true),
            bool_prop("defer", false),
            any_prop("fallback"),
        ],
    )
    .slot(UiSlotSchema::new("content").multiple(true))
    .slot(UiSlotSchema::new("fallback"))
}

fn css_baseline() -> UiComponentDescriptor {
    composite(
        "CssBaseline",
        "Css Baseline",
        UiComponentCategory::Container,
        "css-baseline",
    )
    .with_prop(bool_prop("behavior_utility", true))
    .with_prop(bool_prop("enableColorScheme", false))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn scoped_css_baseline() -> UiComponentDescriptor {
    composite(
        "ScopedCssBaseline",
        "Scoped CSS Baseline",
        UiComponentCategory::Container,
        "scoped-css-baseline",
    )
    .with_prop(bool_prop("behavior_utility", true))
    .with_prop(bool_prop("enableColorScheme", false))
    .with_prop(default_string_prop("component", "div"))
    .slot(UiSlotSchema::new("content").multiple(true))
}

fn init_color_scheme_script() -> UiComponentDescriptor {
    add_props(
        primitive(
            "InitColorSchemeScript",
            "Init Color Scheme Script",
            UiComponentCategory::Container,
            "init-color-scheme-script",
        ),
        [
            bool_prop("behavior_utility", true),
            mui_enum_prop("defaultMode", "system", ["dark", "light", "system"]),
            default_string_prop("defaultLightColorScheme", "light"),
            default_string_prop("defaultDarkColorScheme", "dark"),
            default_string_prop("colorSchemeNode", "document.documentElement"),
            default_string_prop("modeStorageKey", "mui-mode"),
            default_string_prop("colorSchemeStorageKey", "mui-color-scheme"),
            default_string_prop("attribute", "data-mui-color-scheme"),
            default_string_prop("nonce", ""),
        ],
    )
}

fn use_media_query() -> UiComponentDescriptor {
    add_props(
        primitive(
            "UseMediaQuery",
            "Use Media Query",
            UiComponentCategory::Container,
            "use-media-query",
        ),
        [
            bool_prop("behavior_utility", true),
            string_prop("query"),
            bool_prop("defaultMatches", false),
            bool_prop("default_matches", false),
            any_prop("matchMedia"),
            bool_prop("noSsr", false),
            bool_prop("no_ssr", false),
            any_prop("ssrMatchMedia"),
            bool_prop("matches", false),
            string_prop("up"),
            string_prop("down"),
            any_prop("between"),
            string_prop("breakpoint"),
        ],
    )
}

fn add_props<const N: usize>(
    mut descriptor: UiComponentDescriptor,
    props: [UiPropSchema; N],
) -> UiComponentDescriptor {
    for prop in props {
        descriptor = descriptor.with_prop(prop);
    }
    descriptor
}

fn mui_enum_prop<const N: usize>(
    name: &str,
    default: &str,
    options: [&'static str; N],
) -> UiPropSchema {
    enum_prop_with_options(
        name,
        default,
        options.into_iter().map(enum_option_descriptor),
    )
}
