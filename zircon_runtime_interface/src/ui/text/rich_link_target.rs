use core::fmt;
use std::sync::Arc;

use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error as _};

use crate::resource::{ResourceLocator, ResourceLocatorError, ResourceScheme};

/// Parser-approved engine-local destination carried unchanged to the UI host.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UiRichLinkTarget {
    locator: Arc<ResourceLocator>,
}

impl UiRichLinkTarget {
    pub fn parse(value: &str) -> Result<Self, UiRichLinkTargetError> {
        let value = value.trim();
        if value.is_empty() {
            return Err(UiRichLinkTargetError::Empty);
        }
        let locator = if value.contains("://") {
            ResourceLocator::parse(value)
        } else {
            let (path, label) = value
                .split_once('#')
                .map_or((value, None), |(path, label)| {
                    (path, Some(label.to_owned()))
                });
            ResourceLocator::new(ResourceScheme::Res, path, label)
        }
        .map_err(UiRichLinkTargetError::InvalidLocator)?;
        Self::from_locator(locator)
    }

    pub fn from_locator(locator: ResourceLocator) -> Result<Self, UiRichLinkTargetError> {
        if !matches!(
            locator.scheme(),
            ResourceScheme::Res
                | ResourceScheme::Library
                | ResourceScheme::Package
                | ResourceScheme::Builtin
        ) {
            return Err(UiRichLinkTargetError::UnsupportedScheme(locator.scheme()));
        }
        Ok(Self {
            locator: Arc::new(locator),
        })
    }

    pub fn locator(&self) -> &ResourceLocator {
        self.locator.as_ref()
    }

    pub fn matches_display(&self, value: &str) -> bool {
        self.locator.matches_display(value)
    }

    /// Heap bytes retained by the canonical locator components.
    pub fn retained_heap_bytes(&self) -> usize {
        self.locator
            .path()
            .len()
            .saturating_add(self.locator.label().map_or(0, str::len))
    }
}

impl fmt::Display for UiRichLinkTarget {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.locator.fmt(formatter)
    }
}

impl Serialize for UiRichLinkTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for UiRichLinkTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(&value).map_err(D::Error::custom)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum UiRichLinkTargetError {
    Empty,
    InvalidLocator(ResourceLocatorError),
    UnsupportedScheme(ResourceScheme),
}

impl fmt::Display for UiRichLinkTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("rich link target cannot be empty"),
            Self::InvalidLocator(error) => error.fmt(formatter),
            Self::UnsupportedScheme(scheme) => {
                write!(
                    formatter,
                    "rich link target scheme is not allowed: {scheme:?}"
                )
            }
        }
    }
}

impl std::error::Error for UiRichLinkTargetError {}

#[cfg(test)]
mod tests {
    use super::UiRichLinkTarget;

    #[test]
    fn accepts_only_canonical_engine_local_resource_schemes() {
        let target = UiRichLinkTarget::parse(" docs/./guide.zui#intro ").unwrap();
        assert!(target.matches_display("res://docs/guide.zui#intro"));

        for value in [
            "res://docs/guide.zui",
            "lib://docs/guide.zui",
            "package://com.zircon.docs/guide.zui",
            "builtin://docs/guide.zui",
        ] {
            assert!(UiRichLinkTarget::parse(value).is_ok(), "{value}");
        }
        for value in [
            "",
            "mem://transient/guide.zui",
            "https://example.com/guide",
            "res://../guide.zui",
            "res://docs/guide.zui#",
        ] {
            assert!(UiRichLinkTarget::parse(value).is_err(), "{value}");
        }
    }

    #[test]
    fn serde_revalidates_and_keeps_the_canonical_string_wire_shape() {
        let target = UiRichLinkTarget::parse("res://docs/./guide.zui").unwrap();
        assert_eq!(
            serde_json::to_string(&target).unwrap(),
            r#""res://docs/guide.zui""#
        );
        assert!(serde_json::from_str::<UiRichLinkTarget>(r#""mem://guide.zui""#).is_err());
    }
}
