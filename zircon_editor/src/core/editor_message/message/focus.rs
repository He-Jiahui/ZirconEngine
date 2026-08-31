use serde::{Deserialize, Serialize};

use crate::core::editor_message::SelectionDomain;
use crate::core::play::WorldDomain;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum FocusMessage {
    SelectionChanged {
        domain: SelectionDomain,
        revision: u64,
    },
    FocusObject {
        domain: WorldDomain,
        entity: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::play::PlayInstanceId;

    #[test]
    fn selection_domains_preserve_play_instance_identity() {
        let first = PlayInstanceId::for_test(1);
        let second = PlayInstanceId::for_test(2);

        let first_domain = SelectionDomain::Scene(WorldDomain::Play(first));
        let second_domain = SelectionDomain::Scene(WorldDomain::Play(second));

        assert_ne!(first_domain, second_domain);
        assert_ne!(first_domain, SelectionDomain::edit_scene());
        assert_eq!(
            serde_json::from_slice::<SelectionDomain>(
                &serde_json::to_vec(&first_domain).expect("play selection domain must encode")
            )
            .expect("play selection domain must decode"),
            first_domain
        );
    }
}
