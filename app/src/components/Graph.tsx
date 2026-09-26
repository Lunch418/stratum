// Граф зависимостей переменных схемы: узлы — переменные экземпляров
// (сгруппированы по имиджу), дуги — связи схемы и присваивания в текстах.
// Раскладка силовая, считается в браузере на листе, размер которого растёт
// с числом узлов; вид можно двигать мышью и масштабировать колесом. Подписи
// не наезжают друг на друга: каждая ставится туда, где свободно, а если
// места нет — прячется до наведения.
import { useEffect, useMemo, useRef, useState } from 'react';
import { useStore } from '../store';

interface GraphData { instances: { id: string; label: string; class: string }[]; nodes: string[]; edges: { from: string; to: string; kind: 'text' | 'link' }[] }
interface Node { id: string; inst: string; var: string; x: number; y: number; vx: number; vy: number }
interface Label { x: number; y: number; anchor: 'start' | 'end' | 'middle' }

// цвета экземпляров — токены темы, те же, что у рядов на графиках
const COLORS = Array.from({ length: 8 }, (_, i) => `var(--series-${i + 1})`);
const CHAR_W = 6.7, LABEL_H = 13;

// «Показать всё»: сверху место под панель, снизу — под легенду, справа — под подписи
function fitView(nodes: Node[], size: { w: number; h: number }) {
  if (!nodes.length) return { x: 0, y: 0, k: 1 };
  let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
  for (const n of nodes) { x0 = Math.min(x0, n.x); y0 = Math.min(y0, n.y); x1 = Math.max(x1, n.x); y1 = Math.max(y1, n.y); }
  const top = 48, bottom = 56, side = 24, labelRoom = 90;
  const k = Math.min(1.4, (size.w - side * 2 - labelRoom) / Math.max(1, x1 - x0), (size.h - top - bottom) / Math.max(1, y1 - y0));
  return { k, x: side + (size.w - side * 2 - labelRoom - (x1 - x0) * k) / 2 - x0 * k, y: top + (size.h - top - bottom - (y1 - y0) * k) / 2 - y0 * k };
}

export function Graph() {
  const path = useStore(s => s.schemePath);
  const klass = path[path.length - 1];
  const [data, setData] = useState<GraphData | null>(null);
  const [nodes, setNodes] = useState<Node[]>([]);
  const [onlyLinked, setOnlyLinked] = useState(true);
  const [withText, setWithText] = useState(false);
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 800, h: 500 });
  const [hover, setHover] = useState<string | null>(null);
  const [view, setView] = useState({ x: 0, y: 0, k: 1 });
  const [pan, setPan] = useState<{ sx: number; sy: number; vx: number; vy: number } | null>(null);

  useEffect(() => {
    if (!klass) return;
    fetch(`/api/graph/${encodeURIComponent(klass)}`).then(r => r.json()).then(setData).catch(() => setData(null));
  }, [klass]);

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSize({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const shown = useMemo(() => {
    if (!data) return { nodes: [] as string[], edges: [] as GraphData['edges'] };
    // по умолчанию — переменные, участвующие в связях схемы; присваивания
    // из текстов добавляют дуги между уже показанными узлами
    const linkDeg = new Map<string, number>();
    for (const e of data.edges) if (e.kind === 'link') { linkDeg.set(e.from, 1); linkDeg.set(e.to, 1); }
    const textDeg = new Map<string, number>();
    for (const e of data.edges) if (e.kind === 'text') { textDeg.set(e.from, 1); textDeg.set(e.to, 1); }
    const nodes = data.nodes.filter(n => !onlyLinked || linkDeg.has(n) || (withText && textDeg.has(n)));
    const set = new Set(nodes);
    const edges = data.edges.filter(e => set.has(e.from) && set.has(e.to) && (withText || e.kind === 'link'));
    return { nodes, edges };
  }, [data, onlyLinked, withText]);

  // силовая раскладка: экземпляры — кластеры в узлах решётки, переменные
  // тянутся к своему; лист тем больше, чем больше узлов
  useEffect(() => {
    if (!data) { setNodes([]); return; }
    const instOf = (id: string) => id.slice(0, id.indexOf('.'));
    const used = data.instances.map(i => i.id).filter(id => shown.nodes.some(n => instOf(n) === id));
    const aspect = Math.max(0.5, Math.min(3, size.w / Math.max(1, size.h)));
    const cols = Math.max(1, Math.ceil(Math.sqrt(used.length * aspect)));
    const rows = Math.max(1, Math.ceil(used.length / cols));
    const cell = Math.max(200, Math.sqrt(shown.nodes.length / Math.max(1, used.length)) * 80);
    const W = cols * cell, H = rows * cell;
    const centers = new Map(used.map((id, k) => [id, { x: (k % cols + 0.5) * cell, y: (Math.floor(k / cols) + 0.5) * cell }]));
    const ns: Node[] = shown.nodes.map((id, k) => {
      const inst = instOf(id), v = id.slice(inst.length + 1);
      const c = centers.get(inst) ?? { x: W / 2, y: H / 2 };
      return { id, inst, var: v, x: c.x + Math.cos(k) * 30, y: c.y + Math.sin(k) * 30, vx: 0, vy: 0 };
    });
    const index = new Map(ns.map((n, i) => [n.id, i]));
    for (let iter = 0; iter < 250; iter++) {
      for (let i = 0; i < ns.length; i++) {
        const a = ns[i];
        const c = centers.get(a.inst) ?? { x: W / 2, y: H / 2 };
        a.vx += (c.x - a.x) * 0.05; a.vy += (c.y - a.y) * 0.05;
        for (let j = i + 1; j < ns.length; j++) {
          const b = ns[j];
          let dx = a.x - b.x, dy = a.y - b.y;
          let d2 = dx * dx + dy * dy;
          if (d2 < 1) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; d2 = 1; }
          if (d2 > 90000) continue;
          const f = 500 / d2;
          const fx = dx * f, fy = dy * f;
          a.vx += fx; a.vy += fy; b.vx -= fx; b.vy -= fy;
        }
      }
      for (const e of shown.edges) {
        const a = ns[index.get(e.from)!], b = ns[index.get(e.to)!];
        if (!a || !b) continue;
        const dx = b.x - a.x, dy = b.y - a.y;
        const d = Math.hypot(dx, dy) || 1;
        const want = e.kind === 'link' ? 140 : 60;
        const f = (d - want) * 0.004;
        a.vx += dx / d * f; a.vy += dy / d * f; b.vx -= dx / d * f; b.vy -= dy / d * f;
      }
      for (const n of ns) {
        // без жёстких стенок: узлы не липнут к краю, вид потом подгоняется
        n.x += n.vx * 0.5; n.y += n.vy * 0.5;
        n.vx *= 0.6; n.vy *= 0.6;
      }
    }
    setNodes(ns);
    setView(fitView(ns, size));
  }, [shown, size.w, size.h]);

  const fit = () => setView(fitView(nodes, size));

  const sx = (n: { x: number }) => n.x * view.k + view.x;
  const sy = (n: { y: number }) => n.y * view.k + view.y;

  // подписи: справа, слева, сверху или снизу от точки — где не занято
  // другими подписями и точками; иначе подпись видна только при наведении
  const labels = useMemo(() => {
    const sx = (n: { x: number }) => n.x * view.k + view.x;
    const sy = (n: { y: number }) => n.y * view.k + view.y;
    const out = new Map<string, Label>();
    const boxes: [number, number, number, number][] = nodes.map(n => [sx(n) - 5, sy(n) - 5, sx(n) + 5, sy(n) + 5]);
    const free = (b: [number, number, number, number]) => b[0] >= 0 && b[2] <= size.w && boxes.every(o => b[2] <= o[0] || b[0] >= o[2] || b[3] <= o[1] || b[1] >= o[3]);
    // сначала подписи узлов со связями — они важнее
    const deg = new Map<string, number>();
    for (const e of shown.edges) { deg.set(e.from, (deg.get(e.from) ?? 0) + 1); deg.set(e.to, (deg.get(e.to) ?? 0) + 1); }
    for (const n of [...nodes].sort((a, b) => (deg.get(b.id) ?? 0) - (deg.get(a.id) ?? 0))) {
      const x = sx(n), y = sy(n), w = n.var.length * CHAR_W;
      const tries: [Label, [number, number, number, number]][] = [
        [{ x: x + 8, y: y + 4, anchor: 'start' }, [x + 7, y - LABEL_H / 2, x + 9 + w, y + LABEL_H / 2]],
        [{ x: x - 8, y: y + 4, anchor: 'end' }, [x - 9 - w, y - LABEL_H / 2, x - 7, y + LABEL_H / 2]],
        [{ x, y: y - 9, anchor: 'middle' }, [x - w / 2 - 1, y - 8 - LABEL_H, x + w / 2 + 1, y - 7]],
        [{ x, y: y + 17, anchor: 'middle' }, [x - w / 2 - 1, y + 7, x + w / 2 + 1, y + 8 + LABEL_H]],
      ];
      const ok = tries.find(([, b]) => free(b));
      if (ok) { out.set(n.id, ok[0]); boxes.push(ok[1]); }
    }
    return out;
  }, [nodes, view, size.w, shown.edges]);

  const colorOf = (inst: string) => COLORS[Math.max(0, (data?.instances.findIndex(i => i.id === inst) ?? 0)) % COLORS.length];
  const pos = new Map(nodes.map(n => [n.id, n]));
  const related = new Set<string>();
  if (hover) for (const e of shown.edges) { if (e.from === hover) related.add(e.to); if (e.to === hover) related.add(e.from); }
  const hidden = nodes.length - labels.size;

  const onWheel = (e: React.WheelEvent) => {
    const r = box.current!.getBoundingClientRect();
    const mx = e.clientX - r.left, my = e.clientY - r.top;
    const k = Math.max(0.1, Math.min(4, view.k * (e.deltaY < 0 ? 1.15 : 1 / 1.15)));
    setView(v => ({ k, x: mx - (mx - v.x) * (k / v.k), y: my - (my - v.y) * (k / v.k) }));
  };

  return (
    <div className="graph-view" ref={box}>
      <div className="toolbar">
        <label className="small graph-opt">
          <input type="checkbox" checked={onlyLinked} onChange={e => setOnlyLinked(e.target.checked)} />только связанные
        </label>
        <label className="small graph-opt">
          <input type="checkbox" checked={withText} onChange={e => setWithText(e.target.checked)} />присваивания в текстах
        </label>
        <button className="small" onClick={fit} title="Показать весь граф">Показать всё</button>
        <span className="muted small graph-count" title={hidden ? `${hidden} подписей скрыто, чтобы не наезжали друг на друга — наведите на точку или приблизьте колесом` : undefined}>
          {shown.nodes.length} перем. · {shown.edges.length} дуг{hidden ? ` · ${hidden} подп. скрыто` : ''}
        </span>
      </div>
      {!data && <div className="empty">Выберите имидж со схемой — здесь появится граф связей его переменных.</div>}
      {data && !shown.nodes.length && <div className="empty">В схеме «{klass}» нет зависимостей между переменными.</div>}
      <svg width={size.w} height={size.h} onWheel={onWheel} className={pan ? 'panning' : undefined}
        onMouseDown={e => { if (e.button === 0) setPan({ sx: e.clientX, sy: e.clientY, vx: view.x, vy: view.y }); }}
        onMouseMove={e => { if (pan) setView(v => ({ ...v, x: pan.vx + e.clientX - pan.sx, y: pan.vy + e.clientY - pan.sy })); }}
        onMouseUp={() => setPan(null)} onMouseLeave={() => setPan(null)}>
        <defs>
          <marker id="graph-arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--text-muted)" />
          </marker>
        </defs>
        {shown.edges.map((e, i) => {
          const a = pos.get(e.from), b = pos.get(e.to);
          if (!a || !b) return null;
          const dim = hover && e.from !== hover && e.to !== hover;
          return <line key={i} x1={sx(a)} y1={sy(a)} x2={sx(b)} y2={sy(b)} stroke={e.kind === 'link' ? 'var(--type-float)' : 'var(--text-muted)'} strokeWidth={e.kind === 'link' ? 1.4 : 1} strokeDasharray={e.kind === 'link' ? undefined : '3 3'} opacity={dim ? 0.12 : 0.7} markerEnd="url(#graph-arrow)" />;
        })}
        {nodes.map(n => {
          const inst = data?.instances.find(i => i.id === n.inst);
          const lit = hover === n.id || related.has(n.id);
          const dim = hover && !lit;
          const l = labels.get(n.id) ?? (lit ? { x: sx(n) + 8, y: sy(n) + 4, anchor: 'start' as const } : null);
          return (
            <g key={n.id} opacity={dim ? 0.25 : 1} onMouseEnter={() => setHover(n.id)} onMouseLeave={() => setHover(null)}>
              <circle cx={sx(n)} cy={sy(n)} r={lit ? 6 : 4.5} fill={colorOf(n.inst)} stroke="var(--bg-canvas)" strokeWidth={1.5} />
              {l && <text x={l.x} y={l.y} textAnchor={l.anchor} className={`label${lit ? ' lit' : ''}`}>{n.var}</text>}
              <title>{inst?.label ?? n.inst}.{n.var} [{inst?.class}]</title>
            </g>
          );
        })}
      </svg>
      <div className="legend-floating">
        {data?.instances.filter(i => nodes.some(n => n.inst === i.id)).map(i => <span key={i.id}><span className="swatch" style={{ background: colorOf(i.id) }} />{i.label}</span>)}
      </div>
    </div>
  );
}
