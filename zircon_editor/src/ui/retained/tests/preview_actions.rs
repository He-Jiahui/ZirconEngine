use super::{
    extensions, is_workbench_preview_action, WORKBENCH_PREVIEW_ACTION_IDS,
    WORKBENCH_PREVIEW_ACTION_ID_SET,
};

fn preview_action_ids() -> impl Iterator<Item = &'static str> {
    WORKBENCH_PREVIEW_ACTION_IDS
        .iter()
        .chain(extensions::WORKBENCH_EXTENSION_PREVIEW_ACTION_IDS.iter())
        .copied()
}

#[test]
fn workbench_preview_action_registry_covers_module_and_component_lab_samples() {
    for action_id in [
        "workbench.module.effect.select",
        "workbench.module.ability.select",
        "workbench.module.material.select",
        "workbench.module.render.select",
        "workbench.module.browse.invoke",
        "workbench.module.compile.invoke",
        "workbench.module.effect.search.edit",
        "workbench.module.effect.stack_row.select",
        "workbench.module.effect.modifier_healing.select",
        "workbench.module.effect.magnitude.commit",
        "workbench.module.material.node_roughness.select",
        "workbench.module.material.domain.edit",
        "workbench.module.behavior.node_cooldown.select",
        "workbench.module.assets.table_texture.select",
        "workbench.module.vfx.system.edit",
        "workbench.module.ability.phase_cost.select",
        "workbench.module.ability.name.edit",
        "workbench.module.tags.ability_activate.select",
        "workbench.module.perception.sight_cone.select",
        "workbench.module.perception.config.edit",
        "workbench.module.render.lighting_pass.select",
        "workbench.module.render.pipeline.edit",
        "workbench.module.hud.minimap.select",
        "workbench.module.hud.screen.edit",
        "workbench.extension.shader_editor.open",
        "workbench.extension.shader_editor.fragment_row.select",
        "workbench.extension.shader_editor.compile.invoke",
        "workbench.extension.shader_editor.entry.edit",
        "workbench.extension.lighting_bake.open",
        "workbench.extension.lighting_bake.bake.invoke",
        "workbench.extension.lighting_bake.quality.edit",
        "workbench.extension.post_process.open",
        "workbench.extension.post_process.apply.invoke",
        "workbench.extension.post_process.profile.edit",
        "workbench.extension.montage_editor.open",
        "workbench.extension.montage_editor.apply.invoke",
        "workbench.extension.montage_editor.blend.edit",
        "workbench.extension.blend_space.open",
        "workbench.extension.blend_space.apply.invoke",
        "workbench.extension.blend_space.interpolation.edit",
        "workbench.extension.pose_library.open",
        "workbench.extension.pose_library.apply.invoke",
        "workbench.extension.pose_library.mirror.edit",
        "workbench.extension.retarget.open",
        "workbench.extension.retarget.apply.invoke",
        "workbench.extension.retarget.solver.edit",
        "workbench.extension.control_rig.open",
        "workbench.extension.control_rig.validate.invoke",
        "workbench.extension.control_rig.weight.edit",
        "workbench.extension.motion_matching.open",
        "workbench.extension.motion_matching.rebuild.invoke",
        "workbench.extension.motion_matching.cost.edit",
        "workbench.extension.animation_compression.open",
        "workbench.extension.animation_compression.compress.invoke",
        "workbench.extension.animation_compression.tolerance.edit",
        "workbench.extension.data_table.open",
        "workbench.extension.data_table.validate.invoke",
        "workbench.extension.data_table.type.edit",
        "workbench.extension.source_control.open",
        "workbench.extension.source_control.submit.invoke",
        "workbench.extension.source_control.gate.edit",
        "workbench.extension.build_export.open",
        "workbench.extension.build_export.package.invoke",
        "workbench.extension.build_export.channel.edit",
        "workbench.extension.automation_report.open",
        "workbench.extension.automation_report.publish.invoke",
        "workbench.extension.automation_report.retry.edit",
        "workbench.extension.project_overview.open",
        "workbench.extension.project_overview.health.edit",
        "workbench.extension.particle_library.open",
        "workbench.extension.particle_library.compile.invoke",
        "workbench.extension.particle_library.duration.edit",
        "workbench.extension.ui_asset_editor.open",
        "workbench.extension.ui_asset_editor.validate.invoke",
        "workbench.extension.ui_asset_editor.theme.edit",
        "workbench.extension.ui_binding.open",
        "workbench.extension.ui_binding.validate.invoke",
        "workbench.extension.ui_binding.converter.edit",
        "workbench.extension.icon_library.open",
        "workbench.extension.icon_library.validate.invoke",
        "workbench.extension.icon_library.color_token.edit",
        "workbench.extension.accessibility_audit.open",
        "workbench.extension.accessibility_audit.audit_screen.invoke",
        "workbench.extension.accessibility_audit.rule_set.edit",
        "workbench.extension.menu_flow.open",
        "workbench.extension.menu_flow.validate_focus.invoke",
        "workbench.extension.menu_flow.transition.edit",
        "workbench.extension.font_atlas.open",
        "workbench.extension.font_atlas.bake_atlas.invoke",
        "workbench.extension.font_atlas.range.edit",
        "workbench.extension.console_diagnostics.open",
        "workbench.extension.console_diagnostics.filter_console.invoke",
        "workbench.extension.console_diagnostics.regex.edit",
        "workbench.generated_bottom.gameplay_effect_compile_log.select",
        "workbench.generated_bottom.render_pipeline_warnings.select",
        "workbench.generated_bottom.filter.edit",
        "workbench.module.assets.import.invoke",
        "workbench.module.vfx.curve_row.select",
        "component_lab.input_dropdown.open",
        "component_lab.icon_button.add",
        "component_lab.icon_button.open",
        "component_lab.icon_button.save",
        "component_lab.icon_button.delete",
        "component_lab.icon_button.show",
        "component_lab.icon_button.hide",
        "component_lab.icon_button.lock",
        "component_lab.icon_button.more",
        "component_lab.labs_tab_two.select",
    ] {
        assert!(
            is_workbench_preview_action(action_id),
            "{action_id} should stay in the shared preview-action registry"
        );
    }
    assert!(!is_workbench_preview_action("OpenProject"));
    let all_action_count = preview_action_ids().count();
    let unique_action_count = preview_action_ids()
        .collect::<std::collections::BTreeSet<_>>()
        .len();
    assert_eq!(WORKBENCH_PREVIEW_ACTION_ID_SET.len(), all_action_count);
    assert_eq!(
        unique_action_count, all_action_count,
        "preview action ids should remain unique"
    );
}
