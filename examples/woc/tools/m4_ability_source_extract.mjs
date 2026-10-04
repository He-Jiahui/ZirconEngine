// 从固定版本 WOC 源码中提取由一致性场景选取及离线回放保留的技能定义，供 m4_ability_codegen.mjs 消费。
const [sourceModule, ...abilityIds] = process.argv.slice(2);
if (!sourceModule || abilityIds.length === 0) {
  throw new Error('usage: m4_ability_source_extract.mjs <source-module> <ability-id>...');
}

const { ABILITIES } = await import(sourceModule);
// 保持调用方传入的技能 ID 顺序，生成器据此匹配返回的各项定义。
const definitions = abilityIds.map((id) => {
  const definition = ABILITIES[id];
  if (!definition) throw new Error(`M4 ability ${id} is missing from ABILITIES`);
  return definition;
});
process.stdout.write(JSON.stringify(definitions));
