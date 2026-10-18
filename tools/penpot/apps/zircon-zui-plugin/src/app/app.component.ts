import { Component, DestroyRef, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { filter, fromEvent, map, merge, of } from 'rxjs';
import type { ZuiDiagnostic } from '../bridge/zui-document';
import { deferDownloadUrlRevocation } from './download-lifecycle';
import { parseFontResourceManifest, type PenpotFontResourceManifest } from '../bridge/penpot-font-selection';
import type {
  LayoutReviewCaseMessage,
  PluginHostMessage,
  PluginUiMessage,
} from '../model';
import type { RenderedLayoutAudit } from '../penpot-render-layout';
import { parseRoleContract } from '../../../zircon-zop-compiler/src/contract.mjs';

interface StatusState {
  level: 'idle' | 'working' | 'success' | 'warning' | 'error';
  message: string;
}

@Component({
  selector: 'app-root',
  template: `
    <header class="app-header">
      <div>
        <p class="eyebrow">ZirconEngine</p>
        <h1 class="title title-s">ZUI Asset Bridge</h1>
      </div>
      <span class="version body-s">v2</span>
    </header>

    <main>
      <nav role="tablist" aria-label="Authoring path">
        <button role="tab" type="button" [attr.aria-selected]="activeTab() === 'bridge'" (click)="activeTab.set('bridge')">ZUI round trip</button>
        <button role="tab" type="button" [attr.aria-selected]="activeTab() === 'annotate'" (click)="activeTab.set('annotate')">Annotate</button>
      </nav>
      <div class="bridge-tab" [hidden]="activeTab() !== 'bridge'">
      <section class="command-row" aria-label="ZUI asset commands">
        <input
          #fileInput
          class="visually-hidden"
          type="file"
          accept=".zui,text/plain"
          (change)="importFile($event)"
        />
        <button
          type="button"
          data-appearance="primary"
          [disabled]="busy()"
          (click)="fileInput.click()"
        >
          Import .zui
        </button>
        <button
          type="button"
          data-appearance="secondary"
          [disabled]="busy() || !canExport()"
          (click)="exportSelected()"
        >
          Export selected
        </button>
        <button
          type="button"
          data-appearance="secondary"
          [disabled]="busy() || !canExport() || !selectedBoardId()"
          (click)="exportCanvasPng()"
        >
          Export Canvas PNG
        </button>
      </section>

      <section class="command-row" aria-label="Optional font resources">
        <input #fontResourceInput class="visually-hidden" type="file" accept=".json,application/json" data-font-resource-manifest (change)="loadFontResources($event)" />
        <button type="button" data-appearance="secondary" [disabled]="busy()" (click)="fontResourceInput.click()">字体资源清单（可选）</button>
        @if (fontResourceName()) {
          <span class="body-s">{{ fontResourceName() }}</span>
          <button type="button" [disabled]="busy()" (click)="clearFontResources()">清除</button>
        }
      </section>
      </div>
      <section [hidden]="activeTab() !== 'annotate'" aria-label="Operator annotations">
        <p class="body-s">Load the reviewed role contract, then annotate a selected layer. ZOP Export reads fresh Penpot boards and active editor.* library tokens.</p>
        <input #zopContractInput class="visually-hidden" type="file" accept=".toml,text/plain" (change)="loadZopContract($event)" />
        <button type="button" [disabled]="busy()" (click)="zopContractInput.click()">Load roles.contract.toml</button>
        <label for="zop-binding">Role binding JSON</label>
        <textarea #zopBinding id="zop-binding" rows="6" spellcheck="false">{{ '{}' }}</textarea>
        <button type="button" [disabled]="busy() || !zopContractLoaded()" (click)="annotateOperator(zopBinding.value)">Apply annotation</button>
        <label for="zop-screen">Screen</label>
        <select #zopScreen id="zop-screen">
          <option value="project">Project Browser</option>
          <option value="level">Level Editor</option>
          <option value="settings">Settings</option>
        </select>
        <button type="button" [disabled]="busy() || !zopContractLoaded()" (click)="exportOperator(zopScreen.value)">ZOP Export</button>
      </section>

      <section
        class="selection-band"
        [hidden]="activeTab() !== 'bridge'"
        [class.ready]="canExport()"
        [attr.data-selected-board-id]="selectedBoardId()"
        [attr.data-canvas-png-board-id]="canvasPngBoardId()"
      >
        <span class="selection-dot" aria-hidden="true"></span>
        <div>
          <p class="label body-s">Selection</p>
          <p class="selection-name body-m">
            {{ selectionLabel() || 'No ZUI asset selected' }}
          </p>
        </div>
      </section>

      <section
        class="status-panel"
        [attr.data-level]="status().level"
        [attr.data-preview-board-id]="previewBoardId()"
        [attr.data-layout-audit-request-id]="layoutAuditRequestId()"
        [attr.data-preview-x]="layoutAudit()?.assetBounds?.x"
        [attr.data-preview-y]="layoutAudit()?.assetBounds?.y"
        [attr.data-preview-width]="layoutAudit()?.assetBounds?.width"
        [attr.data-preview-height]="layoutAudit()?.assetBounds?.height"
        [attr.data-layout-total]="layoutAudit()?.totalNodes ?? null"
        [attr.data-layout-checked]="layoutAudit()?.checkedNodes ?? null"
        [attr.data-layout-overflow]="layoutAudit()?.overflowCount ?? null"
        [attr.data-layout-invalid]="layoutAudit()?.invalidGeometryCount ?? null"
        [attr.data-layout-overflow-nodes]="
          layoutAudit()?.overflowNodeIds?.join(',')
        "
        [attr.data-layout-invalid-nodes]="
          layoutAudit()?.invalidGeometryNodeIds?.join(',')
        "
        [attr.data-layout-overflow-details]="layoutOverflowDetails()"
        [attr.data-layout-invalid-details]="layoutInvalidDetails()"
        [attr.data-layout-semantic-nodes]="layoutSemanticNodes()"
        [attr.data-review-case-id]="reviewCase()?.id ?? null"
        [attr.data-review-case-state]="reviewCase()?.state ?? null"
        [attr.data-review-case-dpi]="reviewCase()?.dpi ?? null"
        aria-live="polite"
      >
        <div class="status-heading">
          <span class="status-mark" aria-hidden="true"></span>
          <p class="body-m">{{ status().message }}</p>
        </div>
        @if (diagnostics().length > 0) {
          <div class="diagnostic-summary body-s">
            <span
              >{{ warningCount() }}
              {{ warningCount() === 1 ? 'warning' : 'warnings' }}</span
            >
            <span>
              {{ preservedCount() }} preserved
              {{ preservedCount() === 1 ? 'notice' : 'notices' }}
            </span>
          </div>
          <ul class="diagnostics body-s">
            @for (
              diagnostic of diagnostics().slice(0, 5);
              track diagnostic.code + diagnostic.path
            ) {
              <li [attr.data-severity]="diagnostic.severity">
                {{ diagnostic.message }}
              </li>
            }
          </ul>
        }
      </section>
    </main>

    <footer class="body-s">
      <strong>ZUI v2</strong>
      <span class="separator" aria-hidden="true"></span>
      <span>{{ activeTab() === 'annotate' ? 'dev.zircon.zop' : 'dev.zircon.zui' }}</span>
    </footer>
  `,
  styleUrl: './app.component.css',
  host: { '[attr.data-theme]': 'theme()' },
})
export class AppComponent {
  private readonly destroyRef = inject(DestroyRef);
  private readonly messages$ = fromEvent<MessageEvent<PluginHostMessage>>(
    window,
    'message',
  ).pipe(
    filter(({ source }) => source === parent),
    map(({ data }) => data),
    filter(
      (message): message is PluginHostMessage =>
        typeof message?.type === 'string',
    ),
  );

  readonly theme = signal(
    new URLSearchParams(window.location.search).get('theme') ?? 'dark',
  );
  readonly status = signal<StatusState>({ level: 'idle', message: 'Ready' });
  readonly diagnostics = signal<ZuiDiagnostic[]>([]);
  readonly canExport = signal(false);
  readonly selectionLabel = signal<string | null>(null);
  readonly selectedBoardId = signal<string | null>(null);
  readonly canvasPngBoardId = signal<string | null>(null);
  readonly busy = signal(false);
  readonly previewBoardId = signal<string | null>(null);
  readonly layoutAudit = signal<RenderedLayoutAudit | null>(null);
  readonly layoutAuditRequestId = signal<string | null>(null);
  readonly reviewCase = signal<LayoutReviewCaseMessage | null>(null);
  readonly fontResourceName = signal<string | null>(null);
  readonly activeTab = signal<'bridge' | 'annotate'>('bridge');
  readonly zopContractLoaded = signal(false);
  private fontResources?: PenpotFontResourceManifest;

  constructor() {
    merge(
      of<PluginHostMessage>({ type: 'theme', content: this.theme() }),
      this.messages$,
    )
      .pipe(takeUntilDestroyed(this.destroyRef))
      .subscribe((message) => this.handleMessage(message));
    this.sendMessage({ type: 'ready' });
  }

  warningCount(): number {
    return this.diagnostics().filter(({ severity }) => severity === 'warning')
      .length;
  }

  preservedCount(): number {
    return this.diagnostics().filter(({ severity }) => severity === 'info')
      .length;
  }

  layoutOverflowDetails(): string {
    return JSON.stringify(this.layoutAudit()?.overflowDetails ?? []);
  }

  layoutInvalidDetails(): string {
    return JSON.stringify(this.layoutAudit()?.invalidGeometryDetails ?? []);
  }

  layoutSemanticNodes(): string {
    return JSON.stringify(this.layoutAudit()?.semanticNodes ?? []);
  }

  clearFontResources(): void {
    this.fontResources = undefined;
    this.fontResourceName.set(null);
  }

  loadZopContract(event: Event): void {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    this.busy.set(true);
    this.zopContractLoaded.set(false);
    void file.text().then(source => {
      parseRoleContract(source);
      this.sendMessage({ type: 'zop-load-contract', source });
      this.zopContractLoaded.set(true);
    }).catch((error: unknown) => {
      this.busy.set(false);
      this.status.set({ level: 'error', message: error instanceof Error ? error.message : String(error) });
    });
  }

  annotateOperator(source: string): void {
    this.busy.set(true);
    this.sendMessage({ type: 'zop-annotate', source });
  }

  exportOperator(screen: string): void {
    const names: Record<string, string> = { project: 'project_browser', level: 'level_editor', settings: 'settings_window' };
    if (!names[screen]) return;
    this.busy.set(true);
    this.sendMessage({ type: 'zop-export', screen, assetPath: `shell/${names[screen]}.zui` });
  }

  loadFontResources(event: Event): void {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    this.clearFontResources();
    this.busy.set(true);
    void file.text().then((contents) => {
      this.fontResources = parseFontResourceManifest(JSON.parse(contents));
      this.fontResourceName.set(file.name);
      this.status.set({level:'idle',message:`字体资源清单 · ${file.name}`});
    }).catch((error: unknown) => {
      this.status.set({level:'error',message:error instanceof Error ? error.message : String(error)});
    }).finally(() => this.busy.set(false));
  }

  importFile(event: Event): void {
    const input = event.target as HTMLInputElement;
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    if (!file.name.toLowerCase().endsWith('.zui')) {
      this.status.set({ level: 'error', message: 'Choose a .zui file' });
      return;
    }
    this.busy.set(true);
    this.previewBoardId.set(null);
    this.layoutAudit.set(null);
    this.layoutAuditRequestId.set(null);
    this.reviewCase.set(null);
    this.status.set({ level: 'working', message: `Reading ${file.name}` });
    void file
      .text()
      .then((source) =>
        this.sendMessage({
          type: 'import-zui',
          fileName: file.name,
          source,
          replaceSelection: true,
          ...(this.fontResources ? {fontResources:this.fontResources} : {}),
        }),
      )
      .catch((error: unknown) => {
        this.busy.set(false);
        this.status.set({
          level: 'error',
          message: error instanceof Error ? error.message : String(error),
        });
      });
  }

  exportSelected(): void {
    this.busy.set(true);
    this.sendMessage({ type: 'export-zui' });
  }

  exportCanvasPng(): void {
    const expectedBoardId = this.selectedBoardId();
    if (!expectedBoardId) return;
    this.canvasPngBoardId.set(null);
    this.busy.set(true);
    this.sendMessage({ type: 'export-canvas-png', expectedBoardId });
  }

  private handleMessage(message: PluginHostMessage): void {
    if (message.type === 'zop-export-ready') {
      const bytes = new Uint8Array(message.bytes);
      const url = URL.createObjectURL(new Blob([bytes.buffer as ArrayBuffer], { type: 'application/zip' }));
      const link = document.createElement('a');
      link.href = url;
      link.download = message.fileName;
      document.body.appendChild(link);
      link.click();
      link.remove();
      deferDownloadUrlRevocation(url);
      this.busy.set(false);
      return;
    }
    if (message.type === 'theme') {
      this.theme.set(message.content);
      return;
    }
    if (message.type === 'selection') {
      this.canExport.set(message.canExport);
      this.selectionLabel.set(message.label);
      this.selectedBoardId.set(message.boardId);
      return;
    }
    if (message.type === 'status') {
      this.status.set({ level: message.level, message: message.message });
      this.diagnostics.set(message.diagnostics ?? []);
      this.busy.set(message.level === 'working');
      if (message.previewBoardId !== undefined) {
        this.previewBoardId.set(message.previewBoardId);
      }
      if (message.layoutAudit !== undefined) {
        this.layoutAudit.set(message.layoutAudit);
        this.layoutAuditRequestId.set(null);
      }
      if (message.reviewCase !== undefined) {
        this.reviewCase.set(message.reviewCase);
      }
      return;
    }
    if (message.type === 'preview-audit') {
      this.previewBoardId.set(message.previewBoardId);
      this.layoutAudit.set(message.layoutAudit);
      this.layoutAuditRequestId.set(message.requestId ?? null);
      return;
    }
    if (message.type === 'canvas-png-ready') {
      if (message.boardId !== this.selectedBoardId()) {
        this.busy.set(false);
        this.status.set({ level: 'error', message: 'Canvas PNG belongs to a different selected Board.' });
        return;
      }
      this.downloadCanvasPng(message.fileName, message.bytes, message.width, message.height, message.boardId);
      return;
    }
    this.download(message.fileName, message.source);
  }

  private downloadCanvasPng(fileName: string, bytes: Uint8Array, width: number, height: number, boardId: string): void {
    if (!(bytes instanceof Uint8Array) || bytes.byteLength < 24) {
      this.busy.set(false);
      this.status.set({ level: 'error', message: 'Canvas PNG message did not contain PNG bytes.' });
      return;
    }
    const ownedBytes = bytes.slice();
    const url = URL.createObjectURL(
      new Blob([ownedBytes.buffer as ArrayBuffer], { type: 'image/png' }),
    );
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    link.remove();
    this.canvasPngBoardId.set(boardId);
    this.busy.set(false);
    this.status.set({ level: 'success', message: fileName + ' · ' + width + '×' + height });
    deferDownloadUrlRevocation(url);
  }

  private download(fileName: string, source: string): void {
    const url = URL.createObjectURL(
      new Blob([source], { type: 'text/plain;charset=utf-8' }),
    );
    const link = document.createElement('a');
    link.href = url;
    link.download = fileName;
    document.body.appendChild(link);
    link.click();
    link.remove();
    deferDownloadUrlRevocation(url);
  }

  private sendMessage(message: PluginUiMessage): void {
    parent.postMessage(message, '*');
  }
}
