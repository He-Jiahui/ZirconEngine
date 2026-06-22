mod asset_panes;
mod fallback;
mod kind;
mod native_panes;
mod target;
mod template_nodes;
mod viewport;

pub(in crate::ui::retained_host::host_contract) use self::target::dispatch_pane_button;
