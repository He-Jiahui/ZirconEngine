import type { LayoutReviewCaseMessage } from '../model';
import { parseZuiDocument } from './zui-document';
import { reviewDocumentForCase } from './zui-review-case';

const source = `[asset]
kind = "view"
id = "res://review/tabs.zui"
version = 2
[root]
node = "tabs"
[nodes.tabs]
component = "Tabs"
props = { options = ["overview", "details", "stats"], value = "overview", selected_index = 0, selection_state = "single" }
children = [{ node = "overview" }, { node = "details" }, { node = "stats" }]
[nodes.overview]
component = "Tab"
props = { text = "overview", selected = true }
[nodes.details]
component = "Tab"
props = { text = "details", selected = false }
[nodes.stats]
component = "Tab"
props = { text = "stats", selected = false }
`;

const review: LayoutReviewCaseMessage = {
  id: 'hover-240x520-dpi1',
  sourcePath: 'review/tabs.zui',
  host: 'component',
  viewport: { width: 240, height: 520 },
  dpi: 1,
  locale: 'en-US',
  state: 'hover',
  data: {},
};

describe('single-select tab review host', () => {
  it.each(['hover', 'pressed', 'focused', 'disabled'])(
    'targets only the details tab for %s while preserving the selected overview',
    (state) => {
      const document = parseZuiDocument(source).document;
      const original = structuredClone(document);
      const staged = reviewDocumentForCase(document, { ...review, state });
      expect(document).toEqual(original);
      expect(staged.nodes!.overview.state?.selected).toBe(true);
      expect(
        staged.nodes!.details.state?.[state === 'hover' ? 'hovered' : state],
      ).toBe(true);
      expect(
        staged.nodes!.stats.state?.[state === 'hover' ? 'hovered' : state],
      ).not.toBe(true);
      expect(
        staged.nodes!.tabs.state?.[state === 'hover' ? 'hovered' : state],
      ).not.toBe(true);
      expect(staged.nodes!.tabs.props?.['value']).toBe('overview');
    },
  );

  it('moves selection to details, updating the tab group value and index once', () => {
    const document = parseZuiDocument(source).document;
    const staged = reviewDocumentForCase(document, {
      ...review,
      id: 'selected-240x520-dpi1',
      state: 'selected',
    });
    expect(staged.nodes!.tabs.props?.['value']).toBe('details');
    expect(staged.nodes!.tabs.props?.['selected_index']).toBe(1);
    expect(staged.nodes!.overview.state?.selected).not.toBe(true);
    expect(staged.nodes!.details.state?.selected).toBe(true);
    expect(staged.nodes!.stats.state?.selected).not.toBe(true);
  });
});
