import { resolveLayoutCatalogPath } from './zui-layout-paths';
import { readFile, writeFile } from 'node:fs/promises';
import { dirname, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import type { CatalogEntry, CatalogManifest } from './zui-layout-catalog';
import { currentDualReview, verifyCurrentFiles } from './zui-layout-evidence';

const root = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../../docs/_data/layout',
);
const manifest = JSON.parse(
  await readFile(resolve(root, 'catalog.json'), 'utf8'),
) as CatalogManifest;
const escape = (value: string) =>
  value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
const categories = [
  ...new Set(manifest.entries.map((entry) => entry.category)),
];
const repoRoot = resolve(process.env['ZUI_LAYOUT_REPO_ROOT'] ?? resolve(root, '../../..'));
async function renderEntry(entry: CatalogEntry): Promise<string> {
  let current = false;
  let staleReason = '';
  if (entry.penpotEvidence?.length || entry.engineEvidence?.length) {
    try {
      await verifyCurrentFiles(
        entry,
        repoRoot,
        root,
        entry.review?.status === 'accepted',
      );
      current = true;
    } catch (error) {
      staleReason = error instanceof Error ? error.message : String(error);
    }
  }
  const review = current && currentDualReview(entry) ? entry.review : undefined;
  const primaryId = entry.cases?.[0]?.id;
  const engine = current
    ? entry.engineEvidence?.find(
        (item) => item.caseId === primaryId && item.screenshotPath,
      )
    : undefined;
  const penpot = current
    ? entry.penpotEvidence?.find(
        (item) => item.caseId === primaryId && item.screenshotPath,
      )
    : undefined;
  const primary = engine ?? penpot;
  const label = engine ? 'Engine' : 'Penpot';
  const penpotCount = current
    ? (entry.penpotEvidence?.filter((item) => item.status === 'passed')
        .length ?? 0)
    : 0;
  const engineCount = current
    ? (entry.engineEvidence?.filter((item) => item.status === 'passed')
        .length ?? 0)
    : 0;
  const status =
    review?.status ??
    (entry.status === 'failed' || entry.visualStatus === 'failed'
      ? 'needs_revision'
      : 'pending');
  const message = staleReason
    ? '证据已过期，待复拍'
    : entry.status === 'failed'
      ? '投影失败'
      : '尚无当前截图';
  const preview = primary
    ? `<a class="preview" href="${escape(primary.screenshotPath)}"><img loading="lazy" src="${escape(primary.screenshotPath)}" alt="${escape(entry.name)}"><span>${label} · ${primary.status}</span></a>`
    : `<div class="preview empty" title="${escape(staleReason)}">${message}</div>`;
  return `<article data-category="${escape(entry.category)}" data-search="${escape(`${entry.name} ${entry.sourcePath}`.toLowerCase())}">
${preview}<h2>${escape(entry.name)}</h2><p>${escape(entry.sourcePath)}</p>
<p class="status ${status}">${status} · Penpot ${penpotCount}/${entry.cases?.length ?? 0} · Engine ${engineCount}/${entry.cases?.length ?? 0}</p>
<nav><a href="${escape(relative(root, resolveLayoutCatalogPath(root, entry.outputPath)).replaceAll('\\', '/'))}">ZUI</a><a href="${escape(entry.resultPath)}">Result</a>${engine ? `<a href="${escape(engine.screenshotPath)}">Engine</a>` : ''}${penpot ? `<a href="${escape(penpot.screenshotPath)}">Penpot</a>` : ''}</nav></article>`;
}
const entries: string[] = [];
for (const entry of manifest.entries) entries.push(await renderEntry(entry));
const html = `<!doctype html>
<html lang="zh-CN"><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Zircon UI Layouts</title><style>
*{box-sizing:border-box;letter-spacing:0}body{margin:0;background:#141617;color:#e9edec;font:14px/1.5 system-ui,sans-serif}
header{padding:20px 24px;border-bottom:1px solid #3a4042;display:flex;gap:24px;align-items:center;flex-wrap:wrap}
h1{font-size:22px;margin:0}input,select{font:inherit;color:inherit;background:#202426;border:1px solid #50585b;padding:8px 12px;border-radius:4px}
input{flex:1;min-width:180px}main{display:grid;grid-template-columns:repeat(auto-fill,minmax(min(440px,100%),1fr));gap:24px;padding:24px}
article{min-width:0;border-bottom:1px solid #3a4042;padding-bottom:20px}article[hidden]{display:none}.preview{display:flex;align-items:center;justify-content:center;background:#202426;aspect-ratio:16/10;overflow:hidden}
.preview{position:relative}.preview span{position:absolute;bottom:0;left:0;background:#171717;color:#ccc;padding:4px 8px;font-size:12px}.empty{color:#aaa}
img{max-width:100%;max-height:100%;object-fit:contain}h2{font-size:15px;margin:12px 0 4px;overflow-wrap:anywhere}p{margin:4px 0;color:#a6b0b3;font-size:12px;overflow-wrap:anywhere}
a{color:#84caff;text-decoration:none}a:hover{text-decoration:underline}nav{display:flex;gap:20px;margin-top:10px}.status{color:#e1bb6d}.accepted{color:#70d7ad}
</style><header><h1>Zircon UI Layouts</h1><span>${manifest.sourceCount}</span><input id="search" type="search" placeholder="Search layouts" aria-label="Search layouts"><select id="category" aria-label="Category"><option value="">All categories</option>${categories.map((category) => `<option>${escape(category)}</option>`).join('')}</select></header><main>${entries.join('')}</main><script>
const search=document.querySelector('#search'),category=document.querySelector('#category');
function filter(){for(const item of document.querySelectorAll('article'))item.hidden=!(item.dataset.search.includes(search.value.toLowerCase())&&(!category.value||item.dataset.category===category.value));}
search.addEventListener('input',filter);category.addEventListener('change',filter);
</script></html>`;
await writeFile(resolve(root, 'gallery.html'), html, 'utf8');
console.log(
  JSON.stringify({
    gallery: resolve(root, 'gallery.html'),
    total: manifest.sourceCount,
  }),
);
