export function toggleDropdownPopupState(popup) {
  popup.classList.toggle("is-open");
}

export function closeDropdownPopupState(popup) {
  popup?.classList.remove("is-open");
}
