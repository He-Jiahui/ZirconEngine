import { createHash } from 'node:crypto';
import { existsSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptDirectory = dirname(fileURLToPath(import.meta.url));
const projectRoot = resolve(scriptDirectory, '..');
const sourcePath = join(projectRoot, 'contracts', 'command_payloads.json');
const commandCatalogPath = join(projectRoot, 'reference', 'current-head', 'command_catalog.json');
const sourcePayloadCatalogPath = join(projectRoot, 'reference', 'current-head', 'command_payload_catalog.json');
const zrOutput = join(projectRoot, 'scripts', 'woc_game', 'src', 'protocol', 'command_payloads.zr');
const rustOutput = join(
  projectRoot,
  'native',
  'crates',
  'woc_protocol',
  'src',
  'generated_command_payloads.rs',
);
const checkOnly = process.argv.includes('--check');
const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const SOCIAL_COMMANDS = Object.freeze([
  ['guild_event_create', 'guildEventCreate', 'guild-event-create'],
  ['friend_add', 'friendAdd', 'friend-add'],
  ['friend_remove', 'friendRemove', 'friend-remove'],
  ['block_add', 'blockAdd', 'block-add'],
  ['block_remove', 'blockRemove', 'block-remove'],
  ['guild_create', 'guildCreate', 'guild-create'],
  ['guild_invite', 'guildInvite', 'guild-invite'],
  ['guild_accept', 'guildAccept', 'guild-accept'],
  ['guild_decline', 'guildDecline', 'guild-decline'],
  ['guild_leave', 'guildLeave', 'guild-leave'],
  ['guild_kick', 'guildKick', 'guild-kick'],
  ['guild_promote', 'guildPromote', 'guild-promote'],
  ['guild_demote', 'guildDemote', 'guild-demote'],
  ['guild_transfer', 'guildTransfer', 'guild-transfer'],
  ['guild_disband', 'guildDisband', 'guild-disband'],
  ['guild_event_remove', 'guildEventRemove', 'guild-event-remove'],
  ['ignore_add', 'ignoreAdd', 'ignore-add'],
  ['ignore_remove', 'ignoreRemove', 'ignore-remove'],
]);
const PARTY_EXTENDED_COMMANDS = Object.freeze([
  ['setLootMaster', 'partySetLootMaster', 'party-loot-master'],
  ['masterAssign', 'masterAssign', 'master-loot-assignment'],
  ['setMarker', 'partySetMarker', 'party-set-marker'],
  ['clearMarker', 'partyClearMarker', 'party-clear-marker'],
  ['readyrespond', 'partyReadyRespond', 'party-ready-respond'],
]);
const DUEL_ARENA_COMMANDS = Object.freeze([
  ['duel_req', 'duelRequest', 'duel-request', 'DUEL_REQUEST_COMMAND_ID'],
  ['duel_accept', 'duelAccept', 'duel-accept', 'DUEL_ACCEPT_COMMAND_ID'],
  ['duel_decline', 'duelDecline', 'duel-decline', 'DUEL_DECLINE_COMMAND_ID'],
  ['arena_queue', 'arenaQueue', 'arena-queue', 'ARENA_QUEUE_COMMAND_ID'],
  ['arena_leave', 'arenaLeave', 'arena-leave', 'ARENA_LEAVE_COMMAND_ID'],
  ['arena_augment', 'arenaAugment', 'arena-augment', 'ARENA_AUGMENT_COMMAND_ID'],
]);
const TRADE_TRANSPORT_COMMANDS = Object.freeze([
  ['trade_req', 'tradeRequest', 'trade-request', 'TRADE_REQUEST_COMMAND_ID'],
  ['trade_offer', 'tradeOffer', 'trade-offer', 'TRADE_OFFER_COMMAND_ID'],
  ['trade_accept', 'tradeAccept', 'trade-accept', 'TRADE_ACCEPT_COMMAND_ID'],
  ['trade_confirm', 'tradeConfirm', 'trade-confirm', 'TRADE_CONFIRM_COMMAND_ID'],
  ['trade_cancel', 'tradeCancel', 'trade-cancel', 'TRADE_CANCEL_COMMAND_ID'],
]);
const VALE_CUP_COMMANDS = Object.freeze([
  ['vcup_queue', 'valeCupQueue', 'Vale Cup queue', 'VALE_CUP_QUEUE_COMMAND_ID'],
  ['vcup_leave', 'valeCupLeave', 'Vale Cup queue-leave', 'VALE_CUP_LEAVE_COMMAND_ID'],
  ['vcup_role', 'valeCupRole', 'Vale Cup role', 'VALE_CUP_ROLE_COMMAND_ID'],
  ['vcup_ready', 'valeCupReady', 'Vale Cup ready', 'VALE_CUP_READY_COMMAND_ID'],
  ['vcup_bet', 'valeCupBet', 'Vale Cup bet', 'VALE_CUP_BET_COMMAND_ID'],
  ['vcup_practice', 'valeCupPractice', 'Vale Cup practice', 'VALE_CUP_PRACTICE_COMMAND_ID'],
]);
const MAIL_ID_COMMANDS = Object.freeze([
  ['mail_take', 'mailTake', 'mail-take', 'MAIL_TAKE_COMMAND_ID'],
  ['mail_delete', 'mailDelete', 'mail-delete', 'MAIL_DELETE_COMMAND_ID'],
  ['mail_read', 'mailRead', 'mail-read', 'MAIL_READ_COMMAND_ID'],
]);
const MAIL_SEND_COMMAND = Object.freeze([
  'mail_send', 'mailSend', 'mail-send', 'MAIL_SEND_COMMAND_ID',
]);
const BANK_COMMANDS = Object.freeze([
  ['bank_deposit', 'bankDeposit', 'bank-deposit', 'BANK_DEPOSIT_COMMAND_ID'],
  ['bank_withdraw', 'bankWithdraw', 'bank-withdraw', 'BANK_WITHDRAW_COMMAND_ID'],
  ['bank_buy_slots', 'bankBuySlots', 'bank-buy-slots', 'BANK_BUY_SLOTS_COMMAND_ID'],
]);
const DUNGEON_FINDER_COMMANDS = Object.freeze([
  ['df_roles', 'dungeonFinderRoles', 'Dungeon Finder roles', 'DUNGEON_FINDER_ROLES_COMMAND_ID'],
  ['df_queue', 'dungeonFinderQueue', 'Dungeon Finder queue', 'DUNGEON_FINDER_QUEUE_COMMAND_ID'],
  ['df_queue_leave', 'dungeonFinderQueueLeave', 'Dungeon Finder queue-leave', 'DUNGEON_FINDER_QUEUE_LEAVE_COMMAND_ID'],
  ['df_proposal', 'dungeonFinderProposal', 'Dungeon Finder proposal', 'DUNGEON_FINDER_PROPOSAL_COMMAND_ID'],
  ['df_list_create', 'dungeonFinderListingCreate', 'Dungeon Finder listing-create', 'DUNGEON_FINDER_LIST_CREATE_COMMAND_ID'],
  ['df_list_close', 'dungeonFinderListingClose', 'Dungeon Finder listing-close', 'DUNGEON_FINDER_LIST_CLOSE_COMMAND_ID'],
  ['df_apply', 'dungeonFinderApply', 'Dungeon Finder apply', 'DUNGEON_FINDER_APPLY_COMMAND_ID'],
  ['df_apply_cancel', 'dungeonFinderApplyCancel', 'Dungeon Finder apply-cancel', 'DUNGEON_FINDER_APPLY_CANCEL_COMMAND_ID'],
  ['df_app_respond', 'dungeonFinderApplicationRespond', 'Dungeon Finder application-response', 'DUNGEON_FINDER_APPLICATION_RESPONSE_COMMAND_ID'],
]);
const WORLD_OBJECT_COMMANDS = Object.freeze([
  ['loot', 'loot', 'loot', 'LOOT_COMMAND_ID'],
  ['pickup', 'pickup', 'pickup', 'PICKUP_COMMAND_ID'],
  ['autoloot', 'autoLoot', 'auto-loot', 'AUTO_LOOT_COMMAND_ID'],
  ['delve_interact', 'delveInteract', 'Delve interaction', 'DELVE_INTERACT_COMMAND_ID'],
  ['collect_delve_chest_loot', 'collectDelveChestLoot', 'Delve chest collection', 'COLLECT_DELVE_CHEST_LOOT_COMMAND_ID'],
]);
const EMPTY_ACTION_COMMANDS = Object.freeze([
  ['sell_all_junk', 'sellAllJunk', 'sell-all-junk', 'SELL_ALL_JUNK_COMMAND_ID'],
  ['market_collect', 'marketCollect', 'market collection', 'MARKET_COLLECT_COMMAND_ID'],
  ['leave_dungeon', 'leaveDungeon', 'dungeon leave', 'LEAVE_DUNGEON_COMMAND_ID'],
  ['leave_delve', 'leaveDelve', 'Delve leave', 'LEAVE_DELVE_COMMAND_ID'],
]);
const MARKET_ID_COMMANDS = Object.freeze([
  ['market_buy', 'marketBuy', 'market buy', 'MARKET_BUY_COMMAND_ID'],
  ['market_cancel', 'marketCancel', 'market cancel', 'MARKET_CANCEL_COMMAND_ID'],
]);
const DELVE_RITE_COMMANDS = Object.freeze([
  ['delve_rite_choose', 'delveRiteChoose', 'Delve rite choose', 'DELVE_RITE_CHOOSE_COMMAND_ID'],
]);
const DUNGEON_DIFFICULTY_COMMANDS = Object.freeze([
  ['set_dungeon_difficulty', 'setDungeonDifficulty', 'Dungeon difficulty', 'SET_DUNGEON_DIFFICULTY_COMMAND_ID'],
]);
const LOOT_ROLL_COMMANDS = Object.freeze([
  ['lootRoll', 'lootRoll', 'Loot roll', 'LOOT_ROLL_COMMAND_ID'],
]);
const EVENT_SKIN_COMMANDS = Object.freeze([
  ['claim_event_skin', 'claimEventSkin', 'Event-skin claim', 'CLAIM_EVENT_SKIN_COMMAND_ID'],
]);
const CLIENT_SEND_PAYLOADS = new Set([
  'challengeResponse',
  'castSlot',
  'castAt',
  'cast',
  'cancel_aura',
  'target',
  'tab',
  'tabFriendly',
  'targetNearestFriendly',
  'attack',
  'stopattack',
  'stow_weapon',
  'interact',
  'accept',
  'qlinkaccept',
  'turnin',
  'abandon',
  'equip',
  'inv_move',
  'unequip_item',
  'emote',
  'chat',
  'use',
  'discard',
  'buy',
  'sell',
  'buyback',
  'harvest_node',
  'harvestCorpse',
  'mail_send',
  'enter_dungeon',
  'enter_delve',
  'market_search',
  'market_list',
  'craft_item',
  'companion_upgrade',
  'deed_set_title',
  'set_town_focus',
  'change_skin',
  'unequip_mech_chroma',
  'change_weapon_skin',
  'release',
  'releaseEmpowered',
  'pet_abandon',
  'pet_rename',
  'pet_revive',
  'pet_attack',
  'pet_taunt',
  'pet_auto_taunt',
  'equip_bag',
  'unequip_bag',
  'lockpick_engage',
  'lockpick_action',
  'lockpick_abort',
  'applyTalents',
  'respec',
  'setSpec',
  'saveLoadout',
  'switchLoadout',
  'deleteLoadout',
  'selectTalentRow',
  'resurrect_corpse',
  'resurrect_healer',
  'resurrect_respond',
  'pet_water_jet',
  'pet_auto_water_jet',
  'pet_feed',
  'pet_heal',
  'pet_mode',
  'pinvite',
  'paccept',
  'pdecline',
  'pleave',
  'pkick',
  'ppromote',
  'praid',
  'punraid',
  'pmoveRaid',
  'masterAssign',
  'telemetry',
  'card_queue_join',
  'card_queue_leave',
  'play_card',
  'card_forfeit',
  ...SOCIAL_COMMANDS.map(([name]) => name),
  ...PARTY_EXTENDED_COMMANDS.map(([name]) => name),
  ...DUEL_ARENA_COMMANDS.map(([name]) => name),
  ...TRADE_TRANSPORT_COMMANDS.map(([name]) => name),
  ...VALE_CUP_COMMANDS.map(([name]) => name),
  ...MAIL_ID_COMMANDS.map(([name]) => name),
  ...BANK_COMMANDS.map(([name]) => name),
  ...DUNGEON_FINDER_COMMANDS.map(([name]) => name),
  ...WORLD_OBJECT_COMMANDS.map(([name]) => name),
  ...EMPTY_ACTION_COMMANDS.map(([name]) => name),
  ...MARKET_ID_COMMANDS.map(([name]) => name),
  ...DELVE_RITE_COMMANDS.map(([name]) => name),
  ...DUNGEON_DIFFICULTY_COMMANDS.map(([name]) => name),
  ...LOOT_ROLL_COMMANDS.map(([name]) => name),
  ...EVENT_SKIN_COMMANDS.map(([name]) => name),
]);

main();

function main() {
  const document = JSON.parse(readFileSync(sourcePath, 'utf8'));
  const catalog = JSON.parse(readFileSync(commandCatalogPath, 'utf8'));
  const sourcePayloadCatalog = JSON.parse(readFileSync(sourcePayloadCatalogPath, 'utf8'));
  invariant(document.schema_version === 60, 'command payload schema must be 60');
  invariant(document.source_commit === SOURCE_COMMIT, 'command payload source commit drifted');
  invariant(catalog.source_commit === SOURCE_COMMIT, 'command catalog source commit drifted');
  invariant(sourcePayloadCatalog.source_commit === SOURCE_COMMIT, 'source payload catalog commit drifted');
  invariant(Array.isArray(document.entries), 'command payload entries must be an array');

  const catalogFingerprintSource = catalog.entries
    .map((entry) => `${entry.index}\0${entry.name}\0${entry.kind}\0${entry.facet ?? ''}\n`)
    .join('');
  const catalogSha = createHash('sha256')
    .update(catalogFingerprintSource, 'utf8')
    .digest('hex');
  invariant(
    document.command_catalog_sha256 === catalogSha,
    'command payload schema targets a different command catalog',
  );
  invariant(
    sourcePayloadCatalog.command_catalog_sha256 === catalogSha,
    'source payload catalog targets a different command catalog',
  );
  const sourceEntries = new Map(sourcePayloadCatalog.entries.map((entry) => [entry.id, entry]));

  const ids = new Set();
  const names = new Set();
  for (const entry of document.entries) {
    invariant(Number.isInteger(entry.id) && entry.id >= 0, 'command payload id is invalid');
    invariant(!ids.has(entry.id), `duplicate command payload id ${entry.id}`);
    invariant(!names.has(entry.name), `duplicate command payload name ${entry.name}`);
    ids.add(entry.id);
    names.add(entry.name);
    const command = catalog.entries[entry.id];
    invariant(command?.index === entry.id, `unknown command payload id ${entry.id}`);
    invariant(command.name === entry.name, `command payload name drift for id ${entry.id}`);
    validateSourceShape(entry, command, sourceEntries.get(entry.id));
    if (CLIENT_SEND_PAYLOADS.has(entry.name)) {
      invariant(command.kind === 'client_send', `${entry.name} is not client-sendable`);
    }
    invariant(kindCode(entry.kind) !== 0, `invalid payload kind ${entry.kind}`);
    invariant(
      Number.isInteger(entry.min_byte_length) && entry.min_byte_length >= 0,
      'invalid minimum byte length',
    );
    invariant(
      Number.isInteger(entry.max_byte_length) &&
        entry.max_byte_length >= entry.min_byte_length,
      'invalid maximum byte length',
    );
    if (entry.kind === 'empty') {
      invariant(
        entry.min_byte_length === 0 &&
          entry.max_byte_length === 0 &&
          entry.encoding === 'empty',
        `${entry.name} is not canonical empty`,
      );
    } else if (entry.kind === 'target_entity') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'u64_le_zero_clears',
        `${entry.name} is not canonical target_entity`,
      );
    } else if (entry.kind === 'target_entity_raid_group') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 9 &&
          entry.encoding === 'u64_le_target+u8_raid_group_1_or_2',
        `${entry.name} is not canonical target_entity_raid_group`,
      );
    } else if (entry.kind === 'slot_index' || entry.kind === 'i32_value') {
        invariant(
          entry.min_byte_length === 4 &&
            entry.max_byte_length === 4 &&
            entry.encoding === 'i32_le',
          `${entry.name} is not canonical ${entry.kind}`,
      );
    } else if (entry.kind === 'i32_pair') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'i32_le_from+i32_le_to',
        `${entry.name} is not canonical i32_pair`,
      );
    } else if (entry.kind === 'emote_id') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_emote_id',
        `${entry.name} is not canonical emote_id`,
      );
    } else if (entry.kind === 'chat_text') {
      invariant(
        entry.min_byte_length === 4 &&
          entry.max_byte_length === 1024 &&
          entry.max_utf8_bytes === 1020 &&
          entry.max_utf16_code_units === 255 &&
          entry.encoding === 'u32_le_utf8_max_255_utf16',
        `${entry.name} is not canonical chat_text`,
      );
    } else if (entry.kind === 'save_loadout') {
      invariant(
        entry.min_byte_length === 6 &&
          entry.max_byte_length === 5858 &&
          entry.max_utf8_bytes === 256 &&
          entry.max_utf16_code_units === 24 &&
          entry.max_collection_entries === 22 &&
          entry.max_name_utf8_bytes === 96 &&
          entry.encoding ===
            'u32_le_utf8_name+u8_optional_talent_allocation+u8_bar_count+bar_optional_u32_le_utf8',
        `${entry.name} is not canonical save_loadout`,
      );
    } else if (entry.kind === 'u32_index') {
      invariant(
        entry.min_byte_length === 4 &&
          entry.max_byte_length === 4 &&
          entry.encoding === 'u32_le',
        `${entry.name} is not canonical u32_index`,
      );
    } else if (entry.kind === 'utf8_id_f64_pair') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 20 &&
          entry.max_byte_length === 20 + entry.max_utf8_bytes &&
          entry.encoding === 'u32_le_utf8+f64_le_x+f64_le_z',
        `${entry.name} is not canonical utf8_id_f64_pair`,
      );
    } else if (entry.kind === 'market_search') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 24 &&
          entry.max_byte_length === 24 + entry.max_utf8_bytes * 4 &&
          entry.encoding ===
            'u32_le_utf8_q+u32_le_utf8_item_type+u32_le_utf8_subtype+u32_le_utf8_rarity+f64_le_page',
        `${entry.name} is not canonical market_search`,
      );
    } else if (entry.kind === 'trade_offer') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          Number.isInteger(entry.max_collection_entries) &&
          entry.max_collection_entries > 0 &&
          entry.max_collection_entries <= 255 &&
          entry.min_byte_length === 9 &&
          entry.max_byte_length ===
            9 + entry.max_collection_entries * (12 + entry.max_utf8_bytes) &&
          entry.encoding === 'u8_count+repeated_u32_le_utf8_item_id+f64_le_count+f64_le_copper',
        `${entry.name} is not canonical trade_offer`,
      );
    } else if (entry.kind === 'challenge_response') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 12 &&
          entry.max_byte_length === 12 + entry.max_utf8_bytes * 3 &&
          entry.encoding === 'u32_le_utf8_nonce+u32_le_utf8_response+u32_le_utf8_signature',
        `${entry.name} is not canonical challenge_response`,
      );
    } else if (entry.kind === 'linked_quest_acceptance') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 12 &&
          entry.max_byte_length === 12 + entry.max_utf8_bytes &&
          entry.encoding === 'u32_le_utf8_quest_id+f64_le_sharer_pid',
        `${entry.name} is not canonical linked_quest_acceptance`,
      );
    } else if (entry.kind === 'equipment_item_optional_slot') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 5 &&
          entry.max_byte_length === 5 + entry.max_utf8_bytes &&
          entry.encoding === 'u32_le_utf8_item_id+u8_optional_equip_slot',
        `${entry.name} is not canonical equipment_item_optional_slot`,
      );
    } else if (entry.kind === 'equipment_slot') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_equip_slot',
        `${entry.name} is not canonical equipment_slot`,
      );
    } else if (entry.kind === 'telemetry_numeric_fields') {
      invariant(
        entry.min_byte_length === 6 &&
          entry.max_byte_length === 65536 &&
          entry.max_utf8_bytes === 256 &&
          entry.max_collection_entries === 256 &&
          entry.encoding ===
            'u32_le_utf8_kind+u16_le_field_count+repeated_u32_le_utf8_key_f64_le_value',
        `${entry.name} is not canonical telemetry_numeric_fields`,
      );
    } else if (entry.kind === 'town_focus_allocation') {
      invariant(
        entry.min_byte_length === 2 &&
          entry.max_byte_length === 65536 &&
          entry.max_utf8_bytes === 256 &&
          entry.max_collection_entries === 256 &&
          entry.encoding ===
            'u16_le_entry_count+repeated_u32_le_utf8_component+i32_le_points',
        `${entry.name} is not canonical town_focus_allocation`,
      );
    } else if (entry.kind === 'weapon_skin_change') {
      invariant(
        entry.min_byte_length === 2 &&
          entry.max_byte_length === 261 &&
          entry.max_utf8_bytes === 256 &&
          entry.encoding === 'u8_mode+(u32_le_utf8_skin|u8_weapon_type)',
        `${entry.name} is not canonical weapon_skin_change`,
      );
    } else if (entry.kind === 'corpse_harvest') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 12 &&
          entry.max_collection_entries === 3 &&
          entry.encoding === 'u64_le_target+u8_component_count_0_to_3+component_codes',
        `${entry.name} is not canonical corpse_harvest`,
      );
    } else if (entry.kind === 'utf8_id_pair') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 + entry.max_utf8_bytes * 2 &&
          entry.encoding === 'u32_le_utf8+u32_le_utf8',
        `${entry.name} is not canonical utf8_id_pair`,
      );
    } else if (entry.kind === 'utf8_id_optional_utf8_id') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 5 &&
          entry.max_byte_length === 9 + entry.max_utf8_bytes * 2 &&
          entry.encoding === 'u32_le_utf8+u8_presence+u32_le_utf8',
        `${entry.name} is not canonical utf8_id_optional_utf8_id`,
      );
    } else if (entry.kind === 'utf8_id_optional_target_entity') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 5 &&
          entry.max_byte_length === 13 + entry.max_utf8_bytes &&
          entry.encoding === 'u32_le_utf8+u8_presence+u64_le_target',
        `${entry.name} is not canonical utf8_id_optional_target_entity`,
      );
    } else if (entry.kind === 'lockpick_engage') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 9 &&
          entry.encoding === 'u64_le+u8_ante',
        `${entry.name} is not canonical lockpick_engage`,
      );
    } else if (entry.kind === 'lockpick_action') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 2 &&
          entry.max_byte_length === 6 + entry.max_utf8_bytes &&
          entry.encoding === 'u8_presence+u32_le_utf8+u8_action',
        `${entry.name} is not canonical lockpick_action`,
      );
    } else if (entry.kind === 'optional_utf8_id') {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) &&
          entry.max_utf8_bytes > 0 &&
          entry.min_byte_length === 1 &&
          entry.max_byte_length === 5 + entry.max_utf8_bytes &&
          entry.encoding === 'u8_presence+u32_le_utf8',
        `${entry.name} is not canonical optional_utf8_id`,
      );
    } else if (entry.kind === 'talent_row_selection') {
      invariant(
        entry.min_byte_length === 3 &&
          entry.max_byte_length === 3 &&
          entry.encoding === 'u8_row_level+u16_le_option_code',
        `${entry.name} is not canonical talent_row_selection`,
      );
    } else if (entry.kind === 'talent_spec') {
      invariant(
        entry.min_byte_length === 2 &&
          entry.max_byte_length === 2 &&
          entry.encoding === 'u16_le_spec_code',
        `${entry.name} is not canonical talent_spec`,
      );
    } else if (entry.kind === 'talent_allocation') {
      invariant(
        entry.min_byte_length === 14 &&
          entry.max_byte_length === 14 &&
          entry.encoding === 'u16_le_spec_code+6*u16_le_row_option_code',
        `${entry.name} is not canonical talent_allocation`,
      );
    } else if (entry.kind === 'cosmetic_skin') {
      invariant(
        entry.min_byte_length === 2 &&
          entry.max_byte_length === 2 &&
          entry.encoding === 'u8_catalog+u8_skin_index',
        `${entry.name} is not canonical cosmetic_skin`,
      );
    } else if (entry.kind === 'boolean') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_false_or_true',
        `${entry.name} is not canonical boolean`,
      );
    } else if (entry.kind === 'guild_event_create') {
      invariant(
        entry.min_byte_length === 13 &&
          entry.max_byte_length === 863 &&
          entry.max_utf8_bytes === 640 &&
          entry.max_day_utf8_bytes === 10 &&
          entry.max_title_utf8_bytes === 192 &&
          entry.max_note_utf8_bytes === 640 &&
          entry.encoding ===
            'u32_le_utf8_day+u8_presence+f64_le_hour+u32_le_utf8_title+u32_le_utf8_note',
        `${entry.name} is not canonical guild_event_create`,
      );
    } else if (entry.kind === 'party_loot_master') {
      invariant(
        entry.min_byte_length === 10 &&
          entry.max_byte_length === 10 &&
          entry.encoding === 'u8_enabled+f64_le_looter+u8_threshold',
        `${entry.name} is not canonical party_loot_master`,
      );
    } else if (entry.kind === 'master_loot_assignment') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 89 &&
          entry.encoding === 'f64_le_roll_id+u8_count_0_to_10+f64_le_target_pid',
        `${entry.name} is not canonical master_loot_assignment`,
      );
    } else if (entry.kind === 'party_marker') {
      invariant(
        entry.min_byte_length === 16 &&
          entry.max_byte_length === 16 &&
          entry.encoding === 'f64_le_entity_id+f64_le_marker_id',
        `${entry.name} is not canonical party_marker`,
      );
    } else if (entry.kind === 'party_marker_clear') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_entity_id',
        `${entry.name} is not canonical party_marker_clear`,
      );
    } else if (entry.kind === 'duel_request') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_target_id',
        `${entry.name} is not canonical duel_request`,
      );
    } else if (entry.kind === 'trade_request') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_target_id',
        `${entry.name} is not canonical trade_request`,
      );
    } else if (entry.kind === 'mail_send') {
      invariant(
        entry.min_byte_length === 21 &&
          entry.max_byte_length === 16 * 1024 &&
          entry.max_utf8_bytes === 16 * 1024 &&
          entry.max_collection_entries === 3 &&
          entry.encoding ===
            'u32_le_utf8_to+u32_le_utf8_subject+u32_le_utf8_body+f64_le_copper+u8_attachment_count+repeated_u32_le_utf8_item_id_f64_le_count',
        `${entry.name} is not canonical mail_send`,
      );
    } else if (entry.kind === 'mail_id') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_mail_id',
        `${entry.name} is not canonical mail_id`,
      );
    } else if (entry.kind === 'bank_slot_optional_count') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 17 &&
          entry.encoding === 'f64_le_slot+u8_presence+f64_le_count',
        `${entry.name} is not canonical bank_slot_optional_count`,
      );
    } else if (entry.kind === 'dungeon_finder_roles') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 4 &&
          entry.encoding === 'u8_count_0_to_3+u8_finder_role',
        `${entry.name} is not canonical dungeon_finder_roles`,
      );
    } else if (entry.kind === 'dungeon_finder_activities') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 3089 &&
          entry.max_utf8_bytes === 192 &&
          entry.max_utf16_code_units === 64 &&
          entry.encoding === 'u8_count_0_to_16+u8_utf8_activity_id_max_64_utf16',
        `${entry.name} is not canonical dungeon_finder_activities`,
      );
    } else if (entry.kind === 'dungeon_finder_listing') {
      invariant(
        entry.min_byte_length === 2 &&
          entry.max_byte_length === 202 &&
          entry.max_utf8_bytes === 192 &&
          entry.max_utf16_code_units === 64 &&
          entry.encoding ===
            'u8_utf8_activity_id_max_64_utf16+u8_count_0_to_8+u8_finder_listing_tag',
        `${entry.name} is not canonical dungeon_finder_listing`,
      );
    } else if (entry.kind === 'dungeon_finder_listing_id') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_listing_id',
        `${entry.name} is not canonical dungeon_finder_listing_id`,
      );
    } else if (entry.kind === 'dungeon_finder_application_response') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 9 &&
          entry.encoding === 'f64_le_applicant_id+u8_false_or_true',
        `${entry.name} is not canonical dungeon_finder_application_response`,
      );
    } else if (entry.kind === 'world_object_id') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_world_object_id',
        `${entry.name} is not canonical world_object_id`,
      );
    } else if (entry.kind === 'market_listing_id') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_market_listing_id',
        `${entry.name} is not canonical market_listing_id`,
      );
    } else if (entry.kind === 'delve_rite_intensity') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_rite_intensity_easy_medium_hard',
        `${entry.name} is not canonical delve_rite_intensity`,
      );
    } else if (entry.kind === 'dungeon_difficulty') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_dungeon_difficulty_normal_heroic',
        `${entry.name} is not canonical dungeon_difficulty`,
      );
    } else if (entry.kind === 'loot_roll') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 9 &&
          entry.encoding === 'f64_le_roll_id+u8_need_greed_pass',
        `${entry.name} is not canonical loot_roll`,
      );
    } else if (entry.kind === 'event_skin') {
      invariant(
        entry.min_byte_length === 8 &&
          entry.max_byte_length === 8 &&
          entry.encoding === 'f64_le_event_skin_id',
        `${entry.name} is not canonical event_skin`,
      );
    } else if (entry.kind === 'arena_queue_format') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_arena_format',
        `${entry.name} is not canonical arena_queue_format`,
      );
    } else if (entry.kind === 'arena_augment') {
      invariant(
        entry.min_byte_length === 4 &&
          entry.max_byte_length === 260 &&
          entry.max_utf8_bytes === 256 &&
          entry.max_utf16_code_units === 64 &&
          entry.encoding === 'u32_le_utf8',
        `${entry.name} is not canonical arena_augment`,
      );
    } else if (entry.kind === 'vale_cup_queue') {
      invariant(
        entry.min_byte_length === 4 &&
          entry.max_byte_length === 4 &&
          entry.encoding === 'u8_bracket+u8_nation+u8_role+u8_guild',
        `${entry.name} is not canonical vale_cup_queue`,
      );
    } else if (entry.kind === 'vale_cup_role') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_sport_role',
        `${entry.name} is not canonical vale_cup_role`,
      );
    } else if (entry.kind === 'vale_cup_bet') {
      invariant(
        entry.min_byte_length === 9 &&
          entry.max_byte_length === 9 &&
          entry.encoding === 'u8_side+f64_le_amount',
        `${entry.name} is not canonical vale_cup_bet`,
      );
    } else if (entry.kind === 'vale_cup_bracket') {
      invariant(
        entry.min_byte_length === 1 &&
          entry.max_byte_length === 1 &&
          entry.encoding === 'u8_vc_bracket',
        `${entry.name} is not canonical vale_cup_bracket`,
      );
    } else {
      invariant(
        Number.isInteger(entry.max_utf8_bytes) && entry.max_utf8_bytes > 0,
        `${entry.name} has no UTF-8 bound`,
      );
      const optionalBytes = entry.kind === 'utf8_id_optional_u32' ? 5 : 0;
      invariant(
        entry.min_byte_length === 4 + (optionalBytes > 0 ? 1 : 0) &&
          entry.max_byte_length === 4 + entry.max_utf8_bytes + optionalBytes,
        `${entry.name} has noncanonical UTF-8 bounds`,
      );
      invariant(
        entry.encoding ===
          (entry.kind === 'utf8_id'
            ? 'u32_le_utf8'
            : 'u32_le_utf8+u8_presence+u32_le'),
        `${entry.name} has noncanonical UTF-8 encoding`,
      );
    }
  }
  document.entries.sort((left, right) => left.id - right.id);
  const fingerprintSource = document.entries
    .map(
      (entry) =>
        `${entry.id}\0${entry.name}\0${entry.kind}\0${entry.min_byte_length}\0` +
        `${entry.max_byte_length}\0${entry.max_utf8_bytes ?? ''}\0` +
        `${entry.max_utf16_code_units ?? ''}\0${entry.max_collection_entries ?? ''}\0` +
        `${entry.encoding}\n`,
    )
    .join('');
  const sha256 = createHash('sha256').update(fingerprintSource, 'utf8').digest('hex');
  const outputs = new Map([
    [zrOutput, renderZr(document.entries, sha256)],
    [rustOutput, renderRust(document.entries, sha256)],
  ]);
  for (const [path, content] of outputs) writeOrCheck(path, content);
  process.stdout.write(
    `${checkOnly ? 'checked' : 'generated'} ${document.entries.length} WOC command payloads (${sha256.slice(0, 15)})\n`,
  );
}

function validateSourceShape(entry, command, sourceEntry) {
  const shape = entry.source_shape;
  invariant(shape && typeof shape === 'object', `${entry.name} has no source shape`);
  invariant(Array.isArray(shape.fields), `${entry.name} source fields are invalid`);
  invariant(
    shape.fields.every((field) => typeof field === 'string' || field === null),
    `${entry.name} source fields are invalid`,
  );
  invariant(sourceEntry?.name === entry.name, `${entry.name} source payload entry is missing`);
  if (shape.kind === 'dispatch_only') {
    invariant(command.kind === 'dispatch_only', `${entry.name} is not dispatch-only`);
    invariant(shape.method === null && shape.fields.length === 0, `${entry.name} dispatch shape is invalid`);
    invariant(sourceEntry.status === 'dispatch_only', `${entry.name} source dispatch status drifted`);
    return;
  }
  invariant(shape.kind === 'client_send', `${entry.name} source shape kind is invalid`);
  invariant(command.kind === 'client_send', `${entry.name} is not client-sendable`);
  invariant(typeof shape.method === 'string' && shape.method.length > 0, `${entry.name} source method is invalid`);
  const matchingSite = sourceEntry.client_sends.find(
    (site) =>
      site.method === shape.method &&
      JSON.stringify(site.fields.map((field) => field.name)) === JSON.stringify(shape.fields),
  );
  invariant(matchingSite, `${entry.name} source method or fields drifted`);
  const alternates = shape.alternates ?? [];
  invariant(Array.isArray(alternates), `${entry.name} source alternates are invalid`);
  for (const alternate of alternates) {
    invariant(
      alternate && typeof alternate === 'object' &&
        typeof alternate.method === 'string' && alternate.method.length > 0 &&
        Array.isArray(alternate.fields) &&
        alternate.fields.every((field) => typeof field === 'string' || field === null),
      `${entry.name} source alternate is invalid`,
    );
    const alternateSite = sourceEntry.client_sends.find(
      (site) =>
        site.method === alternate.method &&
        JSON.stringify(site.fields.map((field) => field.name)) === JSON.stringify(alternate.fields),
    );
    invariant(alternateSite, `${entry.name} source alternate method or fields drifted`);
  }
}

function renderZr(entries, sha256) {
  const byName = new Map(entries.map((entry) => [entry.name, entry]));
  const castSlot = required(byName, 'castSlot');
  const challengeResponse = required(byName, 'challengeResponse');
  const castAt = required(byName, 'castAt');
  const cast = required(byName, 'cast');
  const cancelAura = required(byName, 'cancel_aura');
  const changeSkin = required(byName, 'change_skin');
  const unequipMechChroma = required(byName, 'unequip_mech_chroma');
  const changeWeaponSkin = required(byName, 'change_weapon_skin');
  const harvestCorpse = required(byName, 'harvestCorpse');
  const enterDungeon = required(byName, 'enter_dungeon');
  const enterDelve = required(byName, 'enter_delve');
  const marketSearch = required(byName, 'market_search');
  const marketList = required(byName, 'market_list');
  const mailSend = required(byName, MAIL_SEND_COMMAND[0]);
  const target = required(byName, 'target');
  const tab = required(byName, 'tab');
  const targetNearest = required(byName, 'targetNearest');
  const tabFriendly = required(byName, 'tabFriendly');
  const targetNearestFriendly = required(byName, 'targetNearestFriendly');
  const attack = required(byName, 'attack');
  const stopAttack = required(byName, 'stopattack');
  const weaponStow = required(byName, 'stow_weapon');
  const interact = required(byName, 'interact');
  const acceptQuest = required(byName, 'accept');
  const linkedQuestAccept = required(byName, 'qlinkaccept');
  const turnInQuest = required(byName, 'turnin');
  const abandon = required(byName, 'abandon');
  const equipItem = required(byName, 'equip');
  const inventoryMove = required(byName, 'inv_move');
  const unequipItem = required(byName, 'unequip_item');
  const emote = required(byName, 'emote');
  const chat = required(byName, 'chat');
  const telemetry = required(byName, 'telemetry');
  const useItem = required(byName, 'use');
  const discardItem = required(byName, 'discard');
  const buy = required(byName, 'buy');
  const sell = required(byName, 'sell');
  const buyback = required(byName, 'buyback');
  const harvestNode = required(byName, 'harvest_node');
  const craftItem = required(byName, 'craft_item');
  const heroicBuy = required(byName, 'heroic_buy');
  const delveBuy = required(byName, 'delve_buy');
  const companionUpgrade = required(byName, 'companion_upgrade');
  const deedSetTitle = required(byName, 'deed_set_title');
  const setTownFocus = required(byName, 'set_town_focus');
  const release = required(byName, 'release');
  const releaseEmpowered = required(byName, 'releaseEmpowered');
  const petAbandon = required(byName, 'pet_abandon');
  const petRename = required(byName, 'pet_rename');
  const petRevive = required(byName, 'pet_revive');
  const petAttack = required(byName, 'pet_attack');
  const petTaunt = required(byName, 'pet_taunt');
  const petAutoTaunt = required(byName, 'pet_auto_taunt');
  const petWaterJet = required(byName, 'pet_water_jet');
  const petAutoWaterJet = required(byName, 'pet_auto_water_jet');
  const petFeed = required(byName, 'pet_feed');
  const petHeal = required(byName, 'pet_heal');
  const petMode = required(byName, 'pet_mode');
  const equipBag = required(byName, 'equip_bag');
  const unequipBag = required(byName, 'unequip_bag');
  const lockpickEngage = required(byName, 'lockpick_engage');
  const lockpickAction = required(byName, 'lockpick_action');
  const lockpickAbort = required(byName, 'lockpick_abort');
  const applyTalents = required(byName, 'applyTalents');
  const respec = required(byName, 'respec');
  const setSpec = required(byName, 'setSpec');
  const saveLoadout = required(byName, 'saveLoadout');
  const switchLoadout = required(byName, 'switchLoadout');
  const deleteLoadout = required(byName, 'deleteLoadout');
  const selectTalentRow = required(byName, 'selectTalentRow');
  const resurrectCorpse = required(byName, 'resurrect_corpse');
  const resurrectHealer = required(byName, 'resurrect_healer');
  const resurrectRespond = required(byName, 'resurrect_respond');
  const partyInvite = required(byName, 'pinvite');
  const partyAccept = required(byName, 'paccept');
  const partyDecline = required(byName, 'pdecline');
  const partyLeave = required(byName, 'pleave');
  const partyKick = required(byName, 'pkick');
  const partyPromote = required(byName, 'ppromote');
  const partyRaid = required(byName, 'praid');
  const partyUnraid = required(byName, 'punraid');
  const partyMoveRaid = required(byName, 'pmoveRaid');
  const cardQueueJoin = required(byName, 'card_queue_join');
  const cardQueueLeave = required(byName, 'card_queue_leave');
  const cardPlay = required(byName, 'play_card');
  const cardForfeit = required(byName, 'card_forfeit');
  const socialCommands = SOCIAL_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const socialCommandFunctions = socialCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const socialContractTest = socialCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${
          entry.min_byte_length === entry.max_byte_length ? entry.min_byte_length : -1
        }`,
    )
    .join(' &&\n');
  const partyExtendedCommands = PARTY_EXTENDED_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const partyExtendedCommandFunctions = partyExtendedCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const partyExtendedContractTest = partyExtendedCommands
    .map(({ entry, functionName }) => {
      const lengthCheck = entry.min_byte_length === entry.max_byte_length
        ? `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`
        : `payloadMinLength(<uint>${entry.id}, true) == ${entry.min_byte_length} && ` +
          `payloadMaxLength(<uint>${entry.id}, true) == ${entry.max_byte_length}`;
      return `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ${lengthCheck}`;
    })
    .join(' &&\n');
  const duelArenaCommands = DUEL_ARENA_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const duelArenaCommandFunctions = duelArenaCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const duelArenaContractTest = duelArenaCommands
    .map(({ entry, functionName }) => {
      const lengthCheck = entry.min_byte_length === entry.max_byte_length
        ? `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`
        : `payloadMinLength(<uint>${entry.id}, true) == ${entry.min_byte_length} && ` +
          `payloadMaxLength(<uint>${entry.id}, true) == ${entry.max_byte_length}`;
      return `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ${lengthCheck}`;
    })
    .join(' &&\n');
  const tradeTransportCommands = TRADE_TRANSPORT_COMMANDS.map(
    ([name, functionName, description]) => ({
      entry: required(byName, name),
      functionName,
      description,
    }),
  );
  const tradeTransportCommandFunctions = tradeTransportCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const tradeTransportContractTest = tradeTransportCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        (entry.min_byte_length === entry.max_byte_length
          ? `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`
          : `payloadMinLength(<uint>${entry.id}, true) == ${entry.min_byte_length} && ` +
            `payloadMaxLength(<uint>${entry.id}, true) == ${entry.max_byte_length}`),
    )
    .join(' &&\n');
  const valeCupCommands = VALE_CUP_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const valeCupCommandFunctions = valeCupCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const valeCupContractTest = valeCupCommands
    .map(({ entry, functionName }) =>
      `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
      `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
      `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const mailIdCommands = MAIL_ID_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const mailIdCommandFunctions = mailIdCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const mailIdContractTest = mailIdCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const bankCommands = BANK_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const bankCommandFunctions = bankCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const bankContractTest = bankCommands
    .map(({ entry, functionName }) => {
      const lengthCheck = entry.min_byte_length === entry.max_byte_length
        ? `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`
        : `payloadMinLength(<uint>${entry.id}, true) == ${entry.min_byte_length} && ` +
          `payloadMaxLength(<uint>${entry.id}, true) == ${entry.max_byte_length}`;
      return `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ${lengthCheck}`;
    })
    .join(' &&\n');
  const dungeonFinderCommands = DUNGEON_FINDER_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const dungeonFinderCommandFunctions = dungeonFinderCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const dungeonFinderContractTest = dungeonFinderCommands
    .map(({ entry, functionName }) => {
      const lengthCheck = entry.min_byte_length === entry.max_byte_length
        ? `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`
        : `payloadMinLength(<uint>${entry.id}, true) == ${entry.min_byte_length} && ` +
          `payloadMaxLength(<uint>${entry.id}, true) == ${entry.max_byte_length}`;
      return `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ${lengthCheck}`;
    })
    .join(' &&\n');
  const worldObjectCommands = WORLD_OBJECT_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const worldObjectCommandFunctions = worldObjectCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const worldObjectContractTest = worldObjectCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const emptyActionCommands = EMPTY_ACTION_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const emptyActionCommandFunctions = emptyActionCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const emptyActionContractTest = emptyActionCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const marketIdCommands = MARKET_ID_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const marketIdCommandFunctions = marketIdCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const marketIdContractTest = marketIdCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const delveRiteCommands = DELVE_RITE_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const delveRiteCommandFunctions = delveRiteCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const delveRiteContractTest = delveRiteCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const dungeonDifficultyCommands = DUNGEON_DIFFICULTY_COMMANDS.map(
    ([name, functionName, description]) => ({
      entry: required(byName, name),
      functionName,
      description,
    }),
  );
  const dungeonDifficultyCommandFunctions = dungeonDifficultyCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const dungeonDifficultyContractTest = dungeonDifficultyCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const lootRollCommands = LOOT_ROLL_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const lootRollCommandFunctions = lootRollCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const lootRollContractTest = lootRollCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const eventSkinCommands = EVENT_SKIN_COMMANDS.map(([name, functionName, description]) => ({
    entry: required(byName, name),
    functionName,
    description,
  }));
  const eventSkinCommandFunctions = eventSkinCommands
    .map(
      ({ entry, functionName, description }) =>
        `pub ${functionName}CommandId(required: bool): uint {\n` +
        `    if (!required) { throw "WOC ${description} command id is required"; }\n` +
        `    return <uint>${entry.id};\n}\n`,
    )
    .join('\n');
  const eventSkinContractTest = eventSkinCommands
    .map(
      ({ entry, functionName }) =>
        `        ${functionName}CommandId(true) == <uint>${entry.id} && ` +
        `payloadKind(<uint>${entry.id}, 1) == ${kindCode(entry.kind)} && ` +
        `payloadLength(<uint>${entry.id}, true) == ${entry.min_byte_length}`,
    )
    .join(' &&\n');
  const kindRows = entries
    .map((entry) => `    if (id == <uint>${entry.id}) { return ${kindCode(entry.kind)}; }`)
    .join('\n');
  const lengthRows = entries
    .map((entry) => {
      const length = entry.min_byte_length === entry.max_byte_length
        ? entry.min_byte_length
        : -1;
      return `    if (id == <uint>${entry.id}) { return ${length}; }`;
    })
    .join('\n');
  const minLengthRows = entries
    .map((entry) => `    if (id == <uint>${entry.id}) { return ${entry.min_byte_length}; }`)
    .join('\n');
  const maxLengthRows = entries
    .map((entry) => `    if (id == <uint>${entry.id}) { return ${entry.max_byte_length}; }`)
    .join('\n');
  return `// Generated by examples/woc/tools/command_payload_codegen.mjs. Do not edit.\n` +
    `pub fingerprintHex(required: bool): string {\n` +
    `    if (!required) { throw "WOC command payload fingerprint is required"; }\n` +
    `    return ${JSON.stringify(sha256)};\n}\n\n` +
    `pub castAtCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC ground-cast command id is required"; }\n` +
    `    return <uint>${castAt.id};\n}\n\n` +
    `pub castCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC cast command id is required"; }\n` +
    `    return <uint>${cast.id};\n}\n\n` +
    `pub cancelAuraCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC cancel-aura command id is required"; }\n` +
    `    return <uint>${cancelAura.id};\n}\n\n` +
    `pub changeSkinCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC change-skin command id is required"; }\n` +
    `    return <uint>${changeSkin.id};\n}\n\n` +
    `pub unequipMechChromaCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC unequip-mech-chroma command id is required"; }\n` +
    `    return <uint>${unequipMechChroma.id};\n}\n\n` +
    `pub changeWeaponSkinCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC change-weapon-skin command id is required"; }\n` +
    `    return <uint>${changeWeaponSkin.id};\n}\n\n` +
    `pub harvestCorpseCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC harvest-corpse command id is required"; }\n` +
    `    return <uint>${harvestCorpse.id};\n}\n\n` +
    `pub enterDungeonCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC enter-dungeon command id is required"; }\n` +
    `    return <uint>${enterDungeon.id};\n}\n\n` +
    `pub enterDelveCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC enter-delve command id is required"; }\n` +
    `    return <uint>${enterDelve.id};\n}\n\n` +
    `pub marketSearchCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC market-search command id is required"; }\n` +
    `    return <uint>${marketSearch.id};\n}\n\n` +
    `pub marketListCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC market-list command id is required"; }\n` +
    `    return <uint>${marketList.id};\n}\n\n` +
    `pub mailSendCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC mail-send command id is required"; }\n` +
    `    return <uint>${mailSend.id};\n}\n\n` +
    `pub challengeResponseCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC challenge-response command id is required"; }\n` +
    `    return <uint>${challengeResponse.id};\n}\n\n` +
    `pub castSlotCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC cast-slot command id is required"; }\n` +
    `    return <uint>${castSlot.id};\n}\n\n` +
    `pub targetCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC target command id is required"; }\n` +
    `    return <uint>${target.id};\n}\n\n` +
    `pub tabCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC tab-target command id is required"; }\n` +
    `    return <uint>${tab.id};\n}\n\n` +
    `pub targetNearestCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC nearest-enemy command id is required"; }\n` +
    `    return <uint>${targetNearest.id};\n}\n\n` +
    `pub tabFriendlyCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC friendly-tab command id is required"; }\n` +
    `    return <uint>${tabFriendly.id};\n}\n\n` +
    `pub targetNearestFriendlyCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC nearest-friendly command id is required"; }\n` +
    `    return <uint>${targetNearestFriendly.id};\n}\n\n` +
    `pub attackCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC attack command id is required"; }\n` +
    `    return <uint>${attack.id};\n}\n\n` +
    `pub stopAttackCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC stop-attack command id is required"; }\n` +
    `    return <uint>${stopAttack.id};\n}\n\n` +
    `pub weaponStowCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC weapon-stow command id is required"; }\n` +
    `    return <uint>${weaponStow.id};\n}\n\n` +
    `pub interactCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC interact command id is required"; }\n` +
    `    return <uint>${interact.id};\n}\n\n` +
    `pub acceptQuestCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC accept-quest command id is required"; }\n` +
    `    return <uint>${acceptQuest.id};\n}\n\n` +
    `pub turnInQuestCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC turn-in-quest command id is required"; }\n` +
    `    return <uint>${turnInQuest.id};\n}\n\n` +
    `pub abandonCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC abandon command id is required"; }\n` +
    `    return <uint>${abandon.id};\n}\n\n` +
    `pub equipItemCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC equip-item command id is required"; }\n` +
    `    return <uint>${equipItem.id};\n}\n\n` +
    `pub inventoryMoveCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC inventory-move command id is required"; }\n` +
    `    return <uint>${inventoryMove.id};\n}\n\n` +
    `pub unequipItemCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC unequip-item command id is required"; }\n` +
    `    return <uint>${unequipItem.id};\n}\n\n` +
    `pub emoteCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC emote command id is required"; }\n` +
    `    return <uint>${emote.id};\n}\n\n` +
    `pub chatCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC chat command id is required"; }\n` +
    `    return <uint>${chat.id};\n}\n\n` +
    `pub telemetryCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC telemetry command id is required"; }\n` +
    `    return <uint>${telemetry.id};\n}\n\n` +
    `pub useItemCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC use-item command id is required"; }\n` +
    `    return <uint>${useItem.id};\n}\n\n` +
    `pub discardItemCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC discard-item command id is required"; }\n` +
    `    return <uint>${discardItem.id};\n}\n\n` +
    `pub buyCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC vendor-buy command id is required"; }\n` +
    `    return <uint>${buy.id};\n}\n\n` +
    `pub sellCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC vendor-sell command id is required"; }\n` +
    `    return <uint>${sell.id};\n}\n\n` +
    `pub buybackCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC vendor-buyback command id is required"; }\n` +
    `    return <uint>${buyback.id};\n}\n\n` +
    `pub harvestNodeCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC harvest-node command id is required"; }\n` +
    `    return <uint>${harvestNode.id};\n}\n\n` +
    `pub craftItemCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC craft-item command id is required"; }\n` +
    `    return <uint>${craftItem.id};\n}\n\n` +
    `pub heroicBuyCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC heroic-buy command id is required"; }\n` +
    `    return <uint>${heroicBuy.id};\n}\n\n` +
    `pub delveBuyCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC Delve-buy command id is required"; }\n` +
    `    return <uint>${delveBuy.id};\n}\n\n` +
    `pub companionUpgradeCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC companion-upgrade command id is required"; }\n` +
    `    return <uint>${companionUpgrade.id};\n}\n\n` +
    `pub deedSetTitleCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC deed-set-title command id is required"; }\n` +
    `    return <uint>${deedSetTitle.id};\n}\n\n` +
    `pub setTownFocusCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC town-focus command id is required"; }\n` +
    `    return <uint>${setTownFocus.id};\n}\n\n` +
    `pub linkedQuestAcceptCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC linked-quest accept command id is required"; }\n` +
    `    return <uint>${linkedQuestAccept.id};\n}\n\n` +
    `pub releaseCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC spirit-release command id is required"; }\n` +
    `    return <uint>${release.id};\n}\n\n` +
    `pub releaseEmpoweredCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC empowered-release command id is required"; }\n` +
    `    return <uint>${releaseEmpowered.id};\n}\n\n` +
    `pub petAbandonCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-abandon command id is required"; }\n` +
    `    return <uint>${petAbandon.id};\n}\n\n` +
    `pub petRenameCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-rename command id is required"; }\n` +
    `    return <uint>${petRename.id};\n}\n\n` +
    `pub petReviveCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-revive command id is required"; }\n` +
    `    return <uint>${petRevive.id};\n}\n\n` +
    `pub petAttackCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-attack command id is required"; }\n` +
    `    return <uint>${petAttack.id};\n}\n\n` +
    `pub petTauntCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-taunt command id is required"; }\n` +
    `    return <uint>${petTaunt.id};\n}\n\n` +
    `pub petAutoTauntCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet auto-taunt command id is required"; }\n` +
    `    return <uint>${petAutoTaunt.id};\n}\n\n` +
    `pub petWaterJetCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet Water Jet command id is required"; }\n` +
    `    return <uint>${petWaterJet.id};\n}\n\n` +
    `pub petAutoWaterJetCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet Water Jet autocast command id is required"; }\n` +
    `    return <uint>${petAutoWaterJet.id};\n}\n\n` +
    `pub petFeedCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-feed command id is required"; }\n` +
    `    return <uint>${petFeed.id};\n}\n\n` +
    `pub petHealCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-heal command id is required"; }\n` +
    `    return <uint>${petHeal.id};\n}\n\n` +
    `pub petModeCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC pet-mode command id is required"; }\n` +
    `    return <uint>${petMode.id};\n}\n\n` +
    `${socialCommandFunctions}\n` +
    `${partyExtendedCommandFunctions}\n` +
    `${duelArenaCommandFunctions}\n` +
    `${tradeTransportCommandFunctions}\n` +
    `${valeCupCommandFunctions}\n` +
    `${mailIdCommandFunctions}\n` +
    `${bankCommandFunctions}\n` +
    `${dungeonFinderCommandFunctions}\n` +
    `${worldObjectCommandFunctions}\n` +
    `${emptyActionCommandFunctions}\n` +
    `${marketIdCommandFunctions}\n` +
    `${delveRiteCommandFunctions}\n` +
    `${dungeonDifficultyCommandFunctions}\n` +
    `${lootRollCommandFunctions}\n` +
    `${eventSkinCommandFunctions}\n` +
    `pub partyInviteCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-invite command id is required"; }\n` +
    `    return <uint>${partyInvite.id};\n}\n\n` +
    `pub partyAcceptCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-accept command id is required"; }\n` +
    `    return <uint>${partyAccept.id};\n}\n\n` +
    `pub partyDeclineCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-decline command id is required"; }\n` +
    `    return <uint>${partyDecline.id};\n}\n\n` +
    `pub partyLeaveCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-leave command id is required"; }\n` +
    `    return <uint>${partyLeave.id};\n}\n\n` +
    `pub partyKickCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-kick command id is required"; }\n` +
    `    return <uint>${partyKick.id};\n}\n\n` +
    `pub partyPromoteCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-promote command id is required"; }\n` +
    `    return <uint>${partyPromote.id};\n}\n\n` +
    `pub partyRaidCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC party-to-raid command id is required"; }\n` +
    `    return <uint>${partyRaid.id};\n}\n\n` +
    `pub partyUnraidCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC raid-to-party command id is required"; }\n` +
    `    return <uint>${partyUnraid.id};\n}\n\n` +
    `pub partyMoveRaidCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC raid-move command id is required"; }\n` +
    `    return <uint>${partyMoveRaid.id};\n}\n\n` +
    `pub cardQueueJoinCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC Card Duel queue-join command id is required"; }\n` +
    `    return <uint>${cardQueueJoin.id};\n}\n\n` +
    `pub cardQueueLeaveCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC Card Duel queue-leave command id is required"; }\n` +
    `    return <uint>${cardQueueLeave.id};\n}\n\n` +
    `pub cardPlayCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC Card Duel play command id is required"; }\n` +
    `    return <uint>${cardPlay.id};\n}\n\n` +
    `pub cardForfeitCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC Card Duel forfeit command id is required"; }\n` +
    `    return <uint>${cardForfeit.id};\n}\n\n` +
    `pub equipBagCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC equip-bag command id is required"; }\n` +
    `    return <uint>${equipBag.id};\n}\n\n` +
    `pub unequipBagCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC unequip-bag command id is required"; }\n` +
    `    return <uint>${unequipBag.id};\n}\n\n` +
    `pub lockpickEngageCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC lockpick-engage command id is required"; }\n` +
    `    return <uint>${lockpickEngage.id};\n}\n\n` +
    `pub lockpickActionCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC lockpick-action command id is required"; }\n` +
    `    return <uint>${lockpickAction.id};\n}\n\n` +
    `pub lockpickAbortCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC lockpick-abort command id is required"; }\n` +
    `    return <uint>${lockpickAbort.id};\n}\n\n` +
    `pub applyTalentsCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC apply-talents command id is required"; }\n` +
    `    return <uint>${applyTalents.id};\n}\n\n` +
    `pub respecCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC respec command id is required"; }\n` +
    `    return <uint>${respec.id};\n}\n\n` +
    `pub setSpecCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC set-spec command id is required"; }\n` +
    `    return <uint>${setSpec.id};\n}\n\n` +
    `pub saveLoadoutCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC save-loadout command id is required"; }\n` +
    `    return <uint>${saveLoadout.id};\n}\n\n` +
    `pub switchLoadoutCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC switch-loadout command id is required"; }\n` +
    `    return <uint>${switchLoadout.id};\n}\n\n` +
    `pub deleteLoadoutCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC delete-loadout command id is required"; }\n` +
    `    return <uint>${deleteLoadout.id};\n}\n\n` +
    `pub selectTalentRowCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC select-talent-row command id is required"; }\n` +
    `    return <uint>${selectTalentRow.id};\n}\n\n` +
    `pub resurrectCorpseCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC corpse-resurrection command id is required"; }\n` +
    `    return <uint>${resurrectCorpse.id};\n}\n\n` +
    `pub resurrectHealerCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC healer-resurrection command id is required"; }\n` +
    `    return <uint>${resurrectHealer.id};\n}\n\n` +
    `pub resurrectRespondCommandId(required: bool): uint {\n` +
    `    if (!required) { throw "WOC resurrection-response command id is required"; }\n` +
    `    return <uint>${resurrectRespond.id};\n}\n\n` +
      `// kind: 1 empty, 2 target_entity, 3 slot_index, 4 utf8_id,\n` +
    `// 5 u32_index, 6 utf8_id_optional_u32, 7 lockpick_engage,\n` +
    `// 8 lockpick_action, 9 optional_utf8_id, 10 utf8_id_f64_pair,\n` +
    `// 11 utf8_id_optional_utf8_id, 12 talent_row_selection, 13 talent_spec,\n` +
    `// 14 talent_allocation, 15 cosmetic_skin, 16 boolean,\n` +
      `// 17 utf8_id_optional_target_entity, 18 target_entity_raid_group,\n` +
    `// 19 i32_value, 20 guild_event_create, 46 linked_quest_acceptance,\n` +
    `// 53 town_focus_allocation, 54 chat_text, 55 utf8_id_pair, 58 mail_send,\n` +
    `// 59 market_search, 60 trade_offer, 61 challenge_response,\n` +
    `// 0 not yet ported.\n` +
    `pub payloadKind(id: uint, marker: int): int {\n` +
    `    if (marker != 1) { throw "WOC command payload kind marker is invalid"; }\n` +
    `${kindRows}\n    return 0;\n}\n\n` +
    `pub payloadLength(id: uint, required: bool): int {\n` +
    `    if (!required) { throw "WOC command payload length is required"; }\n` +
    `${lengthRows}\n    return -1;\n}\n\n` +
    `pub payloadMinLength(id: uint, required: bool): int {\n` +
    `    if (!required) { throw "WOC command payload minimum length is required"; }\n` +
    `${minLengthRows}\n    return -1;\n}\n\n` +
    `pub payloadMaxLength(id: uint, required: bool): int {\n` +
    `    if (!required) { throw "WOC command payload maximum length is required"; }\n` +
    `${maxLengthRows}\n    return -1;\n}\n\n` +
    `pub contractTest(): int {\n` +
    `    return challengeResponseCommandId(true) == <uint>36 && payloadKind(<uint>36, 1) == 61 &&\n` +
    `        payloadMinLength(<uint>36, true) == 12 && payloadMaxLength(<uint>36, true) == 780 &&\n` +
    `        castSlotCommandId(true) == <uint>0 && castAtCommandId(true) == <uint>1 &&\n` +
    `        payloadKind(<uint>1, 1) == 10 && payloadMinLength(<uint>1, true) == 20 &&\n` +
    `        payloadMaxLength(<uint>1, true) == 276 && castCommandId(true) == <uint>2 &&\n` +
    `        payloadKind(<uint>2, 1) == 17 && payloadMinLength(<uint>2, true) == 5 &&\n` +
    `        payloadMaxLength(<uint>2, true) == 269 && cancelAuraCommandId(true) == <uint>3 &&\n` +
    `        payloadKind(<uint>3, 1) == 4 && payloadMinLength(<uint>3, true) == 4 &&\n` +
        `        payloadMaxLength(<uint>3, true) == 260 && targetCommandId(true) == <uint>4 &&\n` +
        `        tabCommandId(true) == <uint>5 && payloadKind(<uint>5, 1) == 1 &&\n` +
        `        targetNearestCommandId(true) == <uint>6 && payloadKind(<uint>6, 1) == 1 &&\n` +
        `        payloadLength(<uint>6, true) == 0 && tabFriendlyCommandId(true) == <uint>7 &&\n` +
        `        payloadKind(<uint>7, 1) == 1 && targetNearestFriendlyCommandId(true) == <uint>8 &&\n` +
        `        payloadKind(<uint>8, 1) == 1 &&\n` +
        `        changeSkinCommandId(true) == <uint>31 && payloadKind(<uint>31, 1) == 15 &&\n` +
    `        payloadLength(<uint>31, true) == 2 &&\n` +
    `        changeWeaponSkinCommandId(true) == <uint>34 && payloadKind(<uint>34, 1) == 56 &&\n` +
    `        harvestCorpseCommandId(true) == <uint>13 && payloadKind(<uint>13, 1) == 57 &&\n` +
    `        payloadMinLength(<uint>13, true) == 9 && payloadMaxLength(<uint>13, true) == 12 &&\n` +
    `        enterDungeonCommandId(true) == <uint>112 && payloadKind(<uint>112, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>112, true) == 4 && payloadMaxLength(<uint>112, true) == 260 &&\n` +
    `        enterDelveCommandId(true) == <uint>115 && payloadKind(<uint>115, 1) == 55 &&\n` +
    `        payloadMinLength(<uint>115, true) == 8 && payloadMaxLength(<uint>115, true) == 520 &&\n` +
    `        marketSearchCommandId(true) == <uint>101 && payloadKind(<uint>101, 1) == 59 &&\n` +
    `        payloadMinLength(<uint>101, true) == 24 && payloadMaxLength(<uint>101, true) == 1048 &&\n` +
    `        marketListCommandId(true) == <uint>102 && payloadKind(<uint>102, 1) == 10 &&\n` +
    `        payloadMinLength(<uint>102, true) == 20 && payloadMaxLength(<uint>102, true) == 276 &&\n` +
    `        mailSendCommandId(true) == <uint>128 && payloadKind(<uint>128, 1) == 58 &&\n` +
    `        payloadMinLength(<uint>128, true) == 21 && payloadMaxLength(<uint>128, true) == 16384 &&\n` +
    `        payloadMinLength(<uint>34, true) == 2 && payloadMaxLength(<uint>34, true) == 261 &&\n` +
    `        attackCommandId(true) == <uint>9 &&\n` +
    `        stopAttackCommandId(true) == <uint>10 && payloadKind(<uint>4, 1) == 2 &&\n` +
    `        weaponStowCommandId(true) == <uint>162 && payloadKind(<uint>162, 1) == 1 &&\n` +
    `        payloadLength(<uint>162, true) == 0 &&\n` +
    `        interactCommandId(true) == <uint>11 && payloadKind(<uint>11, 1) == 1 &&\n` +
    `        payloadLength(<uint>11, true) == 0 &&\n` +
    `        acceptQuestCommandId(true) == <uint>16 && payloadKind(<uint>16, 1) == 11 &&\n` +
    `        payloadMinLength(<uint>16, true) == 5 && payloadMaxLength(<uint>16, true) == 521 &&\n` +
    `        turnInQuestCommandId(true) == <uint>17 && payloadKind(<uint>17, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>17, true) == 4 && payloadMaxLength(<uint>17, true) == 260 &&\n` +
    `        payloadLength(<uint>4, true) == 8 && payloadKind(<uint>0, 1) == 3 &&\n` +
    `        payloadLength(<uint>0, true) == 4 && payloadKind(<uint>5, 1) == 1 &&\n` +
    `        payloadLength(<uint>5, true) == 0 && payloadKind(<uint>8, 1) == 1 &&\n` +
    `        payloadLength(<uint>8, true) == 0 && payloadKind(<uint>9, 1) == 1 &&\n` +
    `        payloadLength(<uint>9, true) == 0 && abandonCommandId(true) == <uint>18 &&\n` +
    `        payloadKind(<uint>18, 1) == 4 && payloadLength(<uint>18, true) == -1 &&\n` +
    `        payloadMinLength(<uint>18, true) == 4 && payloadMaxLength(<uint>18, true) == 260 &&\n` +
    `        linkedQuestAcceptCommandId(true) == <uint>19 && payloadKind(<uint>19, 1) == 46 &&\n` +
    `        payloadMinLength(<uint>19, true) == 12 && payloadMaxLength(<uint>19, true) == 268 &&\n` +
    `        equipItemCommandId(true) == <uint>20 && payloadKind(<uint>20, 1) == 47 &&\n` +
    `        payloadMinLength(<uint>20, true) == 5 && payloadMaxLength(<uint>20, true) == 261 &&\n` +
    `        unequipItemCommandId(true) == <uint>22 && payloadKind(<uint>22, 1) == 48 &&\n` +
    `        payloadLength(<uint>22, true) == 1 &&\n` +
    `        chatCommandId(true) == <uint>37 && payloadKind(<uint>37, 1) == 54 &&\n` +
    `        payloadMinLength(<uint>37, true) == 4 && payloadMaxLength(<uint>37, true) == 1024 &&\n` +
    `        telemetryCommandId(true) == <uint>125 && payloadKind(<uint>125, 1) == 49 &&\n` +
    `        payloadMinLength(<uint>125, true) == 6 && payloadMaxLength(<uint>125, true) == 65536 &&\n` +
    `        useItemCommandId(true) == <uint>23 && discardItemCommandId(true) == <uint>24 &&\n` +
    `        payloadKind(<uint>24, 1) == 6 && payloadMinLength(<uint>24, true) == 5 &&\n` +
    `        buyCommandId(true) == <uint>25 && payloadKind(<uint>25, 1) == 17 &&\n` +
    `        payloadMinLength(<uint>25, true) == 5 && payloadMaxLength(<uint>25, true) == 269 &&\n` +
    `        sellCommandId(true) == <uint>26 && payloadKind(<uint>26, 1) == 6 &&\n` +
    `        payloadMinLength(<uint>26, true) == 5 && payloadMaxLength(<uint>26, true) == 265 &&\n` +
    `        buybackCommandId(true) == <uint>27 && payloadKind(<uint>27, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>27, true) == 4 && payloadMaxLength(<uint>27, true) == 260 &&\n` +
    `        harvestNodeCommandId(true) == <uint>29 && payloadKind(<uint>29, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>29, true) == 4 && payloadMaxLength(<uint>29, true) == 260 &&\n` +
    `        heroicBuyCommandId(true) == <uint>142 && payloadKind(<uint>142, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>142, true) == 4 && payloadMaxLength(<uint>142, true) == 260 &&\n` +
    `        delveBuyCommandId(true) == <uint>119 && payloadKind(<uint>119, 1) == 55 &&\n` +
    `        payloadMinLength(<uint>119, true) == 8 && payloadMaxLength(<uint>119, true) == 520 &&\n` +
    `        companionUpgradeCommandId(true) == <uint>118 && payloadKind(<uint>118, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>118, true) == 4 && payloadMaxLength(<uint>118, true) == 260 &&\n` +
    `        deedSetTitleCommandId(true) == <uint>159 && payloadKind(<uint>159, 1) == 9 &&\n` +
    `        payloadMinLength(<uint>159, true) == 1 && payloadMaxLength(<uint>159, true) == 261 &&\n` +
    `        setTownFocusCommandId(true) == <uint>140 && payloadKind(<uint>140, 1) == 53 &&\n` +
    `        payloadMinLength(<uint>140, true) == 2 && payloadMaxLength(<uint>140, true) == 65536 &&\n` +
    `        releaseCommandId(true) == <uint>35 && payloadKind(<uint>35, 1) == 1 &&\n` +
    `        payloadLength(<uint>35, true) == 0 &&\n` +
    `        releaseEmpoweredCommandId(true) == <uint>149 && payloadKind(<uint>149, 1) == 4 &&\n` +
    `        payloadMinLength(<uint>149, true) == 4 && payloadMaxLength(<uint>149, true) == 260 &&\n` +
    `        petAbandonCommandId(true) == <uint>53 && petRenameCommandId(true) == <uint>54 &&\n` +
    `        petReviveCommandId(true) == <uint>55 && petAttackCommandId(true) == <uint>56 &&\n` +
    `        petTauntCommandId(true) == <uint>58 && petAutoTauntCommandId(true) == <uint>59 &&\n` +
    `        petFeedCommandId(true) == <uint>61 && petHealCommandId(true) == <uint>62 &&\n` +
    `        petModeCommandId(true) == <uint>63 &&\n` +
    `        petWaterJetCommandId(true) == <uint>57 && payloadKind(<uint>57, 1) == 1 &&\n` +
    `        payloadLength(<uint>57, true) == 0 &&\n` +
    `        petAutoWaterJetCommandId(true) == <uint>60 && payloadKind(<uint>60, 1) == 16 &&\n` +
    `        payloadLength(<uint>60, true) == 1 &&\n` +
    `${socialContractTest} &&\n` +
    `${partyExtendedContractTest} &&\n` +
    `${duelArenaContractTest} &&\n` +
    `${tradeTransportContractTest} &&\n` +
    `${valeCupContractTest} &&\n` +
    `${mailIdContractTest} &&\n` +
    `${bankContractTest} &&\n` +
    `${dungeonFinderContractTest} &&\n` +
    `${worldObjectContractTest} &&\n` +
    `${emptyActionContractTest} &&\n` +
    `${marketIdContractTest} &&\n` +
    `${delveRiteContractTest} &&\n` +
    `${dungeonDifficultyContractTest} &&\n` +
    `${lootRollContractTest} &&\n` +
    `${eventSkinContractTest} &&\n` +
    `        partyInviteCommandId(true) == <uint>39 && payloadKind(<uint>39, 1) == 2 &&\n` +
    `        partyAcceptCommandId(true) == <uint>40 && partyDeclineCommandId(true) == <uint>41 &&\n` +
    `        partyLeaveCommandId(true) == <uint>42 && partyKickCommandId(true) == <uint>43 &&\n` +
    `        partyPromoteCommandId(true) == <uint>44 && partyRaidCommandId(true) == <uint>45 &&\n` +
    `        partyUnraidCommandId(true) == <uint>46 && partyMoveRaidCommandId(true) == <uint>47 &&\n` +
    `        payloadKind(<uint>47, 1) == 18 && payloadLength(<uint>47, true) == 9 &&\n` +
    `        cardQueueJoinCommandId(true) == <uint>90 && payloadKind(<uint>90, 1) == 1 &&\n` +
    `        cardQueueLeaveCommandId(true) == <uint>91 && payloadLength(<uint>91, true) == 0 &&\n` +
    `        cardPlayCommandId(true) == <uint>92 && payloadKind(<uint>92, 1) == 3 &&\n` +
    `        payloadLength(<uint>92, true) == 4 && cardForfeitCommandId(true) == <uint>93 &&\n` +
    `        payloadKind(<uint>93, 1) == 1 && payloadLength(<uint>93, true) == 0 &&\n` +
    `        payloadMaxLength(<uint>24, true) == 265 && equipBagCommandId(true) == <uint>126 &&\n` +
    `        unequipBagCommandId(true) == <uint>127 && payloadKind(<uint>127, 1) == 5 &&\n` +
    `        payloadLength(<uint>127, true) == 4 && lockpickEngageCommandId(true) == <uint>120 &&\n` +
    `        payloadKind(<uint>120, 1) == 7 && payloadLength(<uint>120, true) == 9 &&\n` +
    `        lockpickActionCommandId(true) == <uint>121 && payloadKind(<uint>121, 1) == 8 &&\n` +
    `        payloadMinLength(<uint>121, true) == 2 && payloadMaxLength(<uint>121, true) == 262 &&\n` +
    `        lockpickAbortCommandId(true) == <uint>122 && payloadKind(<uint>122, 1) == 9 &&\n` +
    `        payloadMinLength(<uint>122, true) == 1 && payloadMaxLength(<uint>122, true) == 261 &&\n` +
    `        applyTalentsCommandId(true) == <uint>95 && payloadKind(<uint>95, 1) == 14 &&\n` +
    `        payloadLength(<uint>95, true) == 14 &&\n` +
    `        respecCommandId(true) == <uint>96 && payloadKind(<uint>96, 1) == 1 &&\n` +
    `        payloadLength(<uint>96, true) == 0 &&\n` +
    `        setSpecCommandId(true) == <uint>97 && payloadKind(<uint>97, 1) == 13 &&\n` +
    `        payloadLength(<uint>97, true) == 2 &&\n` +
    `        saveLoadoutCommandId(true) == <uint>98 && payloadKind(<uint>98, 1) == 52 &&\n` +
    `        payloadMinLength(<uint>98, true) == 6 && payloadMaxLength(<uint>98, true) == 5858 &&\n` +
    `        switchLoadoutCommandId(true) == <uint>99 && payloadKind(<uint>99, 1) == 5 &&\n` +
    `        payloadLength(<uint>99, true) == 4 &&\n` +
    `        deleteLoadoutCommandId(true) == <uint>100 && payloadKind(<uint>100, 1) == 5 &&\n` +
    `        payloadLength(<uint>100, true) == 4 &&\n` +
    `        selectTalentRowCommandId(true) == <uint>163 && payloadKind(<uint>163, 1) == 12 &&\n` +
    `        payloadLength(<uint>163, true) == 3 &&\n` +
    `        resurrectCorpseCommandId(true) == <uint>135 && payloadKind(<uint>135, 1) == 1 &&\n` +
    `        payloadLength(<uint>135, true) == 0 &&\n` +
    `        resurrectHealerCommandId(true) == <uint>136 && payloadKind(<uint>136, 1) == 1 &&\n` +
    `        payloadLength(<uint>136, true) == 0 &&\n` +
    `        resurrectRespondCommandId(true) == <uint>164 && payloadKind(<uint>164, 1) == 16 &&\n` +
    `        payloadLength(<uint>164, true) == 1 ? 1 : -1;\n` +
    `}\n`;
}

function renderRust(entries, sha256) {
  const rows = entries.map((entry) => {
    const kind = `CommandPayloadKind::${kindRustName(entry.kind)}`;
    return `    CommandPayloadDescriptor {\n` +
      `        id: ${entry.id},\n` +
      `        name: ${JSON.stringify(entry.name)},\n` +
      `        kind: ${kind},\n` +
      `        min_byte_length: ${entry.min_byte_length},\n` +
      `        max_byte_length: ${entry.max_byte_length},\n` +
      `        max_utf8_bytes: ${entry.max_utf8_bytes ?? 0},\n` +
      `        max_utf16_code_units: ${entry.max_utf16_code_units ?? 0},\n` +
      `        max_collection_entries: ${entry.max_collection_entries ?? 0},\n` +
      `    },`;
  });
  const byName = new Map(entries.map((entry) => [entry.name, entry]));
  const socialConstants = SOCIAL_COMMANDS.map(
    ([name]) => `pub const ${name.toUpperCase()}_COMMAND_ID: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const partyExtendedConstants = PARTY_EXTENDED_COMMANDS.map(([name]) => {
    const constant = {
      setLootMaster: 'PARTY_SET_LOOT_MASTER_COMMAND_ID',
      setMarker: 'PARTY_SET_MARKER_COMMAND_ID',
      clearMarker: 'PARTY_CLEAR_MARKER_COMMAND_ID',
      readyrespond: 'PARTY_READY_RESPOND_COMMAND_ID',
      masterAssign: 'MASTER_ASSIGN_COMMAND_ID',
    }[name];
    return `pub const ${constant}: u16 = ${required(byName, name).id};`;
  }).join('\n');
  const duelArenaConstants = DUEL_ARENA_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const tradeTransportConstants = TRADE_TRANSPORT_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const valeCupConstants = VALE_CUP_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const mailIdConstants = MAIL_ID_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const mailSendConstant = `pub const ${MAIL_SEND_COMMAND[3]}: u16 = ${required(byName, MAIL_SEND_COMMAND[0]).id};`;
  const bankConstants = BANK_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const dungeonFinderConstants = DUNGEON_FINDER_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const worldObjectConstants = WORLD_OBJECT_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const emptyActionConstants = EMPTY_ACTION_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const marketIdConstants = MARKET_ID_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const delveRiteConstants = DELVE_RITE_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const dungeonDifficultyConstants = DUNGEON_DIFFICULTY_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const lootRollConstants = LOOT_ROLL_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  const eventSkinConstants = EVENT_SKIN_COMMANDS.map(([name, , , constant]) =>
    `pub const ${constant}: u16 = ${required(byName, name).id};`,
  ).join('\n');
  return `// Generated by examples/woc/tools/command_payload_codegen.mjs. Do not edit.\n` +
    `pub const COMMAND_PAYLOAD_SCHEMA_SHA256: &str =\n    ${JSON.stringify(sha256)};\n` +
    `pub const CAST_SLOT_COMMAND_ID: u16 = ${required(byName, 'castSlot').id};\n` +
    `pub const CHALLENGE_RESPONSE_COMMAND_ID: u16 = ${required(byName, 'challengeResponse').id};\n` +
    `pub const CAST_AT_COMMAND_ID: u16 = ${required(byName, 'castAt').id};\n` +
    `pub const CAST_COMMAND_ID: u16 = ${required(byName, 'cast').id};\n` +
    `pub const CANCEL_AURA_COMMAND_ID: u16 = ${required(byName, 'cancel_aura').id};\n` +
    `pub const CHANGE_SKIN_COMMAND_ID: u16 = ${required(byName, 'change_skin').id};\n` +
    `pub const UNEQUIP_MECH_CHROMA_COMMAND_ID: u16 = ${required(byName, 'unequip_mech_chroma').id};\n` +
    `pub const CHANGE_WEAPON_SKIN_COMMAND_ID: u16 = ${required(byName, 'change_weapon_skin').id};\n` +
    `pub const HARVEST_CORPSE_COMMAND_ID: u16 = ${required(byName, 'harvestCorpse').id};\n` +
    `pub const ENTER_DUNGEON_COMMAND_ID: u16 = ${required(byName, 'enter_dungeon').id};\n` +
    `pub const ENTER_DELVE_COMMAND_ID: u16 = ${required(byName, 'enter_delve').id};\n` +
    `pub const MARKET_SEARCH_COMMAND_ID: u16 = ${required(byName, 'market_search').id};\n` +
    `pub const MARKET_LIST_COMMAND_ID: u16 = ${required(byName, 'market_list').id};\n` +
    `pub const RELEASE_COMMAND_ID: u16 = ${required(byName, 'release').id};\n` +
    `pub const RELEASE_EMPOWERED_COMMAND_ID: u16 = ${required(byName, 'releaseEmpowered').id};\n` +
    `pub const PET_ABANDON_COMMAND_ID: u16 = ${required(byName, 'pet_abandon').id};\n` +
    `pub const PET_RENAME_COMMAND_ID: u16 = ${required(byName, 'pet_rename').id};\n` +
    `pub const PET_REVIVE_COMMAND_ID: u16 = ${required(byName, 'pet_revive').id};\n` +
    `pub const PET_ATTACK_COMMAND_ID: u16 = ${required(byName, 'pet_attack').id};\n` +
    `pub const PET_TAUNT_COMMAND_ID: u16 = ${required(byName, 'pet_taunt').id};\n` +
    `pub const PET_AUTO_TAUNT_COMMAND_ID: u16 = ${required(byName, 'pet_auto_taunt').id};\n` +
    `pub const PET_WATER_JET_COMMAND_ID: u16 = ${required(byName, 'pet_water_jet').id};\n` +
    `pub const PET_AUTO_WATER_JET_COMMAND_ID: u16 = ${required(byName, 'pet_auto_water_jet').id};\n` +
    `pub const PET_FEED_COMMAND_ID: u16 = ${required(byName, 'pet_feed').id};\n` +
    `pub const PET_HEAL_COMMAND_ID: u16 = ${required(byName, 'pet_heal').id};\n` +
    `pub const PET_MODE_COMMAND_ID: u16 = ${required(byName, 'pet_mode').id};\n` +
    `${socialConstants}\n` +
    `${partyExtendedConstants}\n` +
    `${duelArenaConstants}\n` +
    `${tradeTransportConstants}\n` +
    `${valeCupConstants}\n` +
    `${mailIdConstants}\n` +
    `${mailSendConstant}\n` +
    `${bankConstants}\n` +
    `${dungeonFinderConstants}\n` +
    `${worldObjectConstants}\n` +
    `${emptyActionConstants}\n` +
    `${marketIdConstants}\n` +
    `${delveRiteConstants}\n` +
    `${dungeonDifficultyConstants}\n` +
    `${lootRollConstants}\n` +
    `${eventSkinConstants}\n` +
    `pub const PARTY_INVITE_COMMAND_ID: u16 = ${required(byName, 'pinvite').id};\n` +
    `pub const PARTY_ACCEPT_COMMAND_ID: u16 = ${required(byName, 'paccept').id};\n` +
    `pub const PARTY_DECLINE_COMMAND_ID: u16 = ${required(byName, 'pdecline').id};\n` +
    `pub const PARTY_LEAVE_COMMAND_ID: u16 = ${required(byName, 'pleave').id};\n` +
    `pub const PARTY_KICK_COMMAND_ID: u16 = ${required(byName, 'pkick').id};\n` +
    `pub const PARTY_PROMOTE_COMMAND_ID: u16 = ${required(byName, 'ppromote').id};\n` +
    `pub const PARTY_RAID_COMMAND_ID: u16 = ${required(byName, 'praid').id};\n` +
    `pub const PARTY_UNRAID_COMMAND_ID: u16 = ${required(byName, 'punraid').id};\n` +
    `pub const PARTY_MOVE_RAID_COMMAND_ID: u16 = ${required(byName, 'pmoveRaid').id};\n` +
    `pub const CARD_QUEUE_JOIN_COMMAND_ID: u16 = ${required(byName, 'card_queue_join').id};\n` +
    `pub const CARD_QUEUE_LEAVE_COMMAND_ID: u16 = ${required(byName, 'card_queue_leave').id};\n` +
    `pub const CARD_PLAY_COMMAND_ID: u16 = ${required(byName, 'play_card').id};\n` +
    `pub const CARD_FORFEIT_COMMAND_ID: u16 = ${required(byName, 'card_forfeit').id};\n` +
    `pub const TARGET_COMMAND_ID: u16 = ${required(byName, 'target').id};\n` +
    `pub const TAB_COMMAND_ID: u16 = ${required(byName, 'tab').id};\n` +
    `pub const TARGET_NEAREST_COMMAND_ID: u16 = ${required(byName, 'targetNearest').id};\n` +
    `pub const TAB_FRIENDLY_COMMAND_ID: u16 = ${required(byName, 'tabFriendly').id};\n` +
    `pub const TARGET_NEAREST_FRIENDLY_COMMAND_ID: u16 = ${required(byName, 'targetNearestFriendly').id};\n` +
    `pub const ATTACK_COMMAND_ID: u16 = ${required(byName, 'attack').id};\n` +
    `pub const STOP_ATTACK_COMMAND_ID: u16 = ${required(byName, 'stopattack').id};\n` +
    `pub const WEAPON_STOW_COMMAND_ID: u16 = ${required(byName, 'stow_weapon').id};\n` +
    `pub const INTERACT_COMMAND_ID: u16 = ${required(byName, 'interact').id};\n` +
    `pub const ACCEPT_QUEST_COMMAND_ID: u16 = ${required(byName, 'accept').id};\n` +
    `pub const LINKED_QUEST_ACCEPT_COMMAND_ID: u16 = ${required(byName, 'qlinkaccept').id};\n` +
    `pub const TURN_IN_QUEST_COMMAND_ID: u16 = ${required(byName, 'turnin').id};\n` +
    `pub const ABANDON_QUEST_COMMAND_ID: u16 = ${required(byName, 'abandon').id};\n` +
    `pub const EQUIP_ITEM_COMMAND_ID: u16 = ${required(byName, 'equip').id};\n` +
    `pub const INVENTORY_MOVE_COMMAND_ID: u16 = ${required(byName, 'inv_move').id};\n` +
    `pub const UNEQUIP_ITEM_COMMAND_ID: u16 = ${required(byName, 'unequip_item').id};\n` +
    `pub const EMOTE_COMMAND_ID: u16 = ${required(byName, 'emote').id};\n` +
    `pub const CHAT_COMMAND_ID: u16 = ${required(byName, 'chat').id};\n` +
    `pub const TELEMETRY_COMMAND_ID: u16 = ${required(byName, 'telemetry').id};\n` +
    `pub const USE_ITEM_COMMAND_ID: u16 = ${required(byName, 'use').id};\n` +
    `pub const DISCARD_ITEM_COMMAND_ID: u16 = ${required(byName, 'discard').id};\n` +
    `pub const BUY_COMMAND_ID: u16 = ${required(byName, 'buy').id};\n` +
    `pub const SELL_COMMAND_ID: u16 = ${required(byName, 'sell').id};\n` +
    `pub const BUYBACK_COMMAND_ID: u16 = ${required(byName, 'buyback').id};\n` +
    `pub const HARVEST_NODE_COMMAND_ID: u16 = ${required(byName, 'harvest_node').id};\n` +
    `pub const CRAFT_ITEM_COMMAND_ID: u16 = ${required(byName, 'craft_item').id};\n` +
    `pub const HEROIC_BUY_COMMAND_ID: u16 = ${required(byName, 'heroic_buy').id};\n` +
    `pub const DELVE_BUY_COMMAND_ID: u16 = ${required(byName, 'delve_buy').id};\n` +
    `pub const COMPANION_UPGRADE_COMMAND_ID: u16 = ${required(byName, 'companion_upgrade').id};\n` +
    `pub const DEED_SET_TITLE_COMMAND_ID: u16 = ${required(byName, 'deed_set_title').id};\n` +
    `pub const SET_TOWN_FOCUS_COMMAND_ID: u16 = ${required(byName, 'set_town_focus').id};\n` +
    `pub const EQUIP_BAG_COMMAND_ID: u16 = ${required(byName, 'equip_bag').id};\n` +
    `pub const UNEQUIP_BAG_COMMAND_ID: u16 = ${required(byName, 'unequip_bag').id};\n` +
    `pub const LOCKPICK_ENGAGE_COMMAND_ID: u16 = ${required(byName, 'lockpick_engage').id};\n` +
    `pub const LOCKPICK_ACTION_COMMAND_ID: u16 = ${required(byName, 'lockpick_action').id};\n` +
    `pub const LOCKPICK_ABORT_COMMAND_ID: u16 = ${required(byName, 'lockpick_abort').id};\n` +
    `pub const APPLY_TALENTS_COMMAND_ID: u16 = ${required(byName, 'applyTalents').id};\n` +
    `pub const RESPEC_COMMAND_ID: u16 = ${required(byName, 'respec').id};\n` +
    `pub const SET_SPEC_COMMAND_ID: u16 = ${required(byName, 'setSpec').id};\n` +
    `pub const SAVE_LOADOUT_COMMAND_ID: u16 = ${required(byName, 'saveLoadout').id};\n` +
    `pub const SWITCH_LOADOUT_COMMAND_ID: u16 = ${required(byName, 'switchLoadout').id};\n` +
    `pub const DELETE_LOADOUT_COMMAND_ID: u16 = ${required(byName, 'deleteLoadout').id};\n` +
    `pub const SELECT_TALENT_ROW_COMMAND_ID: u16 = ${required(byName, 'selectTalentRow').id};\n` +
    `pub const RESURRECT_CORPSE_COMMAND_ID: u16 = ${required(byName, 'resurrect_corpse').id};\n` +
    `pub const RESURRECT_HEALER_COMMAND_ID: u16 = ${required(byName, 'resurrect_healer').id};\n` +
    `pub const RESURRECT_RESPOND_COMMAND_ID: u16 = ${required(byName, 'resurrect_respond').id};\n\n` +
    `#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n` +
    `pub enum CommandPayloadKind {\n` +
      `    Empty,\n    TargetEntity,\n    SlotIndex,\n    Utf8Id,\n    CorpseHarvest,\n    MailSend,\n    MarketSearch,\n    TradeOffer,\n    ChallengeResponse,\n` +
      `    U32Index,\n    Utf8IdOptionalU32,\n    Utf8IdOptionalTargetEntity,\n    TargetEntityRaidGroup,\n    LockpickEngage,\n    LockpickAction,\n    OptionalUtf8Id,\n    Utf8IdF64Pair,\n    Utf8IdOptionalUtf8Id,\n    Utf8IdPair,\n    TalentRowSelection,\n    TalentSpec,\n    TalentAllocation,\n    SaveLoadout,\n    CosmeticSkin,\n    Boolean,\n    I32Value,\n    I32Pair,\n    EmoteId,\n    ChatText,\n    GuildEventCreate,\n    PartyLootMaster,\n    MasterLootAssignment,\n    PartyMarker,\n    PartyMarkerClear,\n    DuelRequest,\n    ArenaQueueFormat,\n    ArenaAugment,\n    TradeRequest,\n    ValeCupQueue,\n    ValeCupRole,\n    ValeCupBet,\n    ValeCupBracket,\n    MailId,\n    BankSlotOptionalCount,\n    DungeonFinderRoles,\n    DungeonFinderActivities,\n    DungeonFinderListing,\n    DungeonFinderListingId,\n    DungeonFinderApplicationResponse,\n    WorldObjectId,\n    MarketListingId,\n    DelveRiteIntensity,\n    DungeonDifficulty,\n    LootRoll,\n    EventSkin,\n    LinkedQuestAcceptance,\n    EquipmentItemOptionalSlot,\n    EquipmentSlot,\n    TelemetryNumericFields,\n    TownFocusAllocation,\n}\n\n` +
    `#[derive(Clone, Copy, Debug, PartialEq, Eq)]\n` +
    `pub struct CommandPayloadDescriptor {\n` +
    `    pub id: u16,\n    pub name: &'static str,\n    pub kind: CommandPayloadKind,\n` +
    `    pub min_byte_length: usize,\n    pub max_byte_length: usize,\n` +
    `    pub max_utf8_bytes: usize,\n    pub max_utf16_code_units: usize,\n` +
    `    pub max_collection_entries: usize,\n}\n\n` +
    `impl CommandPayloadDescriptor {\n` +
    `    pub const fn fixed_byte_length(self) -> Option<usize> {\n` +
    `        if self.min_byte_length == self.max_byte_length {\n` +
    `            Some(self.min_byte_length)\n` +
    `        } else {\n            None\n        }\n    }\n}\n\n` +
    `pub const COMMAND_PAYLOAD_CATALOG: &[CommandPayloadDescriptor] = &[\n${rows.join('\n')}\n];\n\n` +
    `pub fn command_payload_descriptor(id: u16) -> Option<&'static CommandPayloadDescriptor> {\n` +
    `    COMMAND_PAYLOAD_CATALOG.iter().find(|entry| entry.id == id)\n}\n`;
}

function kindCode(kind) {
  return {
    empty: 1,
    target_entity: 2,
    slot_index: 3,
    utf8_id: 4,
    u32_index: 5,
    utf8_id_optional_u32: 6,
    lockpick_engage: 7,
    lockpick_action: 8,
    optional_utf8_id: 9,
    utf8_id_f64_pair: 10,
    utf8_id_optional_utf8_id: 11,
    utf8_id_pair: 55,
    talent_row_selection: 12,
    talent_spec: 13,
    talent_allocation: 14,
    cosmetic_skin: 15,
    boolean: 16,
    utf8_id_optional_target_entity: 17,
      target_entity_raid_group: 18,
      i32_value: 19,
    guild_event_create: 20,
    party_loot_master: 21,
    master_loot_assignment: 45,
    linked_quest_acceptance: 46,
    equipment_item_optional_slot: 47,
    equipment_slot: 48,
    telemetry_numeric_fields: 49,
    town_focus_allocation: 53,
    i32_pair: 50,
    emote_id: 51,
    save_loadout: 52,
    chat_text: 54,
    party_marker: 22,
    party_marker_clear: 23,
    duel_request: 24,
    arena_queue_format: 25,
    arena_augment: 26,
    trade_request: 27,
    vale_cup_queue: 28,
    vale_cup_role: 29,
    vale_cup_bet: 30,
    vale_cup_bracket: 31,
    mail_id: 32,
    bank_slot_optional_count: 33,
    dungeon_finder_roles: 34,
    dungeon_finder_activities: 35,
    dungeon_finder_listing: 36,
    dungeon_finder_listing_id: 37,
    dungeon_finder_application_response: 38,
    world_object_id: 39,
    market_listing_id: 40,
    delve_rite_intensity: 41,
    dungeon_difficulty: 42,
    loot_roll: 43,
    event_skin: 44,
    weapon_skin_change: 56,
    corpse_harvest: 57,
    mail_send: 58,
    market_search: 59,
    trade_offer: 60,
    challenge_response: 61,
  }[kind] ?? 0;
}

function kindRustName(kind) {
  const name = {
    empty: 'Empty',
    target_entity: 'TargetEntity',
    slot_index: 'SlotIndex',
    utf8_id: 'Utf8Id',
    u32_index: 'U32Index',
    utf8_id_optional_u32: 'Utf8IdOptionalU32',
    lockpick_engage: 'LockpickEngage',
    lockpick_action: 'LockpickAction',
    optional_utf8_id: 'OptionalUtf8Id',
    utf8_id_f64_pair: 'Utf8IdF64Pair',
    utf8_id_optional_utf8_id: 'Utf8IdOptionalUtf8Id',
    utf8_id_pair: 'Utf8IdPair',
    talent_row_selection: 'TalentRowSelection',
    talent_spec: 'TalentSpec',
    talent_allocation: 'TalentAllocation',
    cosmetic_skin: 'CosmeticSkin',
    boolean: 'Boolean',
    utf8_id_optional_target_entity: 'Utf8IdOptionalTargetEntity',
      target_entity_raid_group: 'TargetEntityRaidGroup',
      i32_value: 'I32Value',
    guild_event_create: 'GuildEventCreate',
    party_loot_master: 'PartyLootMaster',
    master_loot_assignment: 'MasterLootAssignment',
    linked_quest_acceptance: 'LinkedQuestAcceptance',
    equipment_item_optional_slot: 'EquipmentItemOptionalSlot',
    equipment_slot: 'EquipmentSlot',
    telemetry_numeric_fields: 'TelemetryNumericFields',
    town_focus_allocation: 'TownFocusAllocation',
    i32_pair: 'I32Pair',
    emote_id: 'EmoteId',
    save_loadout: 'SaveLoadout',
    chat_text: 'ChatText',
    party_marker: 'PartyMarker',
    party_marker_clear: 'PartyMarkerClear',
    duel_request: 'DuelRequest',
    arena_queue_format: 'ArenaQueueFormat',
    arena_augment: 'ArenaAugment',
    trade_request: 'TradeRequest',
    vale_cup_queue: 'ValeCupQueue',
    vale_cup_role: 'ValeCupRole',
    vale_cup_bet: 'ValeCupBet',
    vale_cup_bracket: 'ValeCupBracket',
    mail_id: 'MailId',
    bank_slot_optional_count: 'BankSlotOptionalCount',
    dungeon_finder_roles: 'DungeonFinderRoles',
    dungeon_finder_activities: 'DungeonFinderActivities',
    dungeon_finder_listing: 'DungeonFinderListing',
    dungeon_finder_listing_id: 'DungeonFinderListingId',
    dungeon_finder_application_response: 'DungeonFinderApplicationResponse',
    world_object_id: 'WorldObjectId',
    market_listing_id: 'MarketListingId',
    delve_rite_intensity: 'DelveRiteIntensity',
    dungeon_difficulty: 'DungeonDifficulty',
    loot_roll: 'LootRoll',
    event_skin: 'EventSkin',
    weapon_skin_change: 'WeaponSkinChange',
    corpse_harvest: 'CorpseHarvest',
    mail_send: 'MailSend',
    market_search: 'MarketSearch',
    trade_offer: 'TradeOffer',
    challenge_response: 'ChallengeResponse',
  }[kind];
  invariant(name, `invalid payload kind ${kind}`);
  return name;
}

function required(entries, name) {
  const entry = entries.get(name);
  invariant(entry, `missing required command payload ${name}`);
  return entry;
}

function writeOrCheck(path, content) {
  if (checkOnly) {
    invariant(existsSync(path), `${path} is missing; run npm run generate`);
    invariant(readFileSync(path, 'utf8') === content, `${path} is stale; run npm run generate`);
    return;
  }
  writeFileSync(path, content, 'utf8');
}

function invariant(condition, message) {
  if (!condition) throw new Error(message);
}
