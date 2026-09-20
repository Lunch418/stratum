// Холст схемы: графика листа (SVG из ядра), блоки имиджей, связи;
// панорама, зум к курсору, перетаскивание блоков, вход в подсхему.
// Правка: перетаскивание имиджа из иерархии добавляет экземпляр, тяга от
// порта к блоку создаёт связь, контекстное меню и Delete удаляют.
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
  const [wire, setWire] = useState<{ from: number; x: number; y: number } | null>(null);
  const [selNode, setSelNode] = useState<number | null>(null);
  const [selLink, setSelLink] = useState<number | null>(null);
  const [menu, setMenu] = useState<{ x: number; y: number; handle: number } | null>(null);
  const [renaming, setRenaming] = useState<{ handle: number; value: string } | null>(null);
  const [linkEdit, setLinkEdit] = useState<{ handle: number; source: number; target: number; pairs: [string, string][]; style?: LinkStyle } | null>(null);
  const [linkMenu, setLinkMenu] = useState<{ x: number; y: number; handle: number } | null>(null);
  const [sheetMenu, setSheetMenu] = useState<{ x: number; y: number } | null>(null);
  const [replacing, setReplacing] = useState<{ handle: number; value: string } | null>(null);
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
  const selfNode: NodeLike | null = useMemo(() => {
    if (!klass) return null;
    const b = bounds ?? { x0: 0, y0: 0, x1: 0, y1: 0 };
    return { handle: SELF, class: klass.name, name: '', x: b.x0 - nodeSize({ name: '', class: klass.name }).w - 60, y: b.y0 };
  }, [klass?.name, bounds?.x0, bounds?.y0]);

  // Delete / Escape
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('input, textarea, select, .monaco-editor')) return;
      if (e.key === 'Escape') { setMenu(null); setWire(null); setRenaming(null); }
      if ((e.key === 'Delete' || e.key === 'Backspace') && editable && klass) {
        if (selNode !== null && selNode !== SELF) { e.preventDefault(); removeChild(selNode); }
        else if (selLink !== null) { e.preventDefault(); removeLink(selLink); }
      }
      // буфер обмена схемы: копировать/вырезать/вставить/дублировать блок
      if (e.ctrlKey && klass && selNode !== null && selNode !== SELF && ['c', 'x', 'd'].includes(e.key.toLowerCase())) {
        const c = klass.children.find(x => x.handle === selNode);
        if (!c) return;
        e.preventDefault();
        if (e.key.toLowerCase() === 'd') { if (editable) pasteBlocks([{ class: c.class, name: c.name, x: c.x, y: c.y }]); return; }
        setClipboard([{ class: c.class, name: c.name, x: c.x, y: c.y }]);
        showToast(e.key.toLowerCase() === 'x' ? 'Вырезано' : 'Скопировано');
        if (e.key.toLowerCase() === 'x' && editable) removeChild(selNode);
      } else if (e.ctrlKey && e.key.toLowerCase() === 'v' && editable && clipboard.length) {
        e.preventDefault();
        pasteBlocks(clipboard);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [selNode, selLink, editable, klass?.name]);

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
    setMenu(null);
    if (e.button === 1 || (e.button === 0 && (e.target as Element).closest('.node, .link') === null)) {
      setPan({ sx: e.clientX, sy: e.clientY, vx: view.x, vy: view.y });
      if (e.button === 0) { select(klass?.name ?? null); setSelNode(null); setSelLink(null); }
    }
  }

  function onMouseMove(e: React.MouseEvent) {
    if (pan) setView(v => ({ ...v, x: pan.vx + e.clientX - pan.sx, y: pan.vy + e.clientY - pan.sy }));
    else if (wire) { const p = toScene(e); setWire({ ...wire, x: p.x, y: p.y }); }
    else if (drag && klass) {
      const p = toScene(e);
      // привязка к сетке листа, если она включена в параметрах, иначе шаг 8
      const [sx, sy] = sheet?.gridSnap ? gridStep : [8, 8];
      const [ox, oy] = sheet?.gridSnap ? gridOrigin : [0, 0];
      const gx = Math.round((p.x - drag.dx - ox) / sx) * sx + ox, gy = Math.round((p.y - drag.dy - oy) / sy) * sy + oy;
      const c = klass.children.find(c => c.handle === drag.handle);
      if (c && (c.x !== gx || c.y !== gy)) {
        setDrag({ ...drag, moved: true });
        updateClass({ ...klass, children: klass.children.map(c => c.handle === drag.handle ? { ...c, x: gx, y: gy } : c) });
      }
    }
  }

  function onMouseUp(e: React.MouseEvent) {
    if (drag && klass && drag.moved) {
      const c = klass.children.find(c => c.handle === drag.handle);
      if (c) api.moveChild(klass.name, c.handle, c.x, c.y).then(() => { showToast('Перемещено'); useStore.getState().markUnsaved(); });
    }
    if (wire && klass) {
      const target = (e.target as Element).closest('.node')?.getAttribute('data-handle');
      if (target !== null && target !== undefined && Number(target) !== wire.from) {
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
  async function saveLink(pairs: [string, string][], style: LinkStyle) {
    if (!klass || !linkEdit) return;
    const r = await api.setLink(klass.name, linkEdit.handle, linkEdit.source, linkEdit.target, pairs);
    if (r.handle) await api.setLinkStyle(klass.name, r.handle, style);
    setLinkEdit(null);
    await reload();
    setSelLink(r.handle || null);
    showToast(pairs.length ? 'Связь сохранена' : 'Связь удалена');
  }

  if (!klass) return <div className="scheme" />;

  const nodes: NodeLike[] = selfNode ? [selfNode, ...klass.children] : klass.children;
  const centers = new Map(nodes.map(c => { const { w, h } = nodeSize(c); return [c.handle, { x: c.x + w / 2, y: c.y + h / 2 }]; }));
  const labelOf = (h: number) => {
    const n = nodes.find(n => n.handle === h);
    return n ? (n.handle === SELF ? `${n.class} (сам)` : n.name || n.class) : `#${h}`;
  };
  const classOf = (h: number) => classByName(project, nodes.find(n => n.handle === h)?.class);

  return (
    <div className="scheme" onWheel={onWheel} onDragOver={onDragOver} onDrop={onDrop}>
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
            const a = centers.get(l.source), b = centers.get(l.target);
            if (!a || !b) return null;
            const title = l.vars.map(([p, q]) => `${p} → ${q}`).join(', ');
            const mx = (a.x + b.x) / 2;
            const d = `M ${a.x} ${a.y} C ${mx} ${a.y}, ${mx} ${b.y}, ${b.x} ${b.y}`;
            const st = l.style;
            return (
              <g key={l.handle} className={`link${selLink === l.handle ? ' selected' : ''}${st?.disabled ? ' disabled' : ''}`}
                onMouseDown={e => { e.stopPropagation(); setSelLink(l.handle); setSelNode(null); if (e.button === 2) setLinkMenu({ x: e.clientX, y: e.clientY, handle: l.handle }); }}
                onContextMenu={e => e.preventDefault()}
                onDoubleClick={e => { e.stopPropagation(); if (editable) setLinkEdit({ handle: l.handle, source: l.source, target: l.target, pairs: l.vars, style: l.style }); }}>
                <path className="hit" d={d} />
                <path className="wire" d={d} style={{ stroke: st?.color || undefined, strokeWidth: st?.width ? st.width / view.k : undefined }} markerEnd={st?.arrows ? 'url(#arrow)' : undefined} />
                <title>{(title || 'связь без пар') + (st?.disabled ? ' · выключена' : '')}</title>
              </g>
            );
          })}
          {wire && centers.get(wire.from) && (
            <path className="link drawing" d={`M ${centers.get(wire.from)!.x} ${centers.get(wire.from)!.y} L ${wire.x} ${wire.y}`} />
          )}
          {layers.images && nodes.map(c => {
            const { w, h, label } = nodeSize(c);
            const cls = classByName(project, c.class);
            const isSelf = c.handle === SELF;
            const selected = selNode === c.handle || (selNode === null && selectedClass === c.class && !isSelf);
            const ports = cls?.vars.filter(v => !v.local).slice(0, 6) ?? [];
            return (
              <g key={c.handle} data-handle={c.handle} className={`node${selected ? ' selected' : ''}${isSelf ? ' self' : ''}`} transform={`translate(${c.x} ${c.y})`}
                onMouseDown={e => {
                  e.stopPropagation(); setMenu(null);
                  if (e.button === 2) return;
                  setSelNode(c.handle); setSelLink(null);
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
        </g>
      </svg>
      <div className="toolbar">
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
      {linkMenu && (
        <div className="context" style={{ left: linkMenu.x, top: linkMenu.y }} onMouseDown={e => e.stopPropagation()} onMouseLeave={() => setLinkMenu(null)}>
          <button onClick={() => { const l = klass.links.find(l => l.handle === linkMenu.handle); setLinkMenu(null); if (l && editable) setLinkEdit({ handle: l.handle, source: l.source, target: l.target, pairs: l.vars, style: l.style }); }}>Свойства связи…</button>
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
