use thiserror::Error;

#[derive(Clone, Debug, PartialEq, Error)]
pub enum ProtocolError {
    #[error("frame header is truncated: actual {actual} bytes, minimum {minimum}")]
    TruncatedHeader { actual: usize, minimum: usize },
    #[error("invalid frame magic {actual:02x?}")]
    InvalidMagic { actual: [u8; 4] },
    #[error("unsupported protocol version {actual}, expected {expected}")]
    UnsupportedVersion { actual: u16, expected: u16 },
    #[error("unknown message kind {0}")]
    UnknownMessageKind(u16),
    #[error("unknown command id {0}")]
    UnknownCommandId(u16),
    #[error("command {0} has no authoritative payload contract yet")]
    UnsupportedCommandPayload(u16),
    #[error("movement input violates its protocol contract: {0}")]
    InvalidMovementInput(String),
    #[error("offline session bootstrap violates its protocol contract: {0}")]
    InvalidOfflineBootstrap(String),
    #[error("command {command_id} payload has {actual} bytes, expected {expected}")]
    InvalidCommandPayloadLength {
        command_id: u16,
        actual: usize,
        expected: usize,
    },
    #[error("command {command_id} payload has {actual} bytes, expected {minimum}..={maximum}")]
    InvalidCommandPayloadLengthRange {
        command_id: u16,
        actual: usize,
        minimum: usize,
        maximum: usize,
    },
    #[error("{context} must be a nonzero entity id")]
    InvalidEntityId { context: &'static str },
    #[error("invalid lockpick ante {0}; expected 1, 2, or 3")]
    InvalidLockpickAnte(u8),
    #[error("invalid lockpick action code {0}")]
    InvalidLockpickAction(u8),
    #[error("invalid raid subgroup {0}; expected 1 or 2")]
    InvalidRaidSubgroup(u8),
    #[error("invalid master-loot threshold code {0}")]
    InvalidMasterLootThreshold(u8),
    #[error("invalid arena format code {0}")]
    InvalidArenaFormat(u8),
    #[error("invalid Vale Cup bracket code {0}")]
    InvalidValeCupBracket(u8),
    #[error("invalid Vale Cup nation code {0}")]
    InvalidValeCupNation(u8),
    #[error("invalid Vale Cup role code {0}")]
    InvalidValeCupRole(u8),
    #[error("invalid Vale Cup betting side code {0}")]
    InvalidValeCupSide(u8),
    #[error("invalid Dungeon Finder role code {0}")]
    InvalidDungeonFinderRole(u8),
    #[error("invalid Dungeon Finder listing tag code {0}")]
    InvalidDungeonFinderListingTag(u8),
    #[error("invalid Delve rite intensity code {0}")]
    InvalidDelveRiteIntensity(u8),
    #[error("invalid dungeon difficulty code {0}")]
    InvalidDungeonDifficulty(u8),
    #[error("invalid loot-roll choice code {0}")]
    InvalidLootRollChoice(u8),
    #[error("invalid equipment slot code {0}")]
    InvalidEquipmentSlot(u8),
    #[error("invalid overhead emote code {0}")]
    InvalidEmoteId(u8),
    #[error("invalid talent row level {0}; expected one of 5, 8, 11, 14, 17, or 20")]
    InvalidTalentRowLevel(u8),
    #[error("invalid talent option code {0}")]
    InvalidTalentOptionCode(u16),
    #[error("invalid talent spec code {0}")]
    InvalidTalentSpecCode(u16),
    #[error("invalid talent loadout index {0}; expected 0 through 9")]
    InvalidTalentLoadoutIndex(u32),
    #[error("invalid skin catalog code {0}")]
    InvalidSkinCatalog(u8),
    #[error("invalid class skin index {0}; expected 0 through 7")]
    InvalidClassSkinIndex(u8),
    #[error("invalid weapon-skin payload mode {0}; expected detach 0 or apply 1")]
    InvalidWeaponSkinMode(u8),
    #[error("invalid weapon-skin type code {0}; expected 1 through 8")]
    InvalidWeaponSkinType(u8),
    #[error("invalid corpse-harvest component count {0}; expected 0 through 3")]
    InvalidCorpseHarvestComponentCount(u8),
    #[error("invalid corpse-harvest component code {0}; expected 0 through 8")]
    InvalidCorpseHarvestComponentCode(u8),
    #[error("schema fingerprint mismatch")]
    SchemaMismatch { actual: [u8; 32] },
    #[error("payload length {actual} does not match declared length {declared}")]
    LengthMismatch { declared: usize, actual: usize },
    #[error("payload length {actual} exceeds limit {maximum}")]
    PayloadTooLarge { actual: usize, maximum: usize },
    #[error("field {field} contains non-finite value {value}")]
    NonFinite { field: &'static str, value: f64 },
    #[error("canonical key set contains a duplicate")]
    DuplicateCanonicalKey,
    #[error("command value contains an unknown tag {0}")]
    UnknownCommandValueTag(u8),
    #[error("command value object repeats key {key:?}")]
    DuplicateCommandObjectKey { key: String },
    #[error("{context} is truncated: needs {needed} bytes, only {remaining} remain")]
    TruncatedPayload {
        context: &'static str,
        needed: usize,
        remaining: usize,
    },
    #[error("{context} contains {actual} items/bytes, limit {maximum}")]
    CollectionTooLarge {
        context: &'static str,
        actual: usize,
        maximum: usize,
    },
    #[error("invalid boolean byte {0}")]
    InvalidBoolean(u8),
    #[error("{context} is not valid UTF-8")]
    InvalidUtf8 { context: &'static str },
    #[error("{context} offsets do not partition the payload")]
    InvalidOffsets { context: &'static str },
    #[error("typed payload has {remaining} trailing bytes")]
    TrailingPayload { remaining: usize },
}
