import { Component, DestroyRef, inject, signal } from '@angular/core';
import { takeUntilDestroyed } from '@angular/core/rxjs-interop';
import { filter, fromEvent, map, merge, of } from 'rxjs';
import type { ZuiDiagnostic } from '../bridge/zui-document';
import { deferDownloadUrlRevocation } from './download-lifecycle';
import type {
  LayoutReviewCaseMessage,
  PluginHostMessage,
  PluginUiMessage,
} from '../model';
import type { RenderedLayoutAudit } from '../penpot-render-layout';

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
      </section>

      <section class="selection-band" [class.ready]="canExport()">
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
      <span>dev.zircon.zui</span>
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
  readonly busy = signal(false);
  readonly previewBoardId = signal<string | null>(null);
  readonly layoutAudit = signal<RenderedLayoutAudit | null>(null);
  readonly layoutAuditRequestId = signal<string | null>(null);
  readonly reviewCase = signal<LayoutReviewCaseMessage | null>(null);

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

  private handleMessage(message: PluginHostMessage): void {
    if (message.type === 'theme') {
      this.theme.set(message.content);
      return;
    }
    if (message.type === 'selection') {
      this.canExport.set(message.canExport);
      this.selectionLabel.set(message.label);
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
    this.download(message.fileName, message.source);
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
