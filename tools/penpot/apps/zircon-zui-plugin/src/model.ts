import type { ZuiDiagnostic } from './bridge/zui-document';
import type { PenpotFontResourceManifest } from './bridge/penpot-font-selection';
import type { RenderedLayoutAudit } from './penpot-render-layout';

export interface LayoutReviewCaseMessage {
  id: string;
  sourcePath: string;
  host:
    'editor' | 'plugin' | 'woc' | 'component' | 'toolbar' | 'theme' | 'fixture';
  viewport: { width: number; height: number };
  dpi: number;
  locale: string;
  state: string;
  /** Optional scroll offset combined with the interaction state for this scene. */
  scrollPosition?: 'start' | 'end';
  data: Record<string, unknown>;
  themeSourcePath?: string;
  reviewHost?: { path: string; sha256: string };
}

export type PluginUiMessage =
  | { type: 'ready' }
  | {
      type: 'import-zui';
      fileName: string;
      source: string;
      replaceSelection?: boolean;
      /**
       * Design-review-only materialization policy. Product imports keep the
       * default `full` policy; the visual harness may request `leaf` for a
       * very large retained shell so composite source mappings stay as
       * editable semantic boards while primitive instances remain native.
       */
      nativeComponentMode?: 'full' | 'leaf' | 'semantic';
      /** Keep the authored composite root as an editable local library main. */
      createComponentMaster?: boolean;
      fontResources?: PenpotFontResourceManifest;
    }
  | {
      /** Set by the design-only visual harness before the next import. */
      type: 'set-review-materialization';
      mode: 'full' | 'leaf' | 'semantic';
    }
  | { type: 'export-zui' }
  | { type: 'zop-load-contract'; source: string }
  | { type: 'zop-annotate'; source: string }
  | { type: 'zop-export'; screen: string; assetPath: string }
  | { type: 'export-canvas-png'; expectedBoardId: string }
  | { type: 'set-preview-selection'; selected: boolean }
  | { type: 'audit-preview-layout'; requestId?: string }
  | { type: 'set-review-case'; reviewCase: LayoutReviewCaseMessage }
  | { type: 'inspect-selection' };

export type PluginHostMessage =
  | { type: 'theme'; content: string }
  | { type: 'zop-export-ready'; fileName: string; bytes: Uint8Array }
  | {
      type: 'status';
      level: 'idle' | 'working' | 'success' | 'warning' | 'error';
      message: string;
      diagnostics?: ZuiDiagnostic[];
      previewBoardId?: string;
      layoutAudit?: RenderedLayoutAudit;
      reviewCase?: LayoutReviewCaseMessage;
    }
  | { type: 'selection'; canExport: boolean; label: string | null; boardId: string | null }
  | {
      type: 'canvas-png-ready';
      boardId: string;
      fileName: string;
      bytes: Uint8Array;
      width: number;
      height: number;
    }
  | {
      type: 'preview-audit';
      requestId?: string;
      previewBoardId: string;
      layoutAudit: RenderedLayoutAudit;
    }
  | {
      type: 'export-ready';
      fileName: string;
      source: string;
      diagnostics: ZuiDiagnostic[];
    };
