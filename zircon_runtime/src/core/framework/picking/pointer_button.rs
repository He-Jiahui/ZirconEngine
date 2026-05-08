#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PointerButton {
    Primary,
    Secondary,
    Middle,
    TouchContact,
    PenContact,
    Other(u16),
}
