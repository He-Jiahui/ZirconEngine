use super::OfflinePlayerClass;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ClassRoleType {
    Tank,
    Damage,
    Ranged,
    Healer,
    Hybrid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfflineClassPresentation {
    pub player_class: OfflinePlayerClass,
    pub role_key: &'static str,
    pub role_type: ClassRoleType,
    pub armor_key: &'static str,
    pub weapons_key: &'static str,
    pub signature_abilities: [&'static str; 3],
    pub color_rgb: u32,
    pub visual_key: &'static str,
    pub skin_count: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfflineClassAppearance {
    pub player_class: OfflinePlayerClass,
    pub model_asset: &'static str,
    pub skin_thumbnail_assets: &'static [&'static str],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfflineClassPreview {
    pub player_class: OfflinePlayerClass,
    pub visual_key: &'static str,
    pub color_rgb: u32,
    pub model_asset: &'static str,
    pub skin_variant: u16,
    pub skin_thumbnail_asset: &'static str,
    pub skin_material_asset: Option<&'static str>,
}

const fn presentation(
    player_class: OfflinePlayerClass,
    role_key: &'static str,
    role_type: ClassRoleType,
    armor_key: &'static str,
    weapons_key: &'static str,
    signature_abilities: [&'static str; 3],
    color_rgb: u32,
    visual_key: &'static str,
    skin_count: u16,
) -> OfflineClassPresentation {
    OfflineClassPresentation {
        player_class,
        role_key,
        role_type,
        armor_key,
        weapons_key,
        signature_abilities,
        color_rgb,
        visual_key,
        skin_count,
    }
}

pub static OFFLINE_CLASS_PRESENTATIONS: [OfflineClassPresentation; 9] = [
    presentation(
        OfflinePlayerClass::Warrior,
        "classDetails.roles.warrior",
        ClassRoleType::Hybrid,
        "classDetails.armor.chainLeatherCloth",
        "classDetails.weapons.swordsMacesAxes",
        ["charge", "heroic_strike", "execute"],
        0xc79c6e,
        "player_warrior",
        4,
    ),
    presentation(
        OfflinePlayerClass::Paladin,
        "classDetails.roles.paladin",
        ClassRoleType::Hybrid,
        "classDetails.armor.chainLeatherCloth",
        "classDetails.weapons.swordsMaces",
        ["holy_light", "judgement", "seal_of_righteousness"],
        0xf58cba,
        "player_paladin",
        2,
    ),
    presentation(
        OfflinePlayerClass::Hunter,
        "classDetails.roles.hunter",
        ClassRoleType::Ranged,
        "classDetails.armor.leatherCloth",
        "classDetails.weapons.axesSwords",
        ["serpent_sting", "aimed_shot", "arcane_shot"],
        0xabd473,
        "player_hunter",
        4,
    ),
    presentation(
        OfflinePlayerClass::Rogue,
        "classDetails.roles.rogue",
        ClassRoleType::Damage,
        "classDetails.armor.leatherCloth",
        "classDetails.weapons.daggersSwords",
        ["sinister_strike", "eviscerate", "evasion"],
        0xfff569,
        "player_rogue",
        4,
    ),
    presentation(
        OfflinePlayerClass::Priest,
        "classDetails.roles.priest",
        ClassRoleType::Healer,
        "classDetails.armor.cloth",
        "classDetails.weapons.staves",
        ["smite", "power_word_shield", "shadow_word_pain"],
        0xfffff0,
        "player_priest",
        4,
    ),
    presentation(
        OfflinePlayerClass::Shaman,
        "classDetails.roles.shaman",
        ClassRoleType::Hybrid,
        "classDetails.armor.chainLeatherCloth",
        "classDetails.weapons.macesAxes",
        ["lightning_bolt", "rockbiter_weapon", "ghost_wolf"],
        0x0070de,
        "player_shaman",
        4,
    ),
    presentation(
        OfflinePlayerClass::Mage,
        "classDetails.roles.mage",
        ClassRoleType::Ranged,
        "classDetails.armor.cloth",
        "classDetails.weapons.staves",
        ["fireball", "frostbolt", "polymorph"],
        0x69ccf0,
        "player_mage",
        4,
    ),
    presentation(
        OfflinePlayerClass::Warlock,
        "classDetails.roles.warlock",
        ClassRoleType::Ranged,
        "classDetails.armor.cloth",
        "classDetails.weapons.staves",
        ["shadow_bolt", "corruption", "life_tap"],
        0x9482c9,
        "player_warlock",
        4,
    ),
    presentation(
        OfflinePlayerClass::Druid,
        "classDetails.roles.druid",
        ClassRoleType::Hybrid,
        "classDetails.armor.leatherCloth",
        "classDetails.weapons.staves",
        ["wrath", "bear_form", "rejuvenation"],
        0xff7d0a,
        "player_druid",
        4,
    ),
];

const KNIGHT_SKINS: &[&str] = &[
    "assets/m8/textures/skins/knight/base.png",
    "assets/m8/textures/skins/knight/alt_a.png",
    "assets/m8/textures/skins/knight/alt_b.png",
    "assets/m8/textures/skins/knight/alt_c.png",
];
const PALADIN_SKINS: &[&str] = &[
    "assets/m8/textures/skins/paladin/base.png",
    "assets/m8/textures/skins/paladin/alt_a.png",
];
const RANGER_SKINS: &[&str] = &[
    "assets/m8/textures/skins/ranger/base.png",
    "assets/m8/textures/skins/ranger/alt_a.png",
    "assets/m8/textures/skins/ranger/alt_b.png",
    "assets/m8/textures/skins/ranger/alt_c.png",
];
const ROGUE_SKINS: &[&str] = &[
    "assets/m8/textures/skins/rogue/base.png",
    "assets/m8/textures/skins/rogue/alt_a.png",
    "assets/m8/textures/skins/rogue/alt_b.png",
    "assets/m8/textures/skins/rogue/alt_c.png",
];
const MAGE_SKINS: &[&str] = &[
    "assets/m8/textures/skins/mage/base.png",
    "assets/m8/textures/skins/mage/alt_a.png",
    "assets/m8/textures/skins/mage/alt_b.png",
    "assets/m8/textures/skins/mage/alt_c.png",
];
const BARBARIAN_SKINS: &[&str] = &[
    "assets/m8/textures/skins/barbarian/base.png",
    "assets/m8/textures/skins/barbarian/alt_a.png",
    "assets/m8/textures/skins/barbarian/alt_b.png",
    "assets/m8/textures/skins/barbarian/alt_c.png",
];
const DRUID_SKINS: &[&str] = &[
    "assets/m8/textures/skins/druid/base.png",
    "assets/m8/textures/skins/druid/alt_a.png",
    "assets/m8/textures/skins/druid/alt_b.png",
    "assets/m8/textures/skins/druid/alt_c.png",
];

const fn appearance(
    player_class: OfflinePlayerClass,
    model_asset: &'static str,
    skin_thumbnail_assets: &'static [&'static str],
) -> OfflineClassAppearance {
    OfflineClassAppearance {
        player_class,
        model_asset,
        skin_thumbnail_assets,
    }
}

pub static OFFLINE_CLASS_APPEARANCES: [OfflineClassAppearance; 9] = [
    appearance(
        OfflinePlayerClass::Warrior,
        "assets/m8/models/chars/players/knight.glb",
        KNIGHT_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Paladin,
        "assets/m8/models/chars/players/paladin.glb",
        PALADIN_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Hunter,
        "assets/m8/models/chars/players/ranger.glb",
        RANGER_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Rogue,
        "assets/m8/models/chars/players/rogue.glb",
        ROGUE_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Priest,
        "assets/m8/models/chars/players/mage.glb",
        MAGE_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Shaman,
        "assets/m8/models/chars/players/barbarian.glb",
        BARBARIAN_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Mage,
        "assets/m8/models/chars/players/mage.glb",
        MAGE_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Warlock,
        "assets/m8/models/chars/players/mage.glb",
        MAGE_SKINS,
    ),
    appearance(
        OfflinePlayerClass::Druid,
        "assets/m8/models/chars/players/druid.glb",
        DRUID_SKINS,
    ),
];

const fn class_index(player_class: OfflinePlayerClass) -> usize {
    match player_class {
        OfflinePlayerClass::Warrior => 0,
        OfflinePlayerClass::Paladin => 1,
        OfflinePlayerClass::Hunter => 2,
        OfflinePlayerClass::Rogue => 3,
        OfflinePlayerClass::Priest => 4,
        OfflinePlayerClass::Shaman => 5,
        OfflinePlayerClass::Mage => 6,
        OfflinePlayerClass::Warlock => 7,
        OfflinePlayerClass::Druid => 8,
    }
}

pub const fn offline_class_presentation(
    player_class: OfflinePlayerClass,
) -> &'static OfflineClassPresentation {
    &OFFLINE_CLASS_PRESENTATIONS[class_index(player_class)]
}

pub const fn offline_class_appearance(
    player_class: OfflinePlayerClass,
) -> &'static OfflineClassAppearance {
    &OFFLINE_CLASS_APPEARANCES[class_index(player_class)]
}

pub fn offline_class_preview(
    player_class: OfflinePlayerClass,
    skin_variant: u16,
) -> Option<OfflineClassPreview> {
    let presentation = offline_class_presentation(player_class);
    let appearance = offline_class_appearance(player_class);
    let skin_thumbnail_asset = appearance
        .skin_thumbnail_assets
        .get(usize::from(skin_variant))
        .copied()?;
    Some(OfflineClassPreview {
        player_class,
        visual_key: presentation.visual_key,
        color_rgb: presentation.color_rgb,
        model_asset: appearance.model_asset,
        skin_variant,
        skin_thumbnail_asset,
        skin_material_asset: (skin_variant != 0).then_some(skin_thumbnail_asset),
    })
}
