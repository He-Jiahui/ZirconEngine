use std::fmt;

use super::AssetTypeId;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetTypeRegistryError {
    DuplicateFieldOwner {
        asset_type: AssetTypeId,
        field: &'static str,
        first_owner: String,
        second_owner: String,
    },
    DuplicateEntryOwner {
        asset_type: AssetTypeId,
        collection: &'static str,
        entry_id: String,
        first_owner: String,
        second_owner: String,
    },
    IncompleteDefinition {
        asset_type: AssetTypeId,
        missing_fields: Vec<&'static str>,
    },
    EmptyRequiredField {
        asset_type: AssetTypeId,
        field: &'static str,
    },
}

impl fmt::Display for AssetTypeRegistryError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateFieldOwner {
                asset_type,
                field,
                first_owner,
                second_owner,
            } => write!(
                formatter,
                "asset type `{asset_type}` field `{field}` is owned by both `{first_owner}` and `{second_owner}`"
            ),
            Self::DuplicateEntryOwner {
                asset_type,
                collection,
                entry_id,
                first_owner,
                second_owner,
            } => write!(
                formatter,
                "asset type `{asset_type}` {collection} entry `{entry_id}` is owned by both `{first_owner}` and `{second_owner}`"
            ),
            Self::IncompleteDefinition {
                asset_type,
                missing_fields,
            } => write!(
                formatter,
                "asset type `{asset_type}` is incomplete; missing {}",
                missing_fields.join(", ")
            ),
            Self::EmptyRequiredField { asset_type, field } => write!(
                formatter,
                "asset type `{asset_type}` required field `{field}` is empty"
            ),
        }
    }
}

impl std::error::Error for AssetTypeRegistryError {}
