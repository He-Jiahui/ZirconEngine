import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(import.meta.dirname, '..');
const source = readFileSync(
  resolve(root, '..', '..', 'dev', 'world-of-claudecraft', 'src', 'sim', 'pet', 'pet_commands.ts'),
  'utf8',
);
const payloads = JSON.parse(readFileSync(resolve(root, 'contracts', 'command_payloads.json'), 'utf8'));
const world = readFileSync(resolve(root, 'scripts', 'woc_game', 'src', 'world', 'state.zr'), 'utf8');
const contract = readFileSync(resolve(root, 'contracts', 'world-state.md'), 'utf8');

function requireText(text, expected, label) {
  if (!text.includes(expected)) throw new Error(`${label}: missing ${JSON.stringify(expected)}`);
}

for (const [id, name, kind] of [
  [55, 'pet_revive', 'empty'],
  [56, 'pet_attack', 'empty'],
  [57, 'pet_water_jet', 'empty'],
  [58, 'pet_taunt', 'empty'],
  [59, 'pet_auto_taunt', 'boolean'],
  [60, 'pet_auto_water_jet', 'boolean'],
  [63, 'pet_mode', 'utf8_id'],
]) {
  const entry = payloads.entries.find((candidate) => candidate.id === id);
  if (!entry || entry.name !== name || entry.kind !== kind) {
    throw new Error(`pet command payload drifted: ${id}`);
  }
}

for (const expected of [
  'export function revivePet',
  'export function petAttack',
  'export function petTaunt',
  'export function petWaterJet',
  'export function setPetMode',
  'export function setPetAutoTaunt',
  'export function setPetAutoWaterJet',
  'Math.round(pet.maxHp * 0.35)',
]) requireText(source, expected, 'source pet command');

for (const expected of [
  'var petReviveCommand = payloads.petReviveCommandId(true);',
  'var petAttackCommand = payloads.petAttackCommandId(true);',
  'var petModeCommand = payloads.petModeCommandId(true);',
  'applyOfflineEmberkinPetCommand(',
  'offlineOwnedEmberkinPetIndex(',
  'offlinePetModeFromPayload(',
  'reviveOfflineEmberkinPet(',
  'state.entityPetModes[petIndex] = mode;',
  'state.setThreat(',
  'var restoredHp = <int>math.round(<float>state.entityMaxHp[petIndex] * 0.35);',
  'pub petCommandStateTest(): int',
]) requireText(world, expected, 'world reducer');

requireText(contract, 'The WOS62 Emberkin pet-command closure', 'world-state contract');
requireText(contract, '`pet_revive` restores the retained dead Emberkin', 'world-state contract');

process.stdout.write('WOS76 Emberkin pet-command runtime static guard passed\n');
