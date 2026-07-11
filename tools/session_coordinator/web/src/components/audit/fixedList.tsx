import { useEffect, useMemo, useRef, useState, type ReactNode } from "react";

export function FixedSizeList<T>({ items, rowKey, render, label, follow = false, height = 420, rowHeight = 38 }: { items: T[]; rowKey: (item: T) => string; render: (item: T) => ReactNode; label: string; follow?: boolean; height?: number; rowHeight?: number }) {
  const viewport = useRef<HTMLDivElement>(null);
  const [scrollTop, setScrollTop] = useState(0);
  const overscan = 4;
  const start = Math.max(0, Math.floor(scrollTop / rowHeight) - overscan);
  const count = Math.ceil(height / rowHeight) + overscan * 2;
  const visible = useMemo(() => items.slice(start, start + count), [items, start, count]);
  useEffect(() => {
    if (follow && viewport.current) {
      const next = Math.max(0, items.length * rowHeight - height);
      viewport.current.scrollTop = next;
      setScrollTop(next);
    }
  }, [follow, height, items.length, rowHeight]);
  return <div ref={viewport} className="virtual-list" style={{ height }} onScroll={(event) => setScrollTop(event.currentTarget.scrollTop)} tabIndex={0} role="list" aria-label={`${label}，共 ${items.length} 行`}>
    <div style={{ height: items.length * rowHeight, position: "relative" }}>
      {visible.map((item, index) => <div className="virtual-row" style={{ position: "absolute", top: (start + index) * rowHeight, height: rowHeight, left: 0, right: 0 }} tabIndex={-1} role="listitem" key={rowKey(item)}>{render(item)}</div>)}
    </div>
  </div>;
}
