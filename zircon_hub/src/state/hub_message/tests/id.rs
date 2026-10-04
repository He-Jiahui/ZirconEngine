use super::HubMessageId;
use crate::settings::HubLanguage;

#[test]
fn every_message_id_has_bilingual_templates_with_matching_placeholders() {
    for id in HubMessageId::all() {
        for language in [HubLanguage::English, HubLanguage::Chinese] {
            let template = id.template(language);
            assert!(!template.trim().is_empty(), "{id:?} missing {language:?}");
            for index in 0..id.param_count() {
                assert!(
                    template.contains(&format!("{{{index}}}")),
                    "{id:?} {language:?} template is missing placeholder {{{index}}}: {template}"
                );
            }
            assert!(
                !template.contains(&format!("{{{}}}", id.param_count())),
                "{id:?} {language:?} template has an out-of-range placeholder: {template}"
            );
        }
    }
}

#[test]
fn message_id_round_trips_through_stable_string_ids() {
    for id in HubMessageId::all() {
        assert_eq!(HubMessageId::from_str_id(id.as_str()), Some(id));
    }
}
