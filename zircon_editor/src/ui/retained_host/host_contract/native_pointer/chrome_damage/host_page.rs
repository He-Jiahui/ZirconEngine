mod status;
mod tabs;
mod template_nodes;

use crate::ui::retained_host::host_contract::data::{FrameRect, HostWindowPresentationData};

use self::status::union_host_page_status_damage;
use self::tabs::union_host_page_tab_damage;
use self::template_nodes::union_host_page_template_node_damage;

pub(super) fn host_page_tab_damage_frame(
    presentation: &HostWindowPresentationData,
) -> Option<FrameRect> {
    let mut damage = None;
    damage = union_host_page_tab_damage(damage, presentation);
    damage = union_host_page_template_node_damage(damage, presentation);
    damage = union_host_page_status_damage(damage, presentation);
    damage
}
