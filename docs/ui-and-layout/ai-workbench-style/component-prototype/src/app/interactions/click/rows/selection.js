export function selectExclusiveRows(selectedRow, rows) {
  rows.forEach((row) => row.classList.remove("is-selected"));
  selectedRow.classList.add("is-selected");
}
