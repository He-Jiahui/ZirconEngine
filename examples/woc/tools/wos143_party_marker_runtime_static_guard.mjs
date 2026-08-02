import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const SOURCE_COMMIT = '5ef9f7cb21cd8875b6d2c49701015dfcd78de35a';
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const workspaceRoot = path.resolve(root, '..', '..');
const sourceRoot = path.resolve(workspaceRoot, 'dev', 'world-of-claudecraft');
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), 'utf8');
const source = (file) => execFileSync(
  'git', ['-C', sourceRoot, 'show', `${SOURCE_COMMIT}:${file}`], { encoding: 'utf8' },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const targeting = source('src/sim/targeting.ts');
requireText(targeting, /setMarker\(entityId[\s\S]*?partyOf[\s\S]*?markerId < 0 \|\| markerId > 7[\s\S]*?kind !== 'mob'[\s\S]*?target\.dead[\s\S]*?!target\.hostile[\s\S]*?ownerId !== null/,
  'source raid-marker admission drifted');
requireText(targeting, /marks\.get\(entityId\) === markerId[\s\S]*?marks\.delete\(entityId\)[\s\S]*?mid === markerId[\s\S]*?marks\.set\(entityId, markerId\)/,
  'source raid-marker toggle or uniqueness drifted');
requireText(targeting, /clearEntityMarker[\s\S]*?marks\.delete\(entityId\)[\s\S]*?dropPartyMarkers[\s\S]*?partyMarkers\.delete\(partyId\)/,
  'source raid-marker lifecycle cleanup drifted');

const payloadContract = JSON.parse(read('contracts', 'command_payloads.json'));
const setPayload = payloadContract.entries.find((entry) => entry.name === 'setMarker');
const clearPayload = payloadContract.entries.find((entry) => entry.name === 'clearMarker');
if (setPayload?.id !== 50 || setPayload.encoding !== 'f64_le_entity_id+f64_le_marker_id' ||
    clearPayload?.id !== 51 || clearPayload.encoding !== 'f64_le_entity_id') {
  throw new Error('raid-marker payload contract drifted');
}

const markerModule = read('scripts', 'woc_game', 'src', 'social', 'targeting_markers_state.zr');
requireText(markerModule, new RegExp(SOURCE_COMMIT), 'marker module source pin is stale');
requireText(markerModule, /setParty[\s\S]*?while \(parties\.length < player\)[\s\S]*?setParty\(state, 4, 2\)/,
  'marker module must support the retained raid capacity');

const world = read('scripts', 'woc_game', 'src', 'world', 'state.zr');
requireText(world, /%import\("social\/targeting_markers_state"\)/,
  'world marker module import is missing');
requireText(world, /partyMarkerPartyIds[\s\S]*?partyMarkerEntityIds[\s\S]*?partyMarkerSymbols/,
  'WOS party-marker columns are missing');
requireText(world, /restorePartyMarkerState[\s\S]*?commitPartyMarkerState/,
  'party-marker projection bridge is missing');
requireText(world, /partySetMarkerCommandId\(true\)[\s\S]*?partyClearMarkerCommandId\(true\)[\s\S]*?applyPartyMarkerCommand/,
  'party-marker reducer routing is missing');
requireText(world, /applyPartyMarkerCommand[\s\S]*?binary\.readF64LeAt[\s\S]*?targetingMarkers\.setMarker[\s\S]*?targetingMarkers\.clearMarker/,
  'party-marker finite-f64 reducer is missing');
requireText(world, /partyRemoveMember[\s\S]*?dropPartyMarkerRows\(state, partyId\)/,
  'party-marker disband cleanup is missing');
requireText(world, /dropPartyMarkerRows[\s\S]*?targetingMarkers\.dropPartyMarkers/,
  'party-marker disband bridge is missing');
requireText(world, /clearDeadCasting[\s\S]*?clearPartyMarkerEntityRows/,
  'party-marker death cleanup is missing');
requireText(world, /clearPartyMarkerEntityRows[\s\S]*?targetingMarkers\.clearEntityMarker/,
  'party-marker death bridge is missing');
requireText(world, /writer\.u16\(<uint>78, 1, 1\)[\s\S]*?partyMarkerPartyIds[\s\S]*?partyMarkerEntityIds[\s\S]*?partyMarkerSymbols/,
  'WOS72 marker tail is missing');
requireText(world, /if \(schemaVersion >= <uint>69\)[\s\S]*?partyMarkerPartyIds[\s\S]*?partyMarkerEntityIds[\s\S]*?partyMarkerSymbols/,
  'WOS72 marker migration is missing');
requireText(world, /pub partyMarkerCommandStateTest\(\): int[\s\S]*?setMarker[\s\S]*?clearMarker[\s\S]*?encodeState[\s\S]*?clearDeadCasting/,
  'party-marker state regression is missing');
requireText(world, /if \(partyMarkerCommandStateTest\(\) != 1\) \{[\s\S]*?return -137;/,
  'world selfTest must execute party-marker coverage');

process.stdout.write(`WOS143 party-marker static guards passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
