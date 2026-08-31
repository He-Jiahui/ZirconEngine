use icu_properties::props::JoiningType;
use icu_properties::{CodePointMapData, CodePointMapDataBorrowed};

static COMPILED_JOINING_TYPES: CodePointMapDataBorrowed<'static, JoiningType> =
    CodePointMapData::<JoiningType>::new();

#[derive(Clone, Copy)]
pub(crate) struct TextJoiningTypeMap(CodePointMapDataBorrowed<'static, JoiningType>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TextJoiningType {
    NonJoining,
    JoinCausing,
    DualJoining,
    LeftJoining,
    RightJoining,
    Transparent,
}

impl TextJoiningType {
    pub(crate) const fn joins_with_following_logical_character(self) -> bool {
        matches!(
            self,
            Self::JoinCausing | Self::DualJoining | Self::LeftJoining
        )
    }

    pub(crate) const fn joins_with_preceding_logical_character(self) -> bool {
        matches!(
            self,
            Self::JoinCausing | Self::DualJoining | Self::RightJoining
        )
    }

    pub(crate) const fn is_transparent(self) -> bool {
        matches!(self, Self::Transparent)
    }
}

impl TextJoiningTypeMap {
    pub(crate) fn get(self, ch: char) -> TextJoiningType {
        let joining_type = self.0.get(ch);
        if joining_type == JoiningType::JoinCausing {
            TextJoiningType::JoinCausing
        } else if joining_type == JoiningType::DualJoining {
            TextJoiningType::DualJoining
        } else if joining_type == JoiningType::LeftJoining {
            TextJoiningType::LeftJoining
        } else if joining_type == JoiningType::RightJoining {
            TextJoiningType::RightJoining
        } else if joining_type == JoiningType::Transparent {
            TextJoiningType::Transparent
        } else {
            TextJoiningType::NonJoining
        }
    }
}

pub(crate) fn compiled_joining_type_map() -> TextJoiningTypeMap {
    TextJoiningTypeMap(COMPILED_JOINING_TYPES)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compiled_map_exposes_all_joining_directions() {
        let map = compiled_joining_type_map();

        assert_eq!(map.get('\u{0620}'), TextJoiningType::DualJoining);
        assert_eq!(map.get('\u{0870}'), TextJoiningType::RightJoining);
        assert_eq!(map.get('\u{10acd}'), TextJoiningType::LeftJoining);
        assert_eq!(map.get('\u{0640}'), TextJoiningType::JoinCausing);
        assert_eq!(map.get('\u{064e}'), TextJoiningType::Transparent);
        assert_eq!(map.get('A'), TextJoiningType::NonJoining);
    }
}
