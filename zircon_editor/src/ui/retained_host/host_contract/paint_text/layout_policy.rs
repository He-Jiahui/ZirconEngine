/// Chooses whether retained-host text uses the compact chrome path or Runtime Text wrapping.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::ui::retained_host::host_contract) enum HostTextLayoutPolicy {
    #[default]
    SingleLineEllipsis,
    WordWrap,
}
