// Generated from 5ef9f7cb21cd8875b6d2c49701015dfcd78de35a; do not edit by hand.
pub const WEAPON_SKIN_COUNT: usize = 29;
pub const WEAPON_SKIN_TYPE_COUNT: usize = 8;
pub const WEAPON_SKIN_TYPE_BY_CODE: [u8; WEAPON_SKIN_COUNT] = [
    1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 8, 1, 2, 3, 4, 5, 6, 7, 1, 2, 3, 4, 5, 6, 7, 8,
];

pub fn weapon_skin_code_matches_loadout_type(skin_code: u8, loadout_type_index: usize) -> bool {
    skin_code != 0
        && usize::from(skin_code) <= WEAPON_SKIN_COUNT
        && loadout_type_index < WEAPON_SKIN_TYPE_COUNT
        && WEAPON_SKIN_TYPE_BY_CODE[usize::from(skin_code) - 1]
            == u8::try_from(loadout_type_index + 1).expect("weapon skin type count fits u8")
}
