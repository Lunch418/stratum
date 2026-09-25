// Холст схемы: графика листа (SVG из ядра), блоки имиджей, связи;
// панорама, зум к курсору, перетаскивание блоков, вход в подсхему.
// Правка: перетаскивание имиджа из иерархии добавляет экземпляр, тяга от
// порта к блоку создаёт связь, контекстное меню и Delete удаляют.
// Контактные площадки — точки, через которые связи идут к переменным самого
// имиджа схемы («Вставка → Контактная площадка»).
import { useWindowEvent } from '../hooks';
import { useEffect, useMemo, useRef, useState } from 'react';
import { api, type ClassInfo } from '../api';
import { classByName, useStore } from '../store';
import { LinkDialog } from './LinkDialog';
import type { LinkStyle } from '../api';

export const DRAG_CLASS = 'application/x-stratum-class';

const NODE_W = 120;
const NODE_H = 36;
const ICON = 32;
/// handle «самого» имиджа схемы в связях
const SELF = 0;
/// размер контактной площадки (в оригинале — растр 32×32, x/y — левый верх)
const PAD = 32;

interface View { x: number; y: number; k: number }
interface NodeLike { handle: number; class: string; name: string; x: number; y: number }

function nodeSize(child: { name: string; class: string }) {
  const label = child.name || child.class;
  return { w: Math.max(NODE_W, 44 + label.length * 7), h: NODE_H, label };
}

export function SchemeCanvas() {
  const project = useStore(s => s.project);
  const path = useStore(s => s.schemePath);
  const selectedClass = useStore(s => s.selectedClass);
  const select = useStore(s => s.select);
  const enterScheme = useStore(s => s.enterScheme);
  const updateClass = useStore(s => s.updateClass);
  const showToast = useStore(s => s.showToast);
  const reload = useStore(s => s.reload);
  const setTab = useStore(s => s.setTab);
  const say = useStore(s => s.say);
  const clipboard = useStore(s => s.schemeClipboard);
  const setClipboard = useStore(s => s.setSchemeClipboard);
  const klass = classByName(project, path[path.length - 1]);
  const editable = !!klass && !klass.library;
  const [view, setView] = useState<View>({ x: 0, y: 0, k: 1 });
  const [background, setBackground] = useState<{ svg: string; bounds: { x: number; y: number; w: number; h: number } | null }>({ svg: '', bounds: null });
  const [drag, setDrag] = useState<{ handle: number; dx: number; dy: number; moved: boolean } | null>(null);
  const [pan, setPan] = useState<{ sx: number; sy: number; vx: number; vy: number } | null>(null);
  // тяга связи; fromPad — от контактной площадки (со стороны самого имиджа)
  const [wire, setWire] = useState<{ from: number; fromPad?: number; x: number; y: number } | null>(null);
  const [selPad, setSelPad] = useState<number | null>(null);
  const [padDrag, setPadDrag] = useState<{ id: number; dx: number; dy: number; moved: boolean } | null>(null);
  const [padMenu, setPadMenu] = useState<{ x: number; y: number; id: number } | null>(null);
  const [padProps, setPadProps] = useState<number | null>(null);
  // «Вставка → Контактная площадка»: следующий щелчок по листу ставит площадку
  const [placingPad, setPlacingPad] = useState(false);
  const [selNode, setSelNode] = useState<number | null>(null);
  // групповое выделение: Shift+щелчок и рамка на пустом месте
  const [selSet, setSelSet] = useState<Set<number>>(new Set());
  const [band, setBand] = useState<{ x0: number; y0: number; x1: number; y1: number } | null>(null);
  const [selLink, setSelLink] = useState<number | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; handle: number } | null>(null);
  const [renaming, setRenaming] = useState<{ handle: number; value: string } | null>(null);
  const [linkEdit, setLinkEdit] = useState<{ handle: number; source: number; target: number; pairs: [string, string][]; style?: LinkStyle; pad?: number } | null>(null);
  const [linkMenu, setLinkMenu] = useState<{ x: number; y: number; handle: number } | null>(null);
  const [sheetMenu, setSheetMenu] = useState<{ x: number; y: number } | null>(null);
  const [replacing, setReplacing] = useState<{ handle: number; value: string } | null>(null);
  const [merging, setMerging] = useState<{ picked: Set<number>; name: string } | null>(null);
  const layers = useStore(s => s.layers);
  const setDialog = useStore(s => s.setDialog);
  const sheet = klass?.sheet;
  const gridStep: [number, number] = sheet?.gridVisible ? sheet.gridStep : [16, 16];
  const gridOrigin: [number, number] = sheet?.gridVisible ? sheet.gridOrigin : [0, 0];
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!klass) return;
    api.scheme(klass.name).then(setBackground).catch(() => setBackground({ svg: '', bounds: null }));
  }, [klass?.name]);

  // показать всё при смене схемы
  useEffect(() => {
    if (!klass || !svgRef.current) return;
    fitAll();
    setSelNode(null); setSelLink(null); setMenu(null);
  }, [klass?.name, background.bounds?.w]);

  // подгонка: за основу — графика листа (или медианный блок), к ней
  // добавляются блоки неподалёку; «припаркованные» за тысячи пикселей
  // экземпляры (в корпусе встречаются x = −9000) в кадр не тянем
  const bounds = useMemo(() => {
    if (!klass) return null;
    const boxes = klass.children.map(c => { const { w, h } = nodeSize(c); return { x0: c.x, y0: c.y, x1: c.x + w, y1: c.y + h }; });
    let core: { x0: number; y0: number; x1: number; y1: number } | null = null;
    if (background.bounds) {
      const b = background.bounds;
      core = { x0: b.x, y0: b.y, x1: b.x + b.w, y1: b.y + b.h };
    } else if (boxes.length) {
      const med = (a: number[]) => { const s = [...a].sort((p, q) => p - q); return s[Math.floor(s.length / 2)]; };
      const mx = med(boxes.map(b => b.x0)), my = med(boxes.map(b => b.y0));
      core = { x0: mx, y0: my, x1: mx + NODE_W, y1: my + NODE_H };
    }
    if (!core) return null;
    const reach = Math.max(core.x1 - core.x0, core.y1 - core.y0, 600) * 1.5;
    const r = { ...core };
    for (const b of boxes) {
      if (b.x1 < core.x0 - reach || b.x0 > core.x1 + reach || b.y1 < core.y0 - reach || b.y0 > core.y1 + reach) continue;
      r.x0 = Math.min(r.x0, b.x0); r.y0 = Math.min(r.y0, b.y0); r.x1 = Math.max(r.x1, b.x1); r.y1 = Math.max(r.y1, b.y1);
    }
    return r;
  }, [klass, background.bounds]);

  // «сам» имидж схемы — виртуальный блок слева от детей: к нему идут связи с handle 0
  // (если у всех таких связей есть контактная площадка, блок не нужен)
  const selfNode: NodeLike | null = useMemo(() => {
    if (!klass) return null;
    const pads = new Set((klass.pads ?? []).map(p => p.id));
    if (!klass.links.some(l => (l.source === SELF || l.target === SELF) && !pads.has(l.pad))) return null;
    const b = bounds ?? { x0: 0, y0: 0, x1: 0, y1: 0 };
    return { handle: SELF, class: klass.name, name: '', x: b.x0 - nodeSize({ name: '', class: klass.name }).w - 60, y: b.y0 };
  }, [klass, bounds?.x0, bounds?.y0]);

  useEffect(() => {
    const onInsert = () => { if (editable) { setPlacingPad(true); showToast('Щёлкните на схеме, где поставить контактную площадку'); } };
    window.addEventListener('insert-pad', onInsert);
    return () => window.removeEventListener('insert-pad', onInsert);
  }, [editable, showToast]);

  // Delete / Escape
  useWindowEvent('keydown', (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('input, textarea, select, .monaco-editor')) return;
      if (e.key === 'Escape') { setMenu(null); setWire(null); setRenaming(null); setPlacingPad(false); setPadMenu(null); }
      if ((e.key === 'Delete' || e.key === 'Backspace') && editable && klass) {
        if (selPad !== null) { e.preventDefault(); removePad(selPad); }
        else if (selSet.size > 1) { e.preventDefault(); Promise.all([...selSet].filter(h => h !== SELF).map(h => api.removeChild(klass.name, h))).then(async () => { setSelSet(new Set()); setSelNode(null); useStore.getState().markUnsaved(); await useStore.getState().reload(); showToast('Блоки удалены'); }); }
        else if (selNode !== null && selNode !== SELF) { e.preventDefault(); removeChild(selNode); }
        else if (selLink !== null) { e.preventDefault(); removeLink(selLink); }
      }
      // буфер обмена схемы: копировать/вырезать/вставить/дублировать блок
      if (e.ctrlKey && klass && selNode !== null && selNode !== SELF && ['c', 'x', 'd'].includes(e.key.toLowerCase())) {
        const group = selSet.size ? selSet : new Set([selNode]);
        const items = klass.children.filter(x => group.has(x.handle)).map(c => ({ class: c.class, name: c.name, x: c.x, y: c.y }));
        if (!items.length) return;
        e.preventDefault();
        if (e.key.toLowerCase() === 'd') { if (editable) pasteBlocks(items); return; }
        setClipboard(items);
        showToast((e.key.toLowerCase() === 'x' ? 'Вырезано' : 'Скопировано') + (items.length > 1 ? `: ${items.length}` : ''));
        if (e.key.toLowerCase() === 'x' && editable) Promise.all([...group].map(h => api.removeChild(klass.name, h))).then(async () => { setSelSet(new Set()); setSelNode(null); useStore.getState().markUnsaved(); await useStore.getState().reload(); });
      } else if (e.ctrlKey && e.key.toLowerCase() === 'v' && editable && clipboard.length) {
        e.preventDefault();
        pasteBlocks(clipboard);
      }
  });

  function fitAll() {
    const el = svgRef.current;
    if (!el || !bounds) return;
    const { width, height } = el.getBoundingClientRect();
    const bw = Math.max(bounds.x1 - bounds.x0, 1), bh = Math.max(bounds.y1 - bounds.y0, 1);
    const k = Math.min(width / (bw + 80), height / (bh + 80), 4);
    setView({ k, x: (width - bw * k) / 2 - bounds.x0 * k, y: (height - bh * k) / 2 - bounds.y0 * k });
  }

  function toScene(e: { clientX: number; clientY: number }) {
    const r = svgRef.current!.getBoundingClientRect();
    return { x: (e.clientX - r.left - view.x) / view.k, y: (e.clientY - r.top - view.y) / view.k };
  }

  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const r = svgRef.current!.getBoundingClientRect();
    const mx = e.clientX - r.left, my = e.clientY - r.top;
    const factor = Math.exp(-e.deltaY * 0.0015);
    const k = Math.min(8, Math.max(0.1, view.k * factor));
    setView({ k, x: mx - (mx - view.x) * (k / view.k), y: my - (my - view.y) * (k / view.k) });
  }

  function onMouseDown(e: React.MouseEvent) {
    setMenu(null); setPadMenu(null);
    const empty = (e.target as Element).closest('.node, .link, .pad') === null;
    if (placingPad && e.button === 0 && klass) {
      const p = toScene(e);
      setPlacingPad(false);
      addPad(p.x - PAD / 2, p.y - PAD / 2);
      return;
    }
    if (e.button === 0 && empty) setSelPad(null);
    if (e.button === 1 || (e.button === 0 && empty && e.altKey)) {
      setPan({ sx: e.clientX, sy: e.clientY, vx: view.x, vy: view.y });
    } else if (e.button === 0 && empty) {
      // рамка выделения (Alt — панорама)
      const p = toScene(e);
      setBand({ x0: p.x, y0: p.y, x1: p.x, y1: p.y });
      if (!e.shiftKey) { select(klass?.name ?? null); setSelNode(null); setSelLink(null); setSelSet(new Set()); }
    }
  }

  function onMouseMove(e: React.MouseEvent) {
    if (pan) setView(v => ({ ...v, x: pan.vx + e.clientX - pan.sx, y: pan.vy + e.clientY - pan.sy }));
    else if (band) { const p = toScene(e); setBand({ ...band, x1: p.x, y1: p.y }); }
    else if (wire) { const p = toScene(e); setWire({ ...wire, x: p.x, y: p.y }); }
    else if (padDrag && klass) {
      const p = toScene(e);
      const [sx, sy] = sheet?.gridSnap ? gridStep : [8, 8];
      const [ox, oy] = sheet?.gridSnap ? gridOrigin : [0, 0];
      const gx = Math.round((p.x - padDrag.dx - ox) / sx) * sx + ox, gy = Math.round((p.y - padDrag.dy - oy) / sy) * sy + oy;
      const pad = klass.pads.find(q => q.id === padDrag.id);
      if (pad && (pad.x !== gx || pad.y !== gy)) {
        setPadDrag({ ...padDrag, moved: true });
        updateClass({ ...klass, pads: klass.pads.map(q => q.id === padDrag.id ? { ...q, x: gx, y: gy } : q) });
      }
    }
    else if (drag && klass) {
      const p = toScene(e);
      // привязка к сетке листа, если она включена в параметрах, иначе шаг 8
      const [sx, sy] = sheet?.gridSnap ? gridStep : [8, 8];
      const [ox, oy] = sheet?.gridSnap ? gridOrigin : [0, 0];
      const gx = Math.round((p.x - drag.dx - ox) / sx) * sx + ox, gy = Math.round((p.y - drag.dy - oy) / sy) * sy + oy;
      const c = klass.children.find(c => c.handle === drag.handle);
      if (c && (c.x !== gx || c.y !== gy)) {
        const ddx = gx - c.x, ddy = gy - c.y;
        const group = selSet.has(drag.handle) ? selSet : new Set([drag.handle]);
        setDrag({ ...drag, moved: true });
        updateClass({ ...klass, children: klass.children.map(c => group.has(c.handle) ? { ...c, x: c.x + ddx, y: c.y + ddy } : c) });
      }
    }
  }

  function onMouseUp(e: React.MouseEvent) {
    if (padDrag && klass && padDrag.moved) {
      const pad = klass.pads.find(q => q.id === padDrag.id);
      if (pad) api.padMove(klass.name, pad.id, pad.x, pad.y).then(() => useStore.getState().markUnsaved());
    }
    setPadDrag(null);
    if (drag && klass && drag.moved) {
      const group = selSet.has(drag.handle) ? selSet : new Set([drag.handle]);
      const items = klass.children.filter(c => group.has(c.handle)).map(c => ({ handle: c.handle, x: c.x, y: c.y }));
      api.moveChildren(klass.name, items).then(() => { showToast(items.length > 1 ? `Перемещено блоков: ${items.length}` : 'Перемещено'); useStore.getState().markUnsaved(); });
    }
    if (band && klass) {
      const [bx0, bx1] = [Math.min(band.x0, band.x1), Math.max(band.x0, band.x1)], [by0, by1] = [Math.min(band.y0, band.y1), Math.max(band.y0, band.y1)];
      if (bx1 - bx0 > 2 || by1 - by0 > 2) {
        const inside = klass.children.filter(c => { const { w, h } = nodeSize(c); return c.x >= bx0 && c.y >= by0 && c.x + w <= bx1 && c.y + h <= by1; }).map(c => c.handle);
        const next = new Set(e.shiftKey ? [...selSet, ...inside] : inside);
        setSelSet(next);
        if (inside.length) setSelNode(inside[0]);
      }
      setBand(null);
    }
    if (wire && klass) {
      const el = e.target as Element;
      const target = el.closest('.node')?.getAttribute('data-handle');
      const toPad = el.closest('.pad')?.getAttribute('data-pad');
      if (wire.fromPad !== undefined) {
        // от площадки к блоку: источник — сам имидж
        if (target !== null && target !== undefined && Number(target) !== SELF) setLinkEdit({ handle: 0, source: SELF, target: Number(target), pairs: [], pad: wire.fromPad });
      } else if (toPad !== null && toPad !== undefined && wire.from !== SELF) {
        setLinkEdit({ handle: 0, source: wire.from, target: SELF, pairs: [], pad: Number(toPad) });
      } else if (target !== null && target !== undefined && Number(target) !== wire.from) {
        setLinkEdit({ handle: 0, source: wire.from, target: Number(target), pairs: [] });
      }
    }
    setDrag(null); setPan(null); setWire(null);
  }

  // перетаскивание класса из иерархии
  function onDragOver(e: React.DragEvent) {
    if (editable && e.dataTransfer.types.includes(DRAG_CLASS)) { e.preventDefault(); e.dataTransfer.dropEffect = 'copy'; }
  }
  async function onDrop(e: React.DragEvent) {
    const name = e.dataTransfer.getData(DRAG_CLASS);
    if (!name || !klass || !editable) return;
    e.preventDefault();
    const p = toScene(e);
    try {
      const r = await api.addChild(klass.name, name, Math.round(p.x / 8) * 8, Math.round(p.y / 8) * 8);
      await reload();
      setSelNode(r.handle);
      showToast(`Добавлен ${name}`);
    } catch (err) { say({ level: 'error', where: klass.name, text: String(err) }); }
  }

  async function pasteBlocks(items: { class: string; name: string; x: number; y: number }[]) {
    if (!klass) return;
    let last = 0;
    for (const it of items) {
      const r = await api.addChild(klass.name, it.class, it.x + 16, it.y + 16, it.name ? it.name + '_копия' : '');
      last = r.handle;
    }
    await reload();
    setSelNode(last);
    showToast('Вставлено');
  }

  async function removeChild(handle: number) {
    if (!klass) return;
    await api.removeChild(klass.name, handle);
    await reload();
    setSelNode(null);
    showToast('Удалено');
  }
  async function removeLink(handle: number) {
    if (!klass) return;
    await api.removeLink(klass.name, handle);
    await reload();
    setSelLink(null);
    showToast('Связь удалена');
  }
  async function commitRename() {
    if (!klass || !renaming) return;
    await api.renameChild(klass.name, renaming.handle, renaming.value.trim());
    setRenaming(null);
    await reload();
  }
  async function addPad(x: number, y: number) {
    if (!klass) return;
    const r = await api.padAdd(klass.name, Math.round(x / 8) * 8, Math.round(y / 8) * 8);
    useStore.getState().markUnsaved();
    await reload();
    setSelPad(r.id); setSelNode(null); setSelLink(null);
    showToast('Контактная площадка поставлена — потяните от её края к блоку');
  }
  async function removePad(id: number) {
    if (!klass) return;
    const r = await api.padRemove(klass.name, id);
    useStore.getState().markUnsaved();
    await reload();
    setSelPad(null);
    showToast(r.links ? `Площадка удалена вместе со связями: ${r.links}` : 'Площадка удалена');
  }
  async function saveLink(pairs: [string, string][], style: LinkStyle) {
    if (!klass || !linkEdit) return;
    const r = await api.setLink(klass.name, linkEdit.handle, linkEdit.source, linkEdit.target, pairs, linkEdit.pad ?? 0);
    if (r.handle) await api.setLinkStyle(klass.name, r.handle, style);
    setLinkEdit(null);
    await reload();
    setSelLink(r.handle || null);
    showToast(pairs.length ? 'Связь сохранена' : 'Связь удалена');
  }

  if (!klass) return <div className="scheme" />;

  const nodes: NodeLike[] = selfNode ? [selfNode, ...klass.children] : klass.children;
  const pads = klass.pads ?? [];
  const centers = new Map(nodes.map(c => { const { w, h } = nodeSize(c); return [c.handle, { x: c.x + w / 2, y: c.y + h / 2 }]; }));
  const padCenter = (id: number) => { const p = pads.find(q => q.id === id); return p ? { x: p.x + PAD / 2, y: p.y + PAD / 2 } : undefined; };
  // конец связи: со стороны самого имиджа — площадка, если она есть
  const endOf = (h: number, pad: number) => (h === SELF ? padCenter(pad) : undefined) ?? centers.get(h);
  const padVars = (id: number) => klass.links.filter(l => l.pad === id && (l.source === SELF || l.target === SELF)).flatMap(l => l.vars.map(([a, b]) => l.source === SELF ? a : b));
  const labelOf = (h: number) => {
    if (h === SELF) return `${klass.name} (сам)`;
    const n = nodes.find(n => n.handle === h);
    return n ? n.name || n.class : `#${h}`;
  };
  const classOf = (h: number) => h === SELF ? klass : classByName(project, nodes.find(n => n.handle === h)?.class);

  return (
    <div className={`scheme${placingPad ? ' placing' : ''}`} onWheel={onWheel} onDragOver={onDragOver} onDrop={onDrop}>
      <svg ref={svgRef} className="canvas" onMouseDown={onMouseDown} onMouseMove={onMouseMove} onMouseUp={onMouseUp} onMouseLeave={onMouseUp}
        onContextMenu={e => e.preventDefault()}>
        <defs>
          <pattern id="grid" width={gridStep[0] * view.k} height={gridStep[1] * view.k} patternUnits="userSpaceOnUse" x={view.x + gridOrigin[0] * view.k} y={view.y + gridOrigin[1] * view.k}>
            <path className="grid" d={`M ${gridStep[0] * view.k} 0 L 0 0 0 ${gridStep[1] * view.k}`} fill="none" />
          </pattern>
          <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path d="M0 0 L10 5 L0 10 z" fill="context-stroke" /></marker>
        </defs>
        {layers.grid && <rect width="100%" height="100%" fill="url(#grid)" />}
        <rect width="100%" height="100%" fill="transparent" onContextMenu={e => { e.preventDefault(); if (editable) setSheetMenu({ x: e.clientX, y: e.clientY }); }} />
        <g transform={`translate(${view.x} ${view.y}) scale(${view.k})`}>
          {layers.graphics && background.svg && background.bounds && (
            <g opacity="0.85" dangerouslySetInnerHTML={{ __html: background.svg.replace(/<svg[^>]*>/, `<svg x="${background.bounds.x}" y="${background.bounds.y}" width="${background.bounds.w}" height="${background.bounds.h}" viewBox="0 0 ${background.bounds.w} ${background.bounds.h}" xmlns="http://www.w3.org/2000/svg">`) }} />
          )}
          {layers.links && klass.links.map(l => {
            const a = endOf(l.source, l.pad), b = endOf(l.target, l.pad);
            if (!a || !b) return null;
            const title = l.vars.map(([p, q]) => `${p} → ${q}`).join(', ');
            const mx = (a.x + b.x) / 2;
            const d = `M ${a.x} ${a.y} C ${mx} ${a.y}, ${mx} ${b.y}, ${b.x} ${b.y}`;
            const st = l.style;
            return (
              <g key={l.handle} className={`link${selLink === l.handle ? ' selected' : ''}${st?.disabled ? ' disabled' : ''}`}
                onMouseDown={e => { e.stopPropagation(); setSelLink(l.handle); setSelNode(null); if (e.button === 2) setLinkMenu({ x: e.clientX, y: e.clientY, handle: l.handle }); }}
                onContextMenu={e => e.preventDefault()}
                onDoubleClick={e => { e.stopPropagation(); if (editable) setLinkEdit({ handle: l.handle, source: l.source, target: l.target, pairs: l.vars, style: l.style, pad: l.pad }); }}>
                <path className="hit" d={d} />
                <path className="wire" d={d} style={{ stroke: st?.color || undefined, strokeWidth: st?.width ? st.width / view.k : undefined }} markerEnd={st?.arrows ? 'url(#arrow)' : undefined} />
                <title>{(title || 'связь без пар') + (st?.disabled ? ' · выключена' : '')}</title>
              </g>
            );
          })}
          {band && <rect className="band" x={Math.min(band.x0, band.x1)} y={Math.min(band.y0, band.y1)} width={Math.abs(band.x1 - band.x0)} height={Math.abs(band.y1 - band.y0)} />}
          {wire && (() => {
            const a = wire.fromPad !== undefined ? padCenter(wire.fromPad) : centers.get(wire.from);
            return a && <path className="link drawing" d={`M ${a.x} ${a.y} L ${wire.x} ${wire.y}`} />;
          })()}
          {layers.images && nodes.map(c => {
            const { w, h, label } = nodeSize(c);
            const cls = classByName(project, c.class);
            const isSelf = c.handle === SELF;
            const selected = selNode === c.handle || selSet.has(c.handle) || (selNode === null && selectedClass === c.class && !isSelf);
            const ports = cls?.vars.filter(v => !v.local).slice(0, 6) ?? [];
            return (
              <g key={c.handle} data-handle={c.handle} className={`node${selected ? ' selected' : ''}${isSelf ? ' self' : ''}`} transform={`translate(${c.x} ${c.y})`}
                onMouseDown={e => {
                  e.stopPropagation(); setMenu(null);
                  if (e.button === 2) return;
                  setSelNode(c.handle); setSelLink(null);
                  if (e.shiftKey && !isSelf) setSelSet(prev => { const n = new Set(prev); n.has(c.handle) ? n.delete(c.handle) : n.add(c.handle); return n; });
                  else if (!selSet.has(c.handle)) setSelSet(new Set());
                  if (!isSelf) { const p = toScene(e); setDrag({ handle: c.handle, dx: p.x - c.x, dy: p.y - c.y, moved: false }); }
                  select(c.class, isSelf ? null : instanceIndexFor(c.handle));
                }}
                onContextMenu={e => { e.preventDefault(); e.stopPropagation(); if (editable) { setSelNode(c.handle); setMenu({ x: e.clientX, y: e.clientY, handle: c.handle }); } }}
                onDoubleClick={e => { e.stopPropagation(); if (!isSelf && cls && (cls.children.length || cls.hasScheme)) enterScheme(c.class); }}>
                <rect className="box" width={w} height={h} />
                <image href={api.iconUrl(c.class)} x={4} y={(h - ICON) / 2} width={ICON} height={ICON} />
                <text x={ICON + 10} y={h / 2 + 4}>{isSelf ? `${label} ⌂` : label}<title>{isSelf ? 'Сам имидж схемы: связи с его переменными' : c.name ? `${c.name} [${c.class}]` : c.class}</title></text>
                {ports.map((v, i) => (
                  <circle key={v.name} className="port" cx={i % 2 ? w : 0} cy={8 + Math.floor(i / 2) * 10} r={4} fill={`var(--type-${v.type.toLowerCase()})`} stroke="var(--bg-surface)"
                    onMouseDown={e => { if (!editable) return; e.stopPropagation(); const p = toScene(e); setWire({ from: c.handle, x: p.x, y: p.y }); setSelNode(c.handle); }}>
                    <title>{v.name}: {v.type}{editable ? ' — потяните к другому блоку, чтобы связать' : ''}</title>
                  </circle>
                ))}
              </g>
            );
          })}
          {layers.images && pads.map(p => {
            const vars = padVars(p.id);
            return (
              <g key={'pad' + p.id} data-pad={p.id} className={`pad${selPad === p.id ? ' selected' : ''}`} transform={`translate(${p.x} ${p.y})`}
                onMouseDown={e => {
                  e.stopPropagation(); setMenu(null); setPadMenu(null);
                  if (e.button !== 0) return;
                  setSelPad(p.id); setSelNode(null); setSelLink(null); setSelSet(new Set());
                  select(klass.name);
                  if (!editable) return;
                  const q = toScene(e);
                  // от края площадки тянется связь, за середину — перемещение
                  const r = Math.hypot(q.x - p.x - PAD / 2, q.y - p.y - PAD / 2);
                  if (r > PAD * 0.3 || e.shiftKey) setWire({ from: SELF, fromPad: p.id, x: q.x, y: q.y });
                  else setPadDrag({ id: p.id, dx: q.x - p.x, dy: q.y - p.y, moved: false });
                }}
                onContextMenu={e => { e.preventDefault(); e.stopPropagation(); if (editable) { setSelPad(p.id); setPadMenu({ x: e.clientX, y: e.clientY, id: p.id }); } }}
                onDoubleClick={e => { e.stopPropagation(); setPadProps(p.id); }}>
                <rect className="pad-ring" x={2} y={2} width={PAD - 4} height={PAD - 4} rx={PAD / 2} />
                <rect className="pad-core" x={PAD / 2 - 5} y={PAD / 2 - 5} width={10} height={10} rx={2} transform={`rotate(45 ${PAD / 2} ${PAD / 2})`} />
                {vars.length > 0 && (() => {
                  const label = vars.slice(0, 3).join(', ') + (vars.length > 3 ? '…' : '');
                  const w = label.length * 6.6 + 10;
                  return <g className="pad-label">
                    <rect x={PAD / 2 - w / 2} y={PAD + 2} width={w} height={15} rx={7.5} />
                    <text x={PAD / 2} y={PAD + 13} textAnchor="middle">{label}</text>
                  </g>;
                })()}
                <title>{`Контактная площадка${vars.length ? ': ' + vars.join(', ') : ''}${editable ? ' — потяните от края к блоку, чтобы связать; за середину — переместить' : ''}`}</title>
              </g>
            );
          })}
        </g>
      </svg>
      <div className="toolbar">
        {editable && <button className={`small${placingPad ? ' active' : ''}`} onClick={() => setPlacingPad(v => !v)} title="Вставка → Контактная площадка: щёлкните на листе">+ Площадка</button>}
        <button className="small" onClick={fitAll} title="Shift+1">Показать всё</button>
        <button className="small" onClick={() => setView(v => ({ ...v, k: 1 }))} title="Shift+0">100%</button>
        <span className="muted mono small" style={{ alignSelf: 'center', padding: '0 6px' }}>{Math.round(view.k * 100)}%</span>
      </div>
      {editable && !klass.children.length && (
        <div className="hint muted">Перетащите имидж из иерархии на холст, чтобы добавить его на схему.</div>
      )}
      {menu && (
        <div className="context" style={{ left: menu.x, top: menu.y }} onMouseDown={e => e.stopPropagation()}>
          <button onClick={() => { const n = nodes.find(n => n.handle === menu.handle); setMenu(null); select(n?.class ?? null); setTab('code'); }}>Открыть код</button>
          {menu.handle !== SELF && <button onClick={() => { const n = klass.children.find(n => n.handle === menu.handle); setMenu(null); setRenaming({ handle: menu.handle, value: n?.name ?? '' }); }}>Переименовать…</button>}
          {menu.handle !== SELF && <button onClick={() => { const c = klass.children.find(n => n.handle === menu.handle); setMenu(null); if (c) { setClipboard([{ class: c.class, name: c.name, x: c.x, y: c.y }]); showToast('Скопировано'); } }}>Копировать <span className="muted">Ctrl+C</span></button>}
          {menu.handle !== SELF && <button onClick={() => { const c = klass.children.find(n => n.handle === menu.handle); setMenu(null); if (c) pasteBlocks([{ class: c.class, name: c.name, x: c.x, y: c.y }]); }}>Дублировать <span className="muted">Ctrl+D</span></button>}
          {menu.handle !== SELF && <button onClick={() => { const c = klass.children.find(n => n.handle === menu.handle); setMenu(null); setReplacing({ handle: menu.handle, value: c?.class ?? '' }); }}>Заменить другим…</button>}
          {menu.handle !== SELF && <button onClick={() => { setMenu(null); let n = 1; while (project?.classes.some(c => c.name.toLowerCase() === `блок${n}`)) n++; setMerging({ picked: new Set(selSet.size ? [...selSet, menu.handle] : [menu.handle]), name: `Блок${n}` }); }}>Конвертировать в один имидж…</button>}
          {menu.handle !== SELF && <button onClick={() => { setMenu(null); removeChild(menu.handle); }}>Удалить <span className="muted">Del</span></button>}
        </div>
      )}
      {renaming && (
        <div className="modal-backdrop" onMouseDown={() => setRenaming(null)}>
          <form className="modal" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); commitRename(); }}>
            <div className="panel-title">Имя экземпляра</div>
            <div className="modal-body">
              <input autoFocus type="text" value={renaming.value} onChange={e => setRenaming({ ...renaming, value: e.target.value })} placeholder="пусто — как у класса" />
              <div className="muted small" style={{ marginTop: 6 }}>Имя используется в путях сообщений и в иерархии.</div>
            </div>
            <div className="modal-actions">
              <button type="button" className="ghost" onClick={() => setRenaming(null)}>Отмена</button>
              <button type="submit" className="primary">Переименовать</button>
            </div>
          </form>
        </div>
      )}
      {padMenu && (
        <div className="context" style={{ left: padMenu.x, top: padMenu.y }} onMouseDown={e => e.stopPropagation()} onMouseLeave={() => setPadMenu(null)}>
          <button onClick={() => { setPadProps(padMenu.id); setPadMenu(null); }}>Свойства контактной площадки…</button>
          <button onClick={() => { const id = padMenu.id; setPadMenu(null); removePad(id); }}>Удалить контактную площадку <span className="muted">Del</span></button>
        </div>
      )}
      {padProps !== null && (() => {
        const pad = pads.find(p => p.id === padProps);
        if (!pad) return null;
        const through = klass.links.filter(l => l.pad === pad.id && (l.source === SELF || l.target === SELF));
        return (
          <div className="modal-backdrop" onMouseDown={() => setPadProps(null)}>
            <div className="modal" style={{ width: 440 }} onMouseDown={e => e.stopPropagation()}>
              <div className="panel-title">Контактная площадка</div>
              <div className="modal-body">
                <div className="muted small" style={{ marginBottom: 8 }}>Дескриптор {pad.id} · положение {pad.x}, {pad.y}. Через площадку связи идут к переменным имиджа {klass.name}.</div>
                {through.length === 0 && <div className="muted">Связей нет. Потяните от края площадки к блоку на схеме.</div>}
                {through.map(l => {
                  const other = l.source === SELF ? l.target : l.source;
                  return (
                    <div key={l.handle} className="row" style={{ alignItems: 'center', gap: 8, padding: '4px 0', borderBottom: '1px solid var(--border)' }}>
                      <span style={{ flex: 1 }}><b>{labelOf(other)}</b> <span className="muted">· {l.vars.map(([a, b]) => l.source === SELF ? `${a} ↔ ${b}` : `${b} ↔ ${a}`).join(', ') || 'без пар'}</span></span>
                      {editable && <button className="small ghost" onClick={() => { setPadProps(null); setLinkEdit({ handle: l.handle, source: l.source, target: l.target, pairs: l.vars, style: l.style, pad: l.pad }); }}>Связь…</button>}
                    </div>
                  );
                })}
              </div>
              <div className="modal-actions">
                {editable && <button type="button" className="ghost danger" onClick={() => { setPadProps(null); removePad(pad.id); }}>Удалить</button>}
                <span style={{ flex: 1 }} />
                <button type="button" className="primary" onClick={() => setPadProps(null)}>Закрыть</button>
              </div>
            </div>
          </div>
        );
      })()}
      {linkMenu && (
        <div className="context" style={{ left: linkMenu.x, top: linkMenu.y }} onMouseDown={e => e.stopPropagation()} onMouseLeave={() => setLinkMenu(null)}>
          <button onClick={() => { const l = klass.links.find(l => l.handle === linkMenu.handle); setLinkMenu(null); if (l && editable) setLinkEdit({ handle: l.handle, source: l.source, target: l.target, pairs: l.vars, style: l.style, pad: l.pad }); }}>Свойства связи…</button>
          <button onClick={() => { setLinkMenu(null); removeLink(linkMenu.handle); }} disabled={!editable}>Удалить эту связь <span className="muted">Del</span></button>
        </div>
      )}
      {sheetMenu && (
        <div className="context" style={{ left: sheetMenu.x, top: sheetMenu.y }} onMouseDown={e => e.stopPropagation()} onMouseLeave={() => setSheetMenu(null)}>
          <button onClick={() => { setSheetMenu(null); select(klass.name); setDialog('sheet'); }}>Параметры листа…</button>
          <button onClick={() => { setSheetMenu(null); select(klass.name); setDialog('calcOrder'); }}>Порядок вычислений…</button>
          <button onClick={() => { setSheetMenu(null); select(klass.name); setDialog('classProps'); }}>Свойства имиджа…</button>
          <button onClick={async () => {
            setSheetMenu(null);
            const [sx, sy] = gridStep, [ox, oy] = gridOrigin;
            for (const c of klass.children) {
              const nx = Math.round((c.x - ox) / sx) * sx + ox, ny = Math.round((c.y - oy) / sy) * sy + oy;
              if (nx !== c.x || ny !== c.y) await api.moveChild(klass.name, c.handle, nx, ny);
            }
            useStore.getState().markUnsaved(); await useStore.getState().reload(); showToast('Выстроено по сетке');
          }} disabled={!editable}>Выстроить имиджи по узлам сетки</button>
          <button onClick={() => { setSheetMenu(null); if (clipboard.length) pasteBlocks(clipboard); }} disabled={!clipboard.length}>Вставить <span className="muted">Ctrl+V</span></button>
          <button onClick={() => { const at = toScene({ clientX: sheetMenu.x, clientY: sheetMenu.y }); setSheetMenu(null); addPad(at.x - PAD / 2, at.y - PAD / 2); }}>Контактная площадка</button>
        </div>
      )}
      {merging && (
        <div className="modal-backdrop" onMouseDown={() => setMerging(null)}>
          <form className="modal" style={{ width: 460 }} onMouseDown={e => e.stopPropagation()} onSubmit={async e => {
            e.preventDefault();
            try {
              const r = await api.mergeChildren(klass.name, merging.name.trim(), [...merging.picked]);
              setMerging(null); useStore.getState().markUnsaved(); await useStore.getState().reload(); setSelNode(r.handle); showToast(`Создан имидж ${merging.name}`);
            } catch (err) { useStore.getState().say({ level: 'error', where: 'схема', text: String(err) }); }
          }}>
            <div className="panel-title">Конвертировать в один имидж</div>
            <div className="modal-body">
              <label className="prop"><span>Имя нового имиджа</span><input autoFocus type="text" value={merging.name} onChange={e => setMerging({ ...merging, name: e.target.value })} /></label>
              <div className="muted small" style={{ margin: '8px 0 4px' }}>Блоки, которые уйдут внутрь (внешние связи станут переменными нового имиджа):</div>
              <div className="scroll list" style={{ maxHeight: 240, border: '1px solid var(--border)', borderRadius: 6 }}>
                {klass.children.map(c => (
                  <label key={c.handle} className="row check" style={{ cursor: 'pointer' }}>
                    <input type="checkbox" checked={merging.picked.has(c.handle)} onChange={e => { const p = new Set(merging.picked); e.target.checked ? p.add(c.handle) : p.delete(c.handle); setMerging({ ...merging, picked: p }); }} />
                    <span>{c.name || c.class}</span>{c.name && <span className="muted"> · {c.class}</span>}
                  </label>
                ))}
              </div>
            </div>
            <div className="modal-actions">
              <button type="button" className="ghost" onClick={() => setMerging(null)}>Отмена</button>
              <button type="submit" className="primary" disabled={!merging.picked.size || !merging.name.trim()}>Конвертировать ({merging.picked.size})</button>
            </div>
          </form>
        </div>
      )}
      {replacing && (
        <div className="modal-backdrop" onMouseDown={() => setReplacing(null)}>
          <form className="modal" style={{ width: 420 }} onMouseDown={e => e.stopPropagation()} onSubmit={async e => {
            e.preventDefault();
            const r = await api.replaceChild(klass.name, replacing.handle, replacing.value);
            setReplacing(null);
            useStore.getState().markUnsaved(); await useStore.getState().reload();
            showToast(r.droppedPairs ? `Заменено; снято пар связей: ${r.droppedPairs}` : 'Заменено');
          }}>
            <div className="panel-title">Заменить другим имиджем</div>
            <div className="modal-body">
              <select autoFocus value={replacing.value} onChange={e => setReplacing({ ...replacing, value: e.target.value })} style={{ width: '100%' }}>
                {(project?.classes ?? []).map(c => <option key={c.name} value={c.name}>{c.name}{c.library ? ' · библиотека' : ''}</option>)}
              </select>
              <div className="muted small" style={{ marginTop: 6 }}>Положение, имя и связи сохраняются; пары с переменными, которых нет в новом имидже, снимаются.</div>
            </div>
            <div className="modal-actions">
              <button type="button" className="ghost" onClick={() => setReplacing(null)}>Отмена</button>
              <button type="submit" className="primary">Заменить</button>
            </div>
          </form>
        </div>
      )}
      {linkEdit && (
        <LinkDialog isNew={linkEdit.handle === 0} pairs={linkEdit.pairs} style={linkEdit.style}
          source={{ label: labelOf(linkEdit.source), cls: classOf(linkEdit.source) }}
          target={{ label: labelOf(linkEdit.target), cls: classOf(linkEdit.target) }}
          onSubmit={saveLink} onClose={() => setLinkEdit(null)} />
      )}
    </div>
  );

  function instanceIndexFor(handle: number): number | null {
    const insts = useStore.getState().instances;
    const parent = insts.find(i => i.class.toLowerCase() === klass!.name.toLowerCase());
    const found = insts.find(i => i.handle === handle && (parent ? i.parent === parent.index : true));
    return found ? found.index : null;
  }
}

export type { ClassInfo };
