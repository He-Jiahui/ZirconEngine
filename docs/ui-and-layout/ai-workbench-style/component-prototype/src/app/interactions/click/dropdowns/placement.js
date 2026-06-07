export function positionDropdownPopup(popup, dropdown) {
  const rect = dropdown.getBoundingClientRect();
  popup.style.left = `${Math.min(rect.left, window.innerWidth - 190)}px`;
  popup.style.top = `${rect.bottom + 6}px`;
}
