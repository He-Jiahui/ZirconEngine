import { readFileSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = resolve(dirname(fileURLToPath(import.meta.url)), '..');
const read = (path) => readFileSync(resolve(root, path), 'utf8');
const requireText = (source, text, label) => {
  if (!source.includes(text)) throw new Error(`WOS50 source contract missing ${label}: ${text}`);
};

const world = read('scripts/woc_game/src/world/state.zr');
const main = read('scripts/woc_game/src/main.zr');
const protocol = read('native/crates/woc_protocol/src/lib.rs');

for (const text of [
  'writer.u16(<uint>67, 1, 1);',
  'schemaVersion != <uint>49 && schemaVersion != <uint>50 &&',
  'schemaVersion != <uint>51 && schemaVersion != <uint>52 &&',
  'schemaVersion != <uint>53 && schemaVersion != <uint>54 &&',
  'schemaVersion != <uint>55',
  'pub var entityWanderTargetPresent: container.Array<bool>;',
  'pub var entityWanderTargetX: container.Array<float>;',
  'pub var entityWanderTargetZ: container.Array<float>;',
  'writer.byte(<bool>state.entityWanderTargetPresent[index] ? <uint>1 : <uint>0, 1);',
  'if (schemaVersion >= <uint>50) {',
  'state.entityWanderTargetPresent.add(false);',
  'state.entityWanderTargetX.add(0.0);',
  'state.entityWanderTargetZ.add(0.0);',
  'eastbrookRngCursor.constructorCursorAfterCampSpawns(',
]) requireText(world, text, 'world state');

requireText(main, '\\"world_state\\":\\"WOS67\\"', 'package metadata');
requireText(protocol, 'pub const WORLD_STATE_FORMAT: &str = "WOS67";', 'native state format');
requireText(protocol, 'pub const WORLD_STATE_SCHEMA_VERSION: u16 = 67;', 'native schema version');

process.stdout.write('WOS50 wander-state static guards passed\n');
