// Печать: страница собирается в #print-root из графики вкладки или текста
// имиджа и отдаётся системному окну печати браузера (см. PrintDialog).
import { useStore } from './store';

export type Source = 'scheme' | 'picture' | 'icon' | 'model' | 'code';
type Scale = 'natural' | 'page' | 'width' | 'custom';
export interface PrintOptions {
  source: Source; scale: Scale; percent: number; paper: 'A4' | 'A3' | 'A5' | 'Letter'; landscape: boolean;
  margins: [number, number, number, number]; // левое, верхнее, правое, нижнее, мм
  whole: boolean; rect: [number, number, number, number]; // исходный прямоугольник: x0, y0, x1, y1
  header: boolean; lineNumbers: boolean; mono: boolean; window: number;
}
const defaults: PrintOptions = {
  source: 'scheme', scale: 'page', percent: 100, paper: 'A4', landscape: true, margins: [15, 15, 15, 15],
  whole: true, rect: [0, 0, 640, 480], header: true, lineNumbers: true, mono: false, window: 0,
};
export const PAPER: Record<PrintOptions['paper'], [number, number]> = { A4: [210, 297], A3: [297, 420], A5: [148, 210], Letter: [216, 279] };

export function loadPrint(): PrintOptions {
  try { return { ...defaults, ...JSON.parse(localStorage.getItem('print') ?? '{}') as Partial<PrintOptions> }; } catch { return defaults; }
}

const esc = (t: string) => t.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');

/// Готовая к печати графика: содержимое SVG и его прямоугольник в координатах листа.
interface Drawing { inner: string; box: [number, number, number, number]; className?: string }

/// Графика вкладки «Схема» — берётся с холста как есть (блоки, связи, площадки).
export function schemeDrawing(): Drawing | null {
  const svg = document.querySelector('.scheme svg.canvas');
  const scene = svg?.querySelector(':scope > g') as SVGGElement | null;
  if (!svg || !scene) return null;
  const b = scene.getBBox();
  const clone = scene.cloneNode(true) as SVGGElement;
  clone.removeAttribute('transform');
  const defs = svg.querySelector('defs')?.outerHTML ?? '';
  return { inner: defs + clone.outerHTML, box: [b.x - 8, b.y - 8, b.width + 16, b.height + 16], className: 'scheme' };
}

/// Рисунок или иконка имиджа — из ядра, в координатах пространства.
export async function pictureDrawing(cls: string, kind: 'image' | 'icon'): Promise<Drawing | null> {
  const r = await fetch(`/api/picture/${encodeURIComponent(cls)}?kind=${kind}`);
  if (!r.ok) return null;
  const j = await r.json() as { svg: string; view: [number, number, number, number] };
  const inner = j.svg.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '');
  // внутри уже сдвиг на начало вида: координаты листа = координаты SVG + view.xy
  return { inner: `<g transform="translate(${j.view[0]} ${j.view[1]})">${inner}</g>`, box: j.view };
}

export function modelDrawing(index: number): Drawing | null {
  const w = useStore.getState().frame?.windows[index];
  if (!w) return null;
  const inner = w.svg.replace(/^[\s\S]*?<svg[^>]*>/, '').replace(/<\/svg>\s*$/, '');
  return { inner, box: [0, 0, w.w, w.h] };
}

/// Собрать страницу печати и открыть системное окно печати.
export async function printWith(o: PrintOptions) {
  const s = useStore.getState();
  const cls = s.selectedClass ?? s.project?.root ?? '';
  const [pw, ph] = o.landscape ? [PAPER[o.paper][1], PAPER[o.paper][0]] : PAPER[o.paper];
  const [ml, mt, mr, mb] = o.margins;
  const areaW = pw - ml - mr, areaH = ph - mt - mb - (o.header ? 8 : 0);
  const title = { scheme: 'Схема', picture: 'Рисунок', icon: 'Иконка', model: 'Окно модели', code: 'Текст' }[o.source];
  const header = o.header ? `<div class="print-header"><b>${esc(title)} · ${esc(o.source === 'model' ? s.frame?.windows[o.window]?.name ?? '' : cls)}</b><span>${esc(s.project?.dir ?? '')}</span><span>${new Date().toLocaleString('ru-RU')}</span></div>` : '';
  let body = '';
  if (o.source === 'code') {
    const text = s.project?.classes.find(c => c.name === cls)?.text ?? '';
    const lines = text.split('\n');
    const w = String(lines.length).length;
    body = `<pre class="print-code">${lines.map((l, i) => (o.lineNumbers ? `<span class="ln">${String(i + 1).padStart(w)}</span>` : '') + esc(l)).join('\n')}</pre>`;
  } else {
    if (o.source === 'scheme' && s.tab !== 'scheme') { s.setTab('scheme'); await new Promise(r => setTimeout(r, 500)); }
    const d = o.source === 'scheme' ? schemeDrawing() : o.source === 'model' ? modelDrawing(o.window) : await pictureDrawing(cls, o.source === 'icon' ? 'icon' : 'image');
    if (!d) { s.showToast('Нечего печатать'); return; }
    const [x, y, w, h] = o.whole ? d.box : [o.rect[0], o.rect[1], Math.max(1, o.rect[2] - o.rect[0]), Math.max(1, o.rect[3] - o.rect[1])];
    // размер на бумаге: 1 единица листа = 1 пиксель экрана (1/96 дюйма)
    const mm = 25.4 / 96;
    const k = o.scale === 'natural' ? 1 : o.scale === 'custom' ? o.percent / 100
      : o.scale === 'width' ? areaW / (w * mm) : Math.min(areaW / (w * mm), areaH / (h * mm));
    body = `<div class="${d.className ?? ''} print-drawing"><svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="${x} ${y} ${w} ${h}" style="width:${(w * mm * k).toFixed(2)}mm;height:${(h * mm * k).toFixed(2)}mm">${d.inner}</svg></div>`;
  }
  const root = document.createElement('div');
  root.id = 'print-root';
  root.className = o.mono ? 'mono-print' : '';
  root.innerHTML = header + body;
  const page = document.createElement('style');
  page.textContent = `@page { size: ${pw}mm ${ph}mm; margin: ${mt}mm ${mr}mm ${mb}mm ${ml}mm; }`;
  const theme = document.documentElement.dataset.theme;
  document.documentElement.dataset.theme = 'light';
  document.head.appendChild(page);
  document.body.appendChild(root);
  document.body.classList.add('printing');
  const done = () => {
    document.body.classList.remove('printing');
    root.remove(); page.remove();
    if (theme) document.documentElement.dataset.theme = theme;
    window.removeEventListener('afterprint', done);
  };
  window.addEventListener('afterprint', done);
  // браузер без afterprint (или печать отменена сразу) — убрать страницу позже
  setTimeout(() => { window.print(); setTimeout(done, 1000); }, 50);
}

