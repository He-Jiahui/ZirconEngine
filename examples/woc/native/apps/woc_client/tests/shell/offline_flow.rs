use woc_client::{
    CharacterNameError, OfflinePlayerClass, OfflineSessionError, OfflineShellAction,
    OfflineShellController, OfflineShellError, OfflineShellState, OFFLINE_SESSION_LAUNCH_VERSION,
};

const LAUNCH_SEED: u32 = 20_061;

fn opened_picker() -> OfflineShellController {
    let mut shell = OfflineShellController::default();
    shell
        .open_offline_picker()
        .expect("mode selection should open the offline picker");
    shell
}

#[test]
fn shell_starts_at_mode_selection_without_a_prepared_world() {
    let shell = OfflineShellController::default();

    assert_eq!(shell.state(), OfflineShellState::ModeSelection);
    assert_eq!(shell.draft().player_class(), None);
    assert_eq!(shell.draft().skin_variant(), 0);
    assert_eq!(shell.prepared_launch(), None);
}

#[test]
fn returning_to_mode_selection_clears_name_and_reopening_resets_class_and_skin() {
    let mut shell = opened_picker();
    shell.set_name("Mira").expect("picker name");
    shell
        .set_class(OfflinePlayerClass::Mage)
        .expect("picker class");
    shell.set_skin(3).expect("picker skin");
    shell
        .back_to_mode_selection()
        .expect("picker can return to mode selection");
    assert_eq!(shell.draft().raw_name(), "");

    shell
        .open_offline_picker()
        .expect("offline picker can be reopened");

    assert_eq!(shell.draft().raw_name(), "");
    assert_eq!(shell.draft().player_class(), None);
    assert_eq!(shell.draft().skin_variant(), 0);
}

#[test]
fn picker_edits_are_rejected_outside_the_picker_without_mutation() {
    let mut shell = OfflineShellController::default();

    assert_eq!(
        shell
            .set_name("Mira")
            .expect_err("mode selection is not editable"),
        OfflineShellError::InvalidTransition {
            action: OfflineShellAction::SetName,
            state: OfflineShellState::ModeSelection,
        }
    );
    assert_eq!(shell.draft().raw_name(), "");
    assert_eq!(shell.state(), OfflineShellState::ModeSelection);
}

#[test]
fn picker_rejects_a_skin_outside_the_selected_class_catalog_without_mutation() {
    let mut shell = opened_picker();
    shell
        .set_class(OfflinePlayerClass::Paladin)
        .expect("picker class");

    assert_eq!(
        shell
            .set_skin(2)
            .expect_err("paladin exposes only skin indices zero and one"),
        OfflineShellError::Session(OfflineSessionError::InvalidSkinVariant {
            player_class: OfflinePlayerClass::Paladin,
            skin_variant: 2,
            skin_count: 2,
        })
    );
    assert_eq!(shell.draft().skin_variant(), 0);
    assert_eq!(shell.state(), OfflineShellState::OfflinePicker);
}

#[test]
fn invalid_picker_submission_is_atomic_and_keeps_the_draft_visible() {
    let mut shell = opened_picker();
    shell
        .set_class(OfflinePlayerClass::Druid)
        .expect("picker class");
    shell.set_skin(2).expect("picker skin");
    shell.set_name("Map\tMaker").expect("raw picker name");

    assert_eq!(
        shell
            .submit_offline_picker()
            .expect_err("invalid name must not begin world entry"),
        OfflineShellError::Session(OfflineSessionError::CharacterName(
            CharacterNameError::InvalidCharacter { index: 3 }
        ))
    );
    assert_eq!(shell.state(), OfflineShellState::OfflinePicker);
    assert_eq!(shell.draft().raw_name(), "Map\tMaker");
    assert_eq!(
        shell.draft().player_class(),
        Some(OfflinePlayerClass::Druid)
    );
    assert_eq!(shell.draft().skin_variant(), 2);
    assert_eq!(shell.prepared_launch(), None);
}

#[test]
fn valid_submission_prepares_one_fixed_seed_launch_then_opens_welcome() {
    let mut shell = opened_picker();
    shell
        .set_class(OfflinePlayerClass::Shaman)
        .expect("picker class");
    shell.set_skin(2).expect("picker skin");
    shell
        .set_name("  Storm  Caller  ")
        .expect("raw picker name");

    let launch = shell
        .submit_offline_picker()
        .expect("valid picker submission");

    assert_eq!(shell.state(), OfflineShellState::Welcome);
    assert_eq!(launch.schema_version, OFFLINE_SESSION_LAUNCH_VERSION);
    assert_eq!(launch.player_class, OfflinePlayerClass::Shaman);
    assert_eq!(launch.player_name, "Storm  Caller");
    assert_eq!(launch.skin_variant, 2);
    assert_eq!(launch.world_seed, LAUNCH_SEED);
    assert_eq!(launch.preference_scope(), "offline:shaman:Storm  Caller");
    assert_eq!(shell.prepared_launch(), Some(&launch));
}

#[test]
fn welcome_continue_is_one_shot_and_moves_to_loading() {
    let mut shell = opened_picker();
    shell
        .set_class(OfflinePlayerClass::Warrior)
        .expect("picker class");
    shell.set_name("Mira").expect("picker name");
    let launch = shell
        .submit_offline_picker()
        .expect("valid picker submission");

    assert_eq!(
        shell.continue_from_welcome().expect("first Continue"),
        launch
    );
    assert_eq!(shell.state(), OfflineShellState::Loading);
    assert_eq!(
        shell
            .continue_from_welcome()
            .expect_err("Continue cannot launch twice"),
        OfflineShellError::InvalidTransition {
            action: OfflineShellAction::ContinueWelcome,
            state: OfflineShellState::Loading,
        }
    );
    assert_eq!(shell.prepared_launch(), Some(&launch));
}

#[test]
fn loading_completion_enters_world_once_and_retains_session_identity() {
    let mut shell = opened_picker();
    shell
        .set_class(OfflinePlayerClass::Hunter)
        .expect("picker class");
    shell.set_name("Vale Scout").expect("picker name");
    let launch = shell
        .submit_offline_picker()
        .expect("valid picker submission");
    shell.continue_from_welcome().expect("welcome Continue");

    assert_eq!(shell.finish_loading().expect("world became ready"), launch);
    assert_eq!(shell.state(), OfflineShellState::InWorld);
    assert_eq!(shell.prepared_launch(), Some(&launch));
    assert_eq!(
        shell
            .finish_loading()
            .expect_err("world readiness cannot commit twice"),
        OfflineShellError::InvalidTransition {
            action: OfflineShellAction::FinishLoading,
            state: OfflineShellState::InWorld,
        }
    );
}
