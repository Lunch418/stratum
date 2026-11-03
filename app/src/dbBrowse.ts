// Запросы DbBrowse из кадра ядра: каждый новый запрос открывает окно DBF один раз.
import type { Frame } from './api';
import { useStore } from './store';

const seen = new Set<string>();

export function handleBrowse(reqs: Frame['browse']) {
  if (!reqs || reqs.length === 0) { seen.clear(); return; }
  const live = new Set<string>();
  for (const r of reqs) {
    const key = JSON.stringify([r.window, r.file, r.columns]);
    live.add(key);
    if (seen.has(key)) continue;
    seen.add(key);
    useStore.getState().openDbf(r.file);
  }
  for (const k of seen) if (!live.has(k)) seen.delete(k);
}
