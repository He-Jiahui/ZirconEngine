import type { HubSettingsOptionText, HubSettingsText } from "../types/hub";

// 草稿值用当前语言的选项文案解释；未知值保留原值，避免界面把未识别配置误显示为默认项。
export function settingsOptionLabel(options: HubSettingsOptionText[], value: string): string {
  return options.find((option) => option.value === value)?.label ?? value;
}

// 为尚未保存的并行数草稿生成单复数文案；显示归一化不替代保存时的设置校验，模板约定一个数量占位。
export function settingsJobCountLabel(text: HubSettingsText, jobs: number): string {
  const normalizedJobs = Number.isFinite(jobs) ? Math.max(1, Math.trunc(jobs)) : 1;
  const template = normalizedJobs === 1 ? text.jobCountSingularTemplate : text.jobCountPluralTemplate;
  return template.replace("{jobs}", `${normalizedJobs}`);
}
