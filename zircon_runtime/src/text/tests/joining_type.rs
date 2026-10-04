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
