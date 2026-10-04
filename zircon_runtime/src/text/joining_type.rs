use icu_properties::props::JoiningType;
use icu_properties::{CodePointMapData, CodePointMapDataBorrowed};

static COMPILED_JOINING_TYPES: CodePointMapDataBorrowed<'static, JoiningType> =
    CodePointMapData::<JoiningType>::new();

/// 两端对齐用来筛选阿拉伯延展候选的 Unicode 连写属性表，按逻辑字符顺序判断连接关系。
/// 候选仍须由塑形后端按所选字体与语言确认；属性相连不等于可直接插入延展字形。
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
#[path = "tests/joining_type.rs"]
mod tests;
