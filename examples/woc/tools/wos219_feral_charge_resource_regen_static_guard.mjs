import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const SOURCE_COMMIT = "5ef9f7cb21cd8875b6d2c49701015dfcd78de35a";
const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const workspaceRoot = path.resolve(root, "..", "..");
const sourceRoot = path.resolve(workspaceRoot, "dev", "world-of-claudecraft");
const read = (...parts) => fs.readFileSync(path.join(root, ...parts), "utf8");
const source = (file) => execFileSync(
  "git", ["-C", sourceRoot, "show", `${SOURCE_COMMIT}:${file}`], { encoding: "utf8" },
);
const requireText = (text, pattern, message) => {
  if (!pattern.test(text)) throw new Error(message);
};

const classes = source("src/sim/content/classes.ts");
const dispatch = source("src/sim/combat/effect_dispatch.ts");
const auras = source("src/sim/combat/auras.ts");
requireText(
  classes,
  /feral_charge:[\s\S]*?class: 'druid'[\s\S]*?learnLevel: 10[\s\S]*?cost: 0[\s\S]*?castTime: 0[\s\S]*?cooldown: 90[\s\S]*?school: 'physical'[\s\S]*?offGcd: true[\s\S]*?usableInForm: true[\s\S]*?type: 'feralCharge'/,
  "source Feral Charge profile drifted",
);
requireText(
  dispatch,
  /case 'feralCharge':[\s\S]*?a\.kind === 'form_cat'[\s\S]*?id: 'feral_instinct_energy'[\s\S]*?kind: 'buff_energyregen'[\s\S]*?remaining: 10[\s\S]*?value: 1[\s\S]*?a\.kind === 'form_bear'[\s\S]*?resourceType === 'rage'[\s\S]*?Math\.min\(p\.maxResource, p\.resource \+ 50\)/,
  "source Feral Charge form branches drifted",
);
requireText(
  auras,
  /tickCount % 40 !== 0[\s\S]*?resourceType === 'energy'[\s\S]*?let regen = 20[\s\S]*?kind === 'buff_energyregen'[\s\S]*?regen \*= 1 \+ a\.value[\s\S]*?Math\.round\(regen\)/,
  "source Energy regeneration order or multiplier drifted",
);

const generator = read("tools", "m4_ability_codegen.mjs");
const zrGenerator = read("tools", "m4_ability_zr_codegen.mjs");
requireText(generator, /'feral_charge'[\s\S]*?EXPECTED_ABILITY_COUNT = 117/, "Feral Charge is absent from M4 projection");
requireText(zrGenerator, /document\.entries\.length === 117/, "M4 Zr projection count drifted");
requireText(zrGenerator, /'usableInForm'/, "M4 Zr projection omits usableInForm");

const m4 = JSON.parse(read("contracts", "m4_abilities.json"));
const feralCharge = m4.entries.find((entry) => entry.id === "feral_charge");
if (m4.entries.length !== 117 || !feralCharge || feralCharge.index !== 94 ||
    feralCharge.definition.class !== "druid" || feralCharge.definition.learnLevel !== 10 ||
    feralCharge.definition.cost !== 0 || feralCharge.definition.castTime !== 0 ||
    feralCharge.definition.cooldown !== 90 || feralCharge.definition.range !== 0 ||
    feralCharge.definition.school !== "physical" || feralCharge.definition.requiresTarget ||
    !feralCharge.definition.offGcd || !feralCharge.definition.usableInForm ||
    feralCharge.definition.effects?.[0]?.type !== "feralCharge") {
  throw new Error("M4 Feral Charge source projection drifted");
}

const helper = read("scripts", "woc_game", "src", "combat", "feral_charge_state.zr");
requireText(
  helper,
  /resourceTickModulo\(\): uint \{ return <uint>40; \}[\s\S]*?feralInstinctDuration\(\): float \{ return 10\.0; \}[\s\S]*?bearRageGrant\(\): int \{ return 50; \}[\s\S]*?energyRegenAmount[\s\S]*?baseEnergyRegen\(\) \* 2/,
  "Feral Charge resource helper drifted",
);

const world = read("scripts", "woc_game", "src", "world", "state.zr");
requireText(
  world,
  /entityFeralInstinctEnergyRemaining[\s\S]*?appendDefaultFeralInstinctColumns[\s\S]*?feralInstinctStateIsSchema111Compatible[\s\S]*?schemaVersion >= <uint>112/,
  "WOS112 Feral Instinct persistence is missing",
);
requireText(
  world,
  /feralChargeAbilityCode[\s\S]*?feralChargeProfileIsValid[\s\S]*?offGcd[\s\S]*?usableInForm[\s\S]*?startOfflineFeralChargeCast[\s\S]*?form_cat[\s\S]*?entityFeralInstinctEnergyRemaining[\s\S]*?form_bear[\s\S]*?bearRageGrant/,
  "Feral Charge cast reducer is incomplete",
);
requireText(
  world,
  /applySupportedCastSlotCommand[\s\S]*?feralChargeAbilityCode\(\)[\s\S]*?startOfflineFeralChargeCast[\s\S]*?applySupportedCastCommand[\s\S]*?feralChargePayloadAbilityIsExact[\s\S]*?startOfflineFeralChargeCast/,
  "Feral Charge generic and typed routes are missing",
);
requireText(
  world,
  /stepRetainedPlayerTicks[\s\S]*?stepRetainedCasting[\s\S]*?stepOfflineFeralInstinctEnergyRegen[\s\S]*?ageOfflineFeralInstinct[\s\S]*?stepOfflineConsumables/,
  "Feral Instinct must regenerate before it ages",
);
const testOffset = world.indexOf("pub feralChargeResourceRegenRuntimeStateTest(): int");
const runtimeTest = testOffset >= 0 ? world.slice(testOffset) : "";
for (const required of [
  "abilityIndex != 94",
  "forms.formAbilityCode(\"form_bear\")",
  "entityFeralInstinctEnergyRemaining[0], 10.0",
  "state.tick = <uint>40",
  "entityResources[0] != 40",
  "entityFeralInstinctEnergyRemaining[0] = 0.05",
  "restored.tick = <uint>120",
  "entityResources[0] != 20",
  "restored.entityCastGcdRemaining[0] = 0.75",
  "entityCastGcdRemaining[0], 0.75",
  "encodeStateVersion(restored, <uint>111)",
]) {
  if (!runtimeTest.includes(required)) {
    throw new Error(`Feral Charge runtime regression coverage is missing: ${required}`);
  }
}

const main = read("scripts", "woc_game", "src", "main.zr");
if (!main.includes('world_state\\\":\\\"WOS118')) {
  throw new Error("package state identity must publish WOS118");
}
const fixture = read("scripts", "woc_game", "woc_m4_feral_charge_resource_regen_runtime_tests.zrp");
if (!fixture.includes('backend = "zr_vm:project"') ||
    !fixture.includes('entry = "src/world/feral_charge_resource_regen_runtime_test_main.zr"')) {
  throw new Error("Feral Charge ZrVM project fixture is missing");
}

process.stdout.write(`WOS219 Feral Charge static guard passed (${SOURCE_COMMIT.slice(0, 15)})\n`);
