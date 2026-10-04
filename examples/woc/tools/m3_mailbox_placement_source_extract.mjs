// 从固定版本 WOC 源码中提取邮箱的有效世界坐标，供 m3_mailbox_placement_codegen.mjs 消费。
const mailboxes = await import('wocgit:///src/sim/content/mailboxes.ts');

if (!Array.isArray(mailboxes.MAILBOXES) || mailboxes.MAILBOXES.some(
  (entry) => !Number.isFinite(entry.x) || !Number.isFinite(entry.z),
)) {
  throw new Error('mailbox placement source shape drifted');
}

process.stdout.write(JSON.stringify({ entries: mailboxes.MAILBOXES }));
