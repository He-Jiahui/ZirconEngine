export function assetsFor(subject, category, glyph) {
  return [
    [category, "folder", false, 0],
    [`${subject} Root`, glyph, true, 1],
    [`${subject} Preset A`, "file", false, 2],
    [`${subject} Preset B`, "file", false, 2],
    ["Shared References", "folder", false, 1],
    ["Workbench Style", "material", false, 2]
  ];
}
