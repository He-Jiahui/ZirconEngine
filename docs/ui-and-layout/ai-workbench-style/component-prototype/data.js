export const sceneTree = [
  { id: "root", label: "Root", icon: "cube", expanded: true, locked: false, children: [
    { id: "environment", label: "Environment", icon: "cube", expanded: true, children: [
      { id: "lighting", label: "Lighting", icon: "sun" },
      { id: "sky", label: "Sky", icon: "sun" }
    ] },
    { id: "level", label: "Level", icon: "cube", expanded: true, children: [
      { id: "geometry", label: "Geometry", icon: "cube" },
      { id: "props", label: "Props", icon: "cube", selected: true }
    ] },
    { id: "player-start", label: "PlayerStart", icon: "cursor", collapsed: true },
    { id: "audio-zone", label: "AudioZone", icon: "audio", collapsed: true, locked: true }
  ] }
];

export const tableRows = [
  ["Item_01", "Mesh", "2.4 MB", "2m ago"],
  ["Item_02", "Material", "512 KB", "10m ago"],
  ["Item_03", "Texture", "1.2 MB", "1h ago"]
];

export const inspectorSections = [
  {
    title: "Transform",
    icon: "grid",
    checked: true,
    fields: [
      { label: "Position", values: ["128.4", "64.2", "-32.7"] },
      { label: "Rotation", values: ["0°", "90°", "0°"] },
      { label: "Scale", link: true, values: ["1.00", "1.00", "1.00"] }
    ]
  },
  {
    title: "Mesh Renderer",
    icon: "grid",
    checked: true,
    rows: [
      { label: "Mesh", value: "Box_01", icon: "cube" },
      { label: "Materials", value: "M_Metal", count: "1", swatch: true }
    ],
    nested: [
      ["Lighting", ""],
      ["Cast Shadows", "On"],
      ["Receive Shadows", "check"]
    ]
  }
];

export const alerts = [
  ["info", "Info Alert"],
  ["success", "Success Alert"],
  ["warning", "Warning Alert"],
  ["error", "Error Alert"]
];

export const listItems = [
  { label: "List item" },
  { label: "Selected item", selected: true },
  { label: "Disabled item", disabled: true }
];

export const menuItems = [
  ["New", "plus"],
  ["Open", "folder"],
  ["Save", "save"],
  ["Delete", "trash", "danger"],
  ["More Tools", "chevronRight"]
];
