// Редактор графики имиджа — панель «Рисование» оригинала: выбор и
// перемещение, линия, полилиния, прямоугольник, скруглённый прямоугольник,
// эллипс, дуга, текст, группа, Z-порядок, правка точек, параметры листа.
// Рисует ядро (SVG), страница держит только выбор и текущий инструмент.
import { useEffect, useRef, useState } from 'react';
import { type ObjectProps } from '../api';
import { classByName, useStore } from '../store';

type Tool = 'select' | 'line' | 'polyline' | 'rect' | 'roundrect' | 'ellipse' | 'arc' | 'text' | 'points' | 'pan';
type Kind = 'image' | 'scheme' | 'icon';
interface State { kind: Kind; origin: [number, number]; client: [number, number]; scale: number; view: [number, number, number, number]; svg: string; objects: ObjectProps[] }
type Preview = { dx?: number; dy?: number; resize?: { x: number; y: number; w: number; h: number }; point?: { index: number; p: [number, number] } } | null;

const TOOLS: { id: Tool; label: string; hint: string }[] = [
  { id: 'select', label: '↖', hint: 'Выбор и перемещение (Esc)' },
  { id: 'pan', label: '✥', hint: 'Панорама (или средняя кнопка)' },
  { id: 'line', label: '╱', hint: 'Линия: две точки' },
  { id: 'polyline', label: '⌇', hint: 'Полилиния: точки щелчками, правая кнопка или Enter — закончить' },
  { id: 'rect', label: '▭', hint: 'Прямоугольник' },
  { id: 'roundrect', label: '▢', hint: 'Скруглённый прямоугольник' },
  { id: 'ellipse', label: '◯', hint: 'Эллипс' },
  { id: 'arc', label: '◜', hint: 'Дуга (четверть)' },
  { id: 'text', label: 'T', hint: 'Текст' },
  { id: 'points', label: '⋯', hint: 'Правка точек выбранной линии (Ctrl+щелчок — добавить, Shift+перетащить — удалить)' },
];

export function PictureEditor({ kind }: { kind: Kind }) {
  const project = useStore(s => s.project);
  const selected = useStore(s => s.selectedClass);
  const path = useStore(s => s.schemePath);
  const klass = classByName(project, kind === 'scheme' ? path[path.length - 1] : selected);
  const say = useStore(s => s.say);
  const markUnsaved = useStore(s => s.markUnsaved);
  const [state, setState] = useState<State | null>(null);
  const [tool, setTool] = useState<Tool>('select');
  const [sel, setSel] = useState<number[]>([]);
  const [view, setView] = useState({ x: 40, y: 40, k: 1 });
  const [pen, setPen] = useState({ color: '#000000', width: 1 });
  const [fill, setFill] = useState<{ color: string; on: boolean }>({ color: '#ffffff', on: false });
  const [draft, setDraft] = useState<[number, number][]>([]);
  const [cursor, setCursor] = useState<[number, number] | null>(null);
  const [drag, setDrag] = useState<{ kind: 'move' | 'resize' | 'point' | 'pan'; start: [number, number]; orig?: ObjectProps; corner?: string; index?: number; vx?: number; vy?: number } | null>(null);
  const [preview, setPreview] = useState<Preview>(null);
  const [textAsk, setTextAsk] = useState<{ at: [number, number]; value: string } | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);
  const editable = !!klass && !klass.library;

  const load = async () => {
    if (!klass) { setState(null); return; }
    const r = await fetch(`/api/picture/${encodeURIComponent(klass.name)}?kind=${kind}`);
    if (r.ok) setState(await r.json());
  };
  useEffect(() => { load(); setSel([]); setDraft([]); }, [klass?.name, kind]);

  async function op(body: unknown): Promise<number> {
    if (!klass) return 0;
    const r = await fetch(`/api/picture/${encodeURIComponent(klass.name)}?kind=${kind}`, { method: 'POST', body: JSON.stringify(body) });
    const j = await r.json();
    if (!r.ok) { say({ level: 'error', where: 'рисунок', text: j.error ?? 'ошибка' }); return 0; }
    setState(j.state);
    markUnsaved();
    return j.handle;
  }

  function toScene(e: { clientX: number; clientY: number }): [number, number] {
    const r = svgRef.current!.getBoundingClientRect();
    const st = state!;
    return [(e.clientX - r.left - view.x) / view.k + st.view[0], (e.clientY - r.top - view.y) / view.k + st.view[1]];
  }
  const toScreen = (p: [number, number]): [number, number] => {
    const st = state!;
    return [(p[0] - st.view[0]) * view.k + view.x, (p[1] - st.view[1]) * view.k + view.y];
  };
  const snap = (p: [number, number]): [number, number] => [Math.round(p[0]), Math.round(p[1])];
  const penJson = () => ({ color: pen.color, width: pen.width, style: 0 });
  const brushJson = () => fill.on ? { color: fill.color, style: 0 } : undefined;

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.target as HTMLElement).closest('input, textarea, select, .monaco-editor')) return;
      if (e.key === 'Escape') { setTool('select'); setDraft([]); setSel([]); setTextAsk(null); }
      else if ((e.key === 'Delete' || e.key === 'Backspace') && sel.length && editable) { e.preventDefault(); op(sel.map(h => ({ op: 'delete', handle: h }))).then(() => setSel([])); }
      else if (e.key.toLowerCase() === 'd' && e.ctrlKey && sel.length === 1 && editable) { e.preventDefault(); op({ op: 'duplicate', handle: sel[0] }).then(h => setSel([h])); }
      else if (e.key === 'Enter' && tool === 'polyline' && draft.length >= 2) finishPolyline();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [sel, tool, draft, editable, klass?.name]);

  async function finishPolyline() {
    const pts = draft;
    setDraft([]);
    if (pts.length >= 2) { const h = await op({ op: 'add', shape: 'polyline', points: pts, pen: penJson(), brush: brushJson() }); setSel([h]); }
  }

  function objectAt(p: [number, number]): ObjectProps | undefined {
    if (!state) return undefined;
    for (let i = state.objects.length - 1; i >= 0; i--) {
      const o = state.objects[i];
      if (p[0] >= o.x - 2 && p[0] <= o.x + o.w + 2 && p[1] >= o.y - 2 && p[1] <= o.y + o.h + 2) return o;
    }
    return undefined;
  }

  function onMouseDown(e: React.MouseEvent) {
    if (!state) return;
    const p = toScene(e);
    if (e.button === 1 || tool === 'pan') { setDrag({ kind: 'pan', start: [e.clientX, e.clientY], vx: view.x, vy: view.y }); return; }
    if (e.button === 2) { if (tool === 'polyline') finishPolyline(); return; }
    if (!editable && tool !== 'select') return;
    const target = (e.target as Element).closest('[data-handle-ui]');
    if (target && sel.length === 1) {
      const attr = target.getAttribute('data-handle-ui')!;
      const o = state.objects.find(x => x.handle === sel[0])!;
      if (attr.startsWith('corner:')) { setDrag({ kind: 'resize', start: p, orig: o, corner: attr.slice(7) }); return; }
      if (attr.startsWith('point:')) { setDrag({ kind: 'point', start: p, orig: o, index: Number(attr.slice(6)) }); return; }
    }
    switch (tool) {
      case 'select': case 'points': {
        const o = objectAt(p);
        if (!o) { setSel([]); return; }
        if (e.ctrlKey && tool === 'points' && o.points && sel[0] === o.handle) {
          const pts = o.points.map(q => [q[0] + o.x, q[1] + o.y] as [number, number]);
          let best = 0, bd = Infinity;
          for (let i = 0; i + 1 < pts.length; i++) { const d = distToSeg(p, pts[i], pts[i + 1]); if (d < bd) { bd = d; best = i; } }
          pts.splice(best + 1, 0, snap(p));
          op({ op: 'points', handle: o.handle, points: pts });
          return;
        }
        if (e.shiftKey && tool === 'select') setSel(s => s.includes(o.handle) ? s.filter(h => h !== o.handle) : [...s, o.handle]);
        else if (!sel.includes(o.handle)) setSel([o.handle]);
        if (tool === 'select' && editable) setDrag({ kind: 'move', start: p, orig: o });
        return;
      }
      case 'line': case 'rect': case 'roundrect': case 'ellipse': case 'arc':
        setDraft([snap(p)]);
        return;
      case 'polyline':
        setDraft(d => [...d, snap(p)]);
        return;
      case 'text':
        setTextAsk({ at: snap(p), value: 'Текст' });
        return;
    }
  }

  function onMouseMove(e: React.MouseEvent) {
    if (!state) return;
    const p = toScene(e);
    setCursor(p);
    if (!drag) return;
    if (drag.kind === 'pan') { setView(v => ({ ...v, x: drag.vx! + e.clientX - drag.start[0], y: drag.vy! + e.clientY - drag.start[1] })); return; }
    const dx = p[0] - drag.start[0], dy = p[1] - drag.start[1];
    if (drag.kind === 'move') setPreview({ dx, dy });
    else if (drag.kind === 'resize') setPreview({ resize: resized(drag.orig!, drag.corner!, dx, dy) });
    else if (drag.kind === 'point') setPreview({ point: { index: drag.index!, p: snap(p) } });
  }

  async function onMouseUp(e: React.MouseEvent) {
    if (!state) return;
    const p = toScene(e);
    if (drag) {
      const d = drag; setDrag(null); setPreview(null);
      const dx = Math.round(p[0] - d.start[0]), dy = Math.round(p[1] - d.start[1]);
      if (d.kind === 'move' && (dx || dy)) await op(sel.map(h => ({ op: 'move', handle: h, dx, dy })));
      else if (d.kind === 'resize') { const r = resized(d.orig!, d.corner!, dx, dy); await op({ op: 'resize', handle: d.orig!.handle, ...r }); }
      else if (d.kind === 'point' && d.orig!.points) {
        const pts = d.orig!.points.map(q => [q[0] + d.orig!.x, q[1] + d.orig!.y] as [number, number]);
        if (e.shiftKey && pts.length > 2) pts.splice(d.index!, 1); else pts[d.index!] = snap(p);
        await op({ op: 'points', handle: d.orig!.handle, points: pts });
      }
      return;
    }
    if (draft.length === 1 && ['line', 'rect', 'roundrect', 'ellipse', 'arc'].includes(tool)) {
      const a = draft[0], b = snap(p);
      setDraft([]);
      if (Math.abs(a[0] - b[0]) < 1 && Math.abs(a[1] - b[1]) < 1) return;
      const h = await op({ op: 'add', shape: tool, points: [a, b], pen: penJson(), brush: tool === 'line' ? undefined : brushJson() });
      setSel([h]);
    }
  }

  function onWheel(e: React.WheelEvent) {
    e.preventDefault();
    const r = svgRef.current!.getBoundingClientRect();
    const mx = e.clientX - r.left, my = e.clientY - r.top;
    const k = Math.min(16, Math.max(0.1, view.k * Math.exp(-e.deltaY * 0.0015)));
    setView({ k, x: mx - (mx - view.x) * (k / view.k), y: my - (my - view.y) * (k / view.k) });
  }

  if (!klass) return <div className="muted" style={{ padding: 16 }}>Выберите имидж.</div>;
  if (!state) return <div className="muted" style={{ padding: 16 }}>Загрузка…</div>;

  const selObjs = state.objects.filter(o => sel.includes(o.handle));
  const one = selObjs.length === 1 ? selObjs[0] : null;
  const k = view.k;

  return (
    <div className="picture-editor" onWheel={onWheel}>
      <div className="draw-tools">
        {TOOLS.map(t => <button key={t.id} className={`small${tool === t.id ? ' active' : ''}`} title={t.hint} onClick={() => { setTool(t.id); setDraft([]); }} disabled={!editable && t.id !== 'select' && t.id !== 'pan'}>{t.label}</button>)}
        <span className="sep" />
        <label title="Линия"><input type="color" value={pen.color} onChange={e => setPen({ ...pen, color: e.target.value })} /></label>
        <input type="number" min={0} max={20} value={pen.width} onChange={e => setPen({ ...pen, width: Number(e.target.value) })} style={{ width: 44 }} title="Толщина" />
        <label title="Заливка" style={{ display: 'flex', alignItems: 'center', gap: 2 }}><input type="checkbox" checked={fill.on} onChange={e => setFill({ ...fill, on: e.target.checked })} /><input type="color" value={fill.color} onChange={e => setFill({ ...fill, color: e.target.value })} /></label>
        <span className="sep" />
        <button className="small" disabled={sel.length < 2} onClick={() => op({ op: 'group', handles: sel }).then(h => setSel([h]))} title="Группа">▣</button>
        <button className="small" disabled={!one || one.kind !== 'group'} onClick={() => op({ op: 'ungroup', handle: one!.handle }).then(() => setSel([]))} title="Разгруппировать">▤</button>
        <button className="small" disabled={!one} onClick={() => op({ op: 'zorder', handle: one!.handle, to: 'top' })} title="На передний план">⤒</button>
        <button className="small" disabled={!one} onClick={() => op({ op: 'zorder', handle: one!.handle, to: 'bottom' })} title="На задний план">⤓</button>
        <button className="small" disabled={!sel.length} onClick={() => op(sel.map(h => ({ op: 'delete', handle: h }))).then(() => setSel([]))} title="Удалить (Del)">✕</button>
        <span className="sep" />
        <span className="muted small">Лист</span>
        <input type="number" value={state.client[0]} style={{ width: 56 }} onChange={e => op({ op: 'page', w: Number(e.target.value) })} title="Ширина листа" />
        <input type="number" value={state.client[1]} style={{ width: 56 }} onChange={e => op({ op: 'page', h: Number(e.target.value) })} title="Высота листа" />
        <span className="spacer" />
        <span className="muted small mono">{cursor ? `${Math.round(cursor[0])}, ${Math.round(cursor[1])}` : ''} · {Math.round(view.k * 100)}%</span>
        <button className="small" onClick={() => setView({ x: 40, y: 40, k: 1 })}>100%</button>
      </div>
      <svg ref={svgRef} className="canvas" onMouseDown={onMouseDown} onMouseMove={onMouseMove} onMouseUp={onMouseUp} onContextMenu={e => e.preventDefault()} onDoubleClick={() => tool === 'polyline' && finishPolyline()}>
        <g transform={`translate(${view.x} ${view.y}) scale(${view.k})`}>
          <rect x={state.origin[0] - state.view[0]} y={state.origin[1] - state.view[1]} width={state.client[0]} height={state.client[1]} className="page" />
          <g dangerouslySetInnerHTML={{ __html: state.svg.replace(/<svg[^>]*>/, '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="' + state.view[2] + '" height="' + state.view[3] + '">').replace(/<rect width="100%" height="100%" fill="#ffffff"\/>/, '') }} />
        </g>
        {selObjs.map(o => {
          const box = preview?.resize && one?.handle === o.handle ? preview.resize : { x: o.x + (preview?.dx ?? 0), y: o.y + (preview?.dy ?? 0), w: o.w, h: o.h };
          const [x0, y0] = toScreen([box.x, box.y]);
          const w = box.w * k, h = box.h * k;
          return (
            <g key={o.handle}>
              <rect x={x0} y={y0} width={w} height={h} className="sel-box" />
              {one && tool === 'select' && ['nw', 'ne', 'sw', 'se'].map(c => (
                <rect key={c} data-handle-ui={`corner:${c}`} className="handle" x={(c[1] === 'w' ? x0 : x0 + w) - 4} y={(c[0] === 'n' ? y0 : y0 + h) - 4} width={8} height={8} />
              ))}
              {one && tool === 'points' && o.points?.map((pt, i) => {
                const q = preview?.point?.index === i ? preview.point.p : [pt[0] + o.x, pt[1] + o.y] as [number, number];
                const [sx, sy] = toScreen(q);
                return <circle key={i} data-handle-ui={`point:${i}`} className="handle point" cx={sx} cy={sy} r={4} />;
              })}
            </g>
          );
        })}
        {draft.length > 0 && cursor && (() => {
          const pts = tool === 'polyline' ? [...draft, cursor] : [draft[0], cursor];
          const scr = pts.map(toScreen);
          if (tool === 'line' || tool === 'polyline') return <polyline className="draft" points={scr.map(p => p.join(',')).join(' ')} />;
          const [a, b] = scr;
          const x = Math.min(a[0], b[0]), y = Math.min(a[1], b[1]), w = Math.abs(a[0] - b[0]), h = Math.abs(a[1] - b[1]);
          if (tool === 'ellipse' || tool === 'arc') return <ellipse className="draft" cx={x + w / 2} cy={y + h / 2} rx={w / 2} ry={h / 2} />;
          return <rect className="draft" x={x} y={y} width={w} height={h} rx={tool === 'roundrect' ? Math.min(w, h) * 0.2 : 0} />;
        })()}
      </svg>
      {textAsk && (
        <form className="text-ask" style={{ left: toScreen(textAsk.at)[0], top: toScreen(textAsk.at)[1] }}
          onSubmit={e => { e.preventDefault(); const t = textAsk.value.trim(); setTextAsk(null); if (t) op({ op: 'add', shape: 'text', points: [textAsk.at], text: t, fg: pen.color, w: Math.max(40, t.length * 8), h: 20 }).then(h => setSel([h])); }}>
          <input autoFocus type="text" value={textAsk.value} onChange={e => setTextAsk({ ...textAsk, value: e.target.value })} onKeyDown={e => e.key === 'Escape' && setTextAsk(null)} />
        </form>
      )}
      {one && (
        <div className="picture-props">
          <span className="muted">#{one.handle} {one.kind}</span>
          <label>Имя <input type="text" defaultValue={one.name} key={'n' + one.handle + one.name} onBlur={e => e.target.value !== one.name && op({ op: 'set', handle: one.handle, field: 'name', value: e.target.value })} /></label>
          {(['x', 'y', 'w', 'h'] as const).map(f => (
            <label key={f}>{f.toUpperCase()} <input type="number" className="mono" defaultValue={one[f]} key={f + one.handle + one[f]} onBlur={e => Number(e.target.value) !== one[f] && op({ op: 'resize', handle: one.handle, [f]: Number(e.target.value) })} /></label>
          ))}
          {one.pen && <label>Линия <input type="color" value={one.pen.color} onChange={e => op({ op: 'set', handle: one.handle, field: 'pen.color', value: e.target.value })} /><input type="number" className="mono" defaultValue={one.pen.width} key={'pw' + one.handle + one.pen.width} style={{ width: 44 }} onBlur={e => op({ op: 'set', handle: one.handle, field: 'pen.width', value: Number(e.target.value) })} /></label>}
          {one.brush && <label>Заливка <input type="color" value={one.brush.color} onChange={e => op({ op: 'set', handle: one.handle, field: 'brush.color', value: e.target.value })} />
            <select value={one.brush.style} onChange={e => op({ op: 'set', handle: one.handle, field: 'brush.style', value: Number(e.target.value) })}><option value={0}>сплошная</option><option value={1}>нет</option></select></label>}
          {one.points && <span className="muted">точек: {one.points.length}</span>}
        </div>
      )}
      {!editable && <div className="hint muted">Библиотечный имидж: только просмотр.</div>}
    </div>
  );
}

function resized(o: ObjectProps, corner: string, dx: number, dy: number) {
  let { x, y, w, h } = o;
  if (corner[1] === 'e') w = Math.max(1, w + dx); else { x += dx; w = Math.max(1, w - dx); }
  if (corner[0] === 's') h = Math.max(1, h + dy); else { y += dy; h = Math.max(1, h - dy); }
  return { x: Math.round(x), y: Math.round(y), w: Math.round(w), h: Math.round(h) };
}

function distToSeg(p: [number, number], a: [number, number], b: [number, number]) {
  const vx = b[0] - a[0], vy = b[1] - a[1];
  const l2 = vx * vx + vy * vy || 1;
  const t = Math.max(0, Math.min(1, ((p[0] - a[0]) * vx + (p[1] - a[1]) * vy) / l2));
  return Math.hypot(p[0] - (a[0] + t * vx), p[1] - (a[1] + t * vy));
}
