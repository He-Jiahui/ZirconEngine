// Extracts the authored Litany dais and island safe-ground geometry from the
// pinned WOC source. It intentionally reads the geometry source rather than
// rebuilding a second hand-maintained copy beside the hazard rules.

const litany = await import('wocgit:///src/sim/delve_litany_layout.ts');
const layouts = await import('wocgit:///src/sim/delve_layout.ts');

const layoutIds = Object.keys(layouts.DELVE_MODULE_LAYOUTS);
const modules = litany.LITANY_MODULE_IDS.map((id) => {
  const geometry = litany.litanyModuleGeometry(id);
  if (!geometry) throw new Error(`missing Litany geometry for ${id}`);
  const moduleIndex = layoutIds.indexOf(id);
  if (moduleIndex < 0) throw new Error(`Litany module ${id} is absent from Delve layout order`);
  return {
    id,
    module_index: moduleIndex,
    dais: {
      x: geometry.dais.x,
      z: geometry.dais.z,
      r: geometry.dais.r,
    },
    islands: geometry.islands.map((island) => ({
      x: island.x,
      z: island.z,
      hw: island.hw,
      hd: island.hd,
    })),
  };
});

process.stdout.write(JSON.stringify({ modules }));
