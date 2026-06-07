export function blueprint(value) {
  return value;
}

export function tablePrimary(title, headers, rows, columns) {
  return { kind: "table", title, headers, rows, columns };
}

export function queuePrimary(title, headers, rows) {
  return { kind: "queue", title, headers, rows, columns: "1.2fr 0.8fr 1fr" };
}

export function timelinePrimary(title, headers, rows) {
  return { kind: "timeline", title, headers, rows, columns: "1fr 0.8fr 0.8fr" };
}

export function graphPrimary(title, nodes) {
  return { kind: "graph", title, nodes };
}

export function tree(root, glyph, children) {
  return [
    [root, "folder", false, 0],
    ...children.map((label, index) => [label, index === 0 ? glyph : "file", index === 0, index === 0 ? 1 : 2])
  ];
}

export function selectValue(value) {
  return { kind: "select", value };
}

export function inputValue(value) {
  return { kind: "input", value };
}

export function checkValue(value) {
  return { kind: "checkbox", value };
}
