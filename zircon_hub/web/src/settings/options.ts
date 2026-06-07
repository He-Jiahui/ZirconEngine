import type { HubSettingsOptionText, HubSettingsText } from "../types/hub";

export function settingsOptionLabel(options: HubSettingsOptionText[], value: string): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

export function settingsJobCountLabel(text: HubSettingsText, jobs: number): string {
  const normalizedJobs = Number.isFinite(jobs) ? Math.max(1, Math.trunc(jobs)) : 1;
  const template = normalizedJobs === 1 ? text.jobCountSingularTemplate : text.jobCountPluralTemplate;
  return template.replace("{jobs}", `${normalizedJobs}`);
}
