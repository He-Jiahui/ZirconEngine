#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum TemplateComponentFamily {
    Button,
    IconButton,
    TextInput,
    KeySelector,
    Slider,
    Checkbox,
    Radio,
    Toggle,
    Dropdown,
    Tab,
    SegmentedControl,
    List,
    ListRow,
    TreeRow,
    Table,
    TableRow,
    Popup,
    Tooltip,
    Alert,
    Container,
    Drawer,
    Window,
}

impl TemplateComponentFamily {
    pub(in crate::ui::retained_host::host_contract) fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::IconButton => "icon-button",
            Self::TextInput => "text-input",
            Self::KeySelector => "key-selector",
            Self::Slider => "slider",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Toggle => "toggle",
            Self::Dropdown => "dropdown",
            Self::Tab => "tab",
            Self::SegmentedControl => "segmented-control",
            Self::List => "list",
            Self::ListRow => "list-row",
            Self::TreeRow => "tree-row",
            Self::Table => "table",
            Self::TableRow => "table-row",
            Self::Popup => "popup",
            Self::Tooltip => "tooltip",
            Self::Alert => "alert",
            Self::Container => "container",
            Self::Drawer => "drawer",
            Self::Window => "window",
        }
    }
}
