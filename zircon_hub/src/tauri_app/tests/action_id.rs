use super::*;

#[test]
fn every_action_id_round_trips_between_as_str_and_from_str() {
    assert_eq!(HubActionId::ALL.len(), 32);
    for action in HubActionId::ALL {
        assert_eq!(HubActionId::from_str(action.as_str()), Some(action));
    }
}

#[test]
fn archived_aliases_and_whitespace_resolve_to_canonical_actions() {
    assert_eq!(HubActionId::from_str("page"), Some(HubActionId::ShowPage));
    assert_eq!(
        HubActionId::from_str("project-subpage"),
        Some(HubActionId::ShowProjectSubpage)
    );
    assert_eq!(
        HubActionId::from_str("open-project"),
        Some(HubActionId::SelectProject)
    );
    assert_eq!(
        HubActionId::from_str(" build-project "),
        Some(HubActionId::BuildProject)
    );
    assert_eq!(HubActionId::from_str("upload-to-cloud"), None);
}
