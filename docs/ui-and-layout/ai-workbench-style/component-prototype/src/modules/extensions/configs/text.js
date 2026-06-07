export function titleWord(word) {
  const upper = word.toUpperCase();
  if (["AI", "UI", "UX", "VFX", "HUD", "DCC"].includes(upper)) {
    return upper;
  }
  return word.charAt(0).toUpperCase() + word.slice(1);
}
