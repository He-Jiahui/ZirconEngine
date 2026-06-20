use super::super::super::construction;
use super::super::super::*;

impl RuntimeSessionArchive {
    pub fn from_versioned_json(json: &str) -> Result<Self, RuntimeSessionArchiveError> {
        construction::from_versioned_json(json)
    }

    pub fn to_versioned_json_pretty(&self) -> Result<String, RuntimeSessionArchiveError> {
        construction::to_versioned_json_pretty(self)
    }
}
