import { readFile } from "node:fs/promises";
import { resolve } from "node:path";

const repoRoot = resolve(import.meta.dirname, "..", "..", "..");
const source = await readFile(
  resolve(repoRoot, "dev/world-of-claudecraft/src/sim/interaction.ts"),
  "utf8",
);
const damage = await readFile(
  resolve(repoRoot, "dev/world-of-claudecraft/src/sim/combat/damage.ts"),
  "utf8",
);
const locomotion = await readFile(
  resolve(repoRoot, "dev/world-of-claudecraft/src/sim/mob/locomotion.ts"),
  "utf8",
);
const state = await readFile(
  resolve(repoRoot, "examples/woc/scripts/woc_game/src/world/corpse_loot_rights_state.zr"),
  "utf8",
);

function requireText(text, needle, label) {
  if (!text.includes(needle)) {
    throw new Error(label + " is missing: " + needle);
  }
}

requireText(source, "lootCorpse(", "source manual loot route");
requireText(source, "autoLootForParty(", "source passive loot route");
requireText(source, "honorFfa && lootHasGoneFfa", "source FFA admission");
requireText(damage, "target.tappedById === null", "source first-hit tap gate");
requireText(damage, "source.kind === 'player' ? source.id : source.ownerId", "source pet tap owner");
requireText(locomotion, "if (mob.lootFfaTimer > 0) mob.lootFfaTimer -= DT", "source FFA tick");

for (const needle of [
  "recordDamageTap(",
  "beginCorpseLoot(",
  "advance(state: CorpseLootRightsState",
  "sharedLootAllowed(",
  "honorFfa &&",
  "entityTapperPresent",
  "entityLootFfaTimerActive",
  "lootFfaDelayMicros()",
]) {
  requireText(state, needle, "WOC corpse loot rights state");
}

if (state.includes("entityOwnerIds[targetIndex]")) {
  throw new Error("WOC uses owner identity as a tapper instead of resolving a player owner");
}

console.log("corpse loot rights source contract: ok");
