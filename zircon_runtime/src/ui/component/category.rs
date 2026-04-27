use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum UiComponentCategory {
    Visual,
    Input,
    Numeric,
    Selection,
    Reference,
    Collection,
    Container,
    Feedback,
}
