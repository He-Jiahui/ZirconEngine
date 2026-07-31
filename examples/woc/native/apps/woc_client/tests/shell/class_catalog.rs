use std::collections::BTreeSet;

use woc_client::{
    offline_class_appearance, offline_class_presentation, offline_class_preview, ClassRoleType,
    OfflinePlayerClass, OFFLINE_CLASS_APPEARANCES, OFFLINE_CLASS_PRESENTATIONS,
};

#[test]
fn presentation_catalog_covers_the_nine_classes_in_picker_order() {
    assert_eq!(OFFLINE_CLASS_PRESENTATIONS.len(), 9);
    assert_eq!(
        OFFLINE_CLASS_PRESENTATIONS.map(|entry| entry.player_class),
        OfflinePlayerClass::ALL
    );
}

#[test]
fn class_roles_armor_weapons_colors_and_visual_keys_match_the_target() {
    let actual = OFFLINE_CLASS_PRESENTATIONS.map(|entry| {
        (
            entry.player_class.as_str(),
            entry.role_type,
            entry.role_key,
            entry.armor_key,
            entry.weapons_key,
            entry.color_rgb,
            entry.visual_key,
            entry.skin_count,
        )
    });
    assert_eq!(
        actual,
        [
            (
                "warrior",
                ClassRoleType::Hybrid,
                "classDetails.roles.warrior",
                "classDetails.armor.chainLeatherCloth",
                "classDetails.weapons.swordsMacesAxes",
                0xc79c6e,
                "player_warrior",
                4
            ),
            (
                "paladin",
                ClassRoleType::Hybrid,
                "classDetails.roles.paladin",
                "classDetails.armor.chainLeatherCloth",
                "classDetails.weapons.swordsMaces",
                0xf58cba,
                "player_paladin",
                2
            ),
            (
                "hunter",
                ClassRoleType::Ranged,
                "classDetails.roles.hunter",
                "classDetails.armor.leatherCloth",
                "classDetails.weapons.axesSwords",
                0xabd473,
                "player_hunter",
                4
            ),
            (
                "rogue",
                ClassRoleType::Damage,
                "classDetails.roles.rogue",
                "classDetails.armor.leatherCloth",
                "classDetails.weapons.daggersSwords",
                0xfff569,
                "player_rogue",
                4
            ),
            (
                "priest",
                ClassRoleType::Healer,
                "classDetails.roles.priest",
                "classDetails.armor.cloth",
                "classDetails.weapons.staves",
                0xfffff0,
                "player_priest",
                4
            ),
            (
                "shaman",
                ClassRoleType::Hybrid,
                "classDetails.roles.shaman",
                "classDetails.armor.chainLeatherCloth",
                "classDetails.weapons.macesAxes",
                0x0070de,
                "player_shaman",
                4
            ),
            (
                "mage",
                ClassRoleType::Ranged,
                "classDetails.roles.mage",
                "classDetails.armor.cloth",
                "classDetails.weapons.staves",
                0x69ccf0,
                "player_mage",
                4
            ),
            (
                "warlock",
                ClassRoleType::Ranged,
                "classDetails.roles.warlock",
                "classDetails.armor.cloth",
                "classDetails.weapons.staves",
                0x9482c9,
                "player_warlock",
                4
            ),
            (
                "druid",
                ClassRoleType::Hybrid,
                "classDetails.roles.druid",
                "classDetails.armor.leatherCloth",
                "classDetails.weapons.staves",
                0xff7d0a,
                "player_druid",
                4
            ),
        ]
    );
}

#[test]
fn every_class_has_three_distinct_target_signature_abilities() {
    let expected = [
        (
            OfflinePlayerClass::Warrior,
            ["charge", "heroic_strike", "execute"],
        ),
        (
            OfflinePlayerClass::Paladin,
            ["holy_light", "judgement", "seal_of_righteousness"],
        ),
        (
            OfflinePlayerClass::Hunter,
            ["serpent_sting", "aimed_shot", "arcane_shot"],
        ),
        (
            OfflinePlayerClass::Rogue,
            ["sinister_strike", "eviscerate", "evasion"],
        ),
        (
            OfflinePlayerClass::Priest,
            ["smite", "power_word_shield", "shadow_word_pain"],
        ),
        (
            OfflinePlayerClass::Shaman,
            ["lightning_bolt", "rockbiter_weapon", "ghost_wolf"],
        ),
        (
            OfflinePlayerClass::Mage,
            ["fireball", "frostbolt", "polymorph"],
        ),
        (
            OfflinePlayerClass::Warlock,
            ["shadow_bolt", "corruption", "life_tap"],
        ),
        (
            OfflinePlayerClass::Druid,
            ["wrath", "bear_form", "rejuvenation"],
        ),
    ];

    for (player_class, abilities) in expected {
        let entry = offline_class_presentation(player_class);
        assert_eq!(entry.signature_abilities, abilities);
        assert_eq!(abilities.into_iter().collect::<BTreeSet<_>>().len(), 3);
    }
}

#[test]
fn class_lookup_returns_the_catalog_identity_without_allocation() {
    for (index, player_class) in OfflinePlayerClass::ALL.into_iter().enumerate() {
        assert!(std::ptr::eq(
            offline_class_presentation(player_class),
            &OFFLINE_CLASS_PRESENTATIONS[index]
        ));
    }
}

#[test]
fn appearance_catalog_maps_nine_classes_to_the_seven_pinned_player_models() {
    let actual = OFFLINE_CLASS_APPEARANCES.map(|entry| {
        (
            entry.player_class,
            entry.model_asset,
            entry.skin_thumbnail_assets,
        )
    });
    assert_eq!(
        actual,
        [
            (
                OfflinePlayerClass::Warrior,
                "assets/m8/models/chars/players/knight.glb",
                &[
                    "assets/m8/textures/skins/knight/base.png",
                    "assets/m8/textures/skins/knight/alt_a.png",
                    "assets/m8/textures/skins/knight/alt_b.png",
                    "assets/m8/textures/skins/knight/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Paladin,
                "assets/m8/models/chars/players/paladin.glb",
                &[
                    "assets/m8/textures/skins/paladin/base.png",
                    "assets/m8/textures/skins/paladin/alt_a.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Hunter,
                "assets/m8/models/chars/players/ranger.glb",
                &[
                    "assets/m8/textures/skins/ranger/base.png",
                    "assets/m8/textures/skins/ranger/alt_a.png",
                    "assets/m8/textures/skins/ranger/alt_b.png",
                    "assets/m8/textures/skins/ranger/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Rogue,
                "assets/m8/models/chars/players/rogue.glb",
                &[
                    "assets/m8/textures/skins/rogue/base.png",
                    "assets/m8/textures/skins/rogue/alt_a.png",
                    "assets/m8/textures/skins/rogue/alt_b.png",
                    "assets/m8/textures/skins/rogue/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Priest,
                "assets/m8/models/chars/players/mage.glb",
                &[
                    "assets/m8/textures/skins/mage/base.png",
                    "assets/m8/textures/skins/mage/alt_a.png",
                    "assets/m8/textures/skins/mage/alt_b.png",
                    "assets/m8/textures/skins/mage/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Shaman,
                "assets/m8/models/chars/players/barbarian.glb",
                &[
                    "assets/m8/textures/skins/barbarian/base.png",
                    "assets/m8/textures/skins/barbarian/alt_a.png",
                    "assets/m8/textures/skins/barbarian/alt_b.png",
                    "assets/m8/textures/skins/barbarian/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Mage,
                "assets/m8/models/chars/players/mage.glb",
                &[
                    "assets/m8/textures/skins/mage/base.png",
                    "assets/m8/textures/skins/mage/alt_a.png",
                    "assets/m8/textures/skins/mage/alt_b.png",
                    "assets/m8/textures/skins/mage/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Warlock,
                "assets/m8/models/chars/players/mage.glb",
                &[
                    "assets/m8/textures/skins/mage/base.png",
                    "assets/m8/textures/skins/mage/alt_a.png",
                    "assets/m8/textures/skins/mage/alt_b.png",
                    "assets/m8/textures/skins/mage/alt_c.png",
                ][..],
            ),
            (
                OfflinePlayerClass::Druid,
                "assets/m8/models/chars/players/druid.glb",
                &[
                    "assets/m8/textures/skins/druid/base.png",
                    "assets/m8/textures/skins/druid/alt_a.png",
                    "assets/m8/textures/skins/druid/alt_b.png",
                    "assets/m8/textures/skins/druid/alt_c.png",
                ][..],
            ),
        ]
    );
}

#[test]
fn appearance_skin_counts_match_the_class_presentation_catalog() {
    for player_class in OfflinePlayerClass::ALL {
        let appearance = offline_class_appearance(player_class);
        let presentation = offline_class_presentation(player_class);

        assert_eq!(appearance.player_class, player_class);
        assert_eq!(
            appearance.skin_thumbnail_assets.len(),
            usize::from(presentation.skin_count)
        );
    }
}

#[test]
fn appearance_lookup_returns_the_catalog_identity_without_allocation() {
    for (index, player_class) in OfflinePlayerClass::ALL.into_iter().enumerate() {
        assert!(std::ptr::eq(
            offline_class_appearance(player_class),
            &OFFLINE_CLASS_APPEARANCES[index]
        ));
    }
}

#[test]
fn preview_requests_pair_each_valid_skin_with_its_model_and_texture() {
    for player_class in OfflinePlayerClass::ALL {
        let presentation = offline_class_presentation(player_class);
        let appearance = offline_class_appearance(player_class);
        for skin_variant in 0..presentation.skin_count {
            let preview = offline_class_preview(player_class, skin_variant)
                .expect("catalog skin must resolve a preview request");
            assert_eq!(preview.player_class, player_class);
            assert_eq!(preview.visual_key, presentation.visual_key);
            assert_eq!(preview.color_rgb, presentation.color_rgb);
            assert_eq!(preview.model_asset, appearance.model_asset);
            assert_eq!(preview.skin_variant, skin_variant);
            assert_eq!(
                preview.skin_thumbnail_asset,
                appearance.skin_thumbnail_assets[usize::from(skin_variant)]
            );
            assert_eq!(
                preview.skin_material_asset,
                if skin_variant == 0 {
                    None
                } else {
                    Some(appearance.skin_thumbnail_assets[usize::from(skin_variant)])
                }
            );
        }
        assert!(offline_class_preview(player_class, presentation.skin_count).is_none());
    }
}
