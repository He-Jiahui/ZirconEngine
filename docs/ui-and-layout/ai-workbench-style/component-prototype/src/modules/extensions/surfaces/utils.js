export const esc = (value) => String(value ?? "").replace(/[&<>"']/g, (char) => ({
  "&": "&amp;",
  "<": "&lt;",
  ">": "&gt;",
  '"': "&quot;",
  "'": "&#039;"
}[char]));

export function toRows(table, selectedIndex = 0) {
  return table.map((row, index) => ({
    cells: row,
    selected: index === selectedIndex
  }));
}

export function progressValue(row, index) {
  const raw = Number.parseFloat(row[2]);
  if (Number.isFinite(raw)) return raw;
  if (/done|passed|ready/i.test(String(row[1]))) return 100;
  if (/running|warning/i.test(String(row[1]))) return index === 0 ? 62 : 34;
  return 0;
}
