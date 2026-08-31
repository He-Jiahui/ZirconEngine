use std::error::Error;
use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityPath {
    raw: String,
    segments: Vec<String>,
}

impl EntityPath {
    pub fn new(segments: Vec<String>) -> Result<Self, PathParseError> {
        if segments.is_empty() {
            return Err(PathParseError::new(
                "entity path must contain at least one segment",
            ));
        }
        if segments.iter().any(|segment| segment.trim().is_empty()) {
            return Err(PathParseError::new(
                "entity path segments must not be empty or whitespace",
            ));
        }

        Ok(Self {
            raw: segments.join("/"),
            segments,
        })
    }

    pub fn parse(path: &str) -> Result<Self, PathParseError> {
        let mut segments = Vec::with_capacity(path.len().saturating_div(2).max(1));
        segments.extend(
            path.split('/')
                .map(str::trim)
                .filter(|segment| !segment.is_empty())
                .map(ToOwned::to_owned),
        );

        Self::new(segments)
    }

    pub fn segments(&self) -> &[String] {
        &self.segments
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

#[cfg(test)]
#[path = "entity_path/single_scan_parse_tests.rs"]
mod single_scan_parse_tests;

impl fmt::Display for EntityPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentPropertyPath {
    raw: String,
    component: String,
    property_segments: Vec<String>,
}

impl ComponentPropertyPath {
    pub fn new(
        component: impl Into<String>,
        property_segments: Vec<String>,
    ) -> Result<Self, PathParseError> {
        let component = component.into();
        if component.trim().is_empty() {
            return Err(PathParseError::new(
                "component path must include a component name",
            ));
        }
        let property_len = validated_property_segments_len(&property_segments)?;

        Ok(Self {
            raw: component_property_path_raw(&component, &property_segments, property_len),
            component,
            property_segments,
        })
    }

    pub fn parse(path: &str) -> Result<Self, PathParseError> {
        let mut segments = path
            .split('.')
            .map(str::trim)
            .filter(|segment| !segment.is_empty());
        let component = segments.next().ok_or_else(|| {
            PathParseError::new("component property path must use `<component>.<property>` syntax")
        })?;
        let property_segments = segments.map(ToOwned::to_owned).collect::<Vec<_>>();

        Self::new(component.to_string(), property_segments)
    }

    pub fn component(&self) -> &str {
        &self.component
    }

    pub fn property_segments(&self) -> &[String] {
        &self.property_segments
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

fn validated_property_segments_len(property_segments: &[String]) -> Result<usize, PathParseError> {
    if property_segments.is_empty() {
        return Err(PathParseError::new(
            "component path must include at least one property segment",
        ));
    }

    let mut property_len = 0;
    for segment in property_segments {
        if segment.trim().is_empty() {
            return Err(PathParseError::new(
                "component property segments must not be empty or whitespace",
            ));
        }
        property_len += segment.len();
    }

    Ok(property_len)
}

fn component_property_path_raw(
    component: &str,
    property_segments: &[String],
    property_len: usize,
) -> String {
    let mut raw = String::with_capacity(component.len() + property_len + property_segments.len());
    raw.push_str(component);
    for segment in property_segments {
        raw.push('.');
        raw.push_str(segment);
    }
    raw
}

impl fmt::Display for ComponentPropertyPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.raw)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PathParseError {
    message: String,
}

impl PathParseError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for PathParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for PathParseError {}
