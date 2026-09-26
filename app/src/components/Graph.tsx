// Граф зависимостей переменных схемы: узлы — переменные экземпляров
// (сгруппированы по имиджу), дуги — связи схемы и присваивания в текстах.
// Раскладка силовая, считается в браузере.
import { useEffect, useMemo, useRef, useState } from 'react';
import { useStore } from '../store';

interface GraphData { instances: { id: string; label: string; class: string }[]; nodes: string[]; edges: { from: string; to: string; kind: 'text' | 'link' }[] }
interface Node { id: string; inst: string; var: string; x: number; y: number; vx: number; vy: number }

const COLORS = ['#2f6fdb', '#e07b39', '#1f9d55', '#8a63d2', '#c93b3b', '#1f9e9e', '#d08a00', '#6b7280', '#b5651d', '#3a86ff'];

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

  // силовая раскладка: экземпляры — кластеры по кругу, переменные тянутся к своему
  useEffect(() => {
    if (!data) { setNodes([]); return; }
    const insts = data.instances.map(i => i.id);
    const centers = new Map(insts.map((id, k) => {
      const a = (k / Math.max(1, insts.length)) * Math.PI * 2;
      const r = Math.min(size.w, size.h) * 0.3;
      return [id, { x: size.w / 2 + r * Math.cos(a), y: size.h / 2 + r * Math.sin(a) }];
    }));
    const ns: Node[] = shown.nodes.map((id, k) => {
      const dot = id.indexOf('.');
      const inst = id.slice(0, dot), v = id.slice(dot + 1);
      const c = centers.get(inst) ?? { x: size.w / 2, y: size.h / 2 };
      return { id, inst, var: v, x: c.x + Math.cos(k) * 30, y: c.y + Math.sin(k) * 30, vx: 0, vy: 0 };
    });
    const index = new Map(ns.map((n, i) => [n.id, i]));
    for (let iter = 0; iter < 250; iter++) {
      for (let i = 0; i < ns.length; i++) {
        const a = ns[i];
        const c = centers.get(a.inst) ?? { x: size.w / 2, y: size.h / 2 };
        a.vx += (c.x - a.x) * 0.05; a.vy += (c.y - a.y) * 0.05;
        for (let j = i + 1; j < ns.length; j++) {
          const b = ns[j];
          let dx = a.x - b.x, dy = a.y - b.y;
          let d2 = dx * dx + dy * dy;
          if (d2 < 1) { dx = Math.random() - 0.5; dy = Math.random() - 0.5; d2 = 1; }
          const f = 400 / d2;
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
        const f = (d - want) * 0.01;
        a.vx += dx / d * f; a.vy += dy / d * f; b.vx -= dx / d * f; b.vy -= dy / d * f;
      }
      for (const n of ns) {
        n.x = Math.max(60, Math.min(size.w - 80, n.x + n.vx * 0.5));
        n.y = Math.max(40, Math.min(size.h - 130, n.y + n.vy * 0.5));
        n.vx *= 0.6; n.vy *= 0.6;
      }
    }
    setNodes(ns);
  }, [shown, size.w, size.h]);

  const colorOf = (inst: string) => COLORS[Math.max(0, (data?.instances.findIndex(i => i.id === inst) ?? 0)) % COLORS.length];
  const pos = new Map(nodes.map(n => [n.id, n]));
  const related = new Set<string>();
  if (hover) for (const e of shown.edges) { if (e.from === hover) related.add(e.to); if (e.to === hover) related.add(e.from); }

  return (
    <div className="graph-view" ref={box}>
      <div className="toolbar">
        <label className="small" style={{ display: 'flex', gap: 4, alignItems: 'center', background: 'var(--bg-surface)', padding: '2px 8px', border: '1px solid var(--border)', borderRadius: 4 }}>
          <input type="checkbox" checked={onlyLinked} onChange={e => setOnlyLinked(e.target.checked)} />только связанные
        </label>
        <label className="small" style={{ display: 'flex', gap: 4, alignItems: 'center', background: 'var(--bg-surface)', padding: '2px 8px', border: '1px solid var(--border)', borderRadius: 4 }}>
          <input type="checkbox" checked={withText} onChange={e => setWithText(e.target.checked)} />присваивания в текстах
        </label>
        <span className="muted small" style={{ alignSelf: 'center', padding: '0 6px' }}>{shown.nodes.length} перем. · {shown.edges.length} дуг</span>
      </div>
      {!data && <div className="empty">Выберите имидж со схемой — здесь появится граф связей его переменных.</div>}
      {data && !shown.nodes.length && <div className="empty">В схеме «{klass}» нет зависимостей между переменными.</div>}
      <svg width={size.w} height={size.h}>
        <defs>
          <marker id="arrow" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
            <path d="M 0 0 L 10 5 L 0 10 z" fill="var(--text-muted)" />
          </marker>
        </defs>
        {shown.edges.map((e, i) => {
          const a = pos.get(e.from), b = pos.get(e.to);
          if (!a || !b) return null;
          const dim = hover && e.from !== hover && e.to !== hover;
          return <line key={i} x1={a.x} y1={a.y} x2={b.x} y2={b.y} stroke={e.kind === 'link' ? 'var(--type-float)' : 'var(--text-muted)'} strokeWidth={e.kind === 'link' ? 1.6 : 1} strokeDasharray={e.kind === 'link' ? undefined : '3 3'} opacity={dim ? 0.15 : 0.8} markerEnd="url(#arrow)" />;
        })}
        {nodes.map(n => {
          const inst = data?.instances.find(i => i.id === n.inst);
          const dim = hover && hover !== n.id && !related.has(n.id);
          return (
            <g key={n.id} transform={`translate(${n.x} ${n.y})`} opacity={dim ? 0.25 : 1} onMouseEnter={() => setHover(n.id)} onMouseLeave={() => setHover(null)}>
              <circle r={5} fill={colorOf(n.inst)} stroke="var(--bg-surface)" />
              <text x={8} y={4} className="label">{n.var}</text>
              <title>{inst?.label ?? n.inst}.{n.var} [{inst?.class}]</title>
            </g>
          );
        })}
      </svg>
      <div className="legend-floating">
        {data?.instances.map(i => <span key={i.id}><span className="swatch" style={{ background: colorOf(i.id) }} />{i.label}</span>)}
      </div>
    </div>
  );
}
