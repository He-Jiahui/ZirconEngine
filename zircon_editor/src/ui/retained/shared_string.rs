use std::{fmt, ops::Deref, rc::Rc};

/// Single-threaded retained UI text. Cloning shares the UTF-8 allocation.
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct SharedString(Rc<str>);
impl From<String> for SharedString {
    fn from(text: String) -> Self {
        Self(text.into())
    }
}
impl From<&str> for SharedString {
    fn from(text: &str) -> Self {
        Self(text.into())
    }
}
impl Deref for SharedString {
    type Target = str;
    fn deref(&self) -> &str {
        &self.0
    }
}
impl AsRef<str> for SharedString {
    fn as_ref(&self) -> &str {
        self
    }
}
impl fmt::Display for SharedString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self)
    }
}
impl PartialEq<str> for SharedString {
    fn eq(&self, other: &str) -> bool {
        self.as_ref() == other
    }
}
impl PartialEq<&str> for SharedString {
    fn eq(&self, other: &&str) -> bool {
        self.as_ref() == *other
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn clones_share_text_and_preserve_string_semantics() {
        let text = SharedString::from(String::from("hello"));
        let copy = text.clone();
        assert!(Rc::ptr_eq(&text.0, &copy.0));
        assert_eq!(copy, "hello");
        assert_eq!(copy.to_string(), "hello");
    }
}
