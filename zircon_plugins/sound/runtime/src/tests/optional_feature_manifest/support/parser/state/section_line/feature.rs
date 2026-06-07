mod current;

use super::super::super::line::parse_optional_feature_line;
use super::super::OptionalFeatureParserState;

pub(super) fn parse_feature_section_line(state: &mut OptionalFeatureParserState, line: &str) {
    parse_optional_feature_line(line, current::required_current_feature(state));
}
