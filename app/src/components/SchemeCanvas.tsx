// Холст схемы: графика листа (SVG из ядра), блоки имиджей, связи;
// панорама, зум к курсору, перетаскивание блоков, вход в подсхему.
import { useEffect, useMemo, useRef, useState } from 'react';
import { api, type ClassInfo } from '../api';
import { classByName, useStore } from '../store';

const NODE_W = 120;
const NODE_H = 36;
const ICON = 32;

interface View { x: number; y: number; k: number }

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
  const klass = classByName(project, path[path.length - 1]);
  const [view, setView] = useState<View>({ x: 0, y: 0, k: 1 });
  const [background, setBackground] = useState<{ svg: string; bounds: { x: number; y: number; w: number; h: number } | null }>({ svg: '', bounds: null });
  const [drag, setDrag] = useState<{ handle: number; dx: number; dy: number } | null>(null);
  const [pan, setPan] = useState<{ sx: number; sy: number; vx: number; vy: number } | null>(null);
  const svgRef = useRef<SVGSVGElement>(null);

  useEffect(() => {
    if (!klass) return;
    api.scheme(klass.name).then(setBackground).catch(() => setBackground({ svg: '', bounds: null }));
  }, [klass?.name]);

  // показать всё при смене схемы
  useEffect(() => {
    if (!klass || !svgRef.current) return;
    fitAll();
  }, [klass?.name, background.bounds?.w]);

  // подгонка по блокам имиджей; графика листа может быть много больше
  const bounds = useMemo(() => {
    if (!klass) return null;
    let x0 = Infinity, y0 = Infinity, x1 = -Infinity, y1 = -Infinity;
    for (const c of klass.children) {
      const { w, h } = nodeSize(c);
      x0 = Math.min(x0, c.x); y0 = Math.min(y0, c.y); x1 = Math.max(x1, c.x + w); y1 = Math.max(y1, c.y + h);
    }
    if (!klass.children.length && background.bounds) {
      const b = background.bounds;
      x0 = Math.min(x0, b.x); y0 = Math.min(y0, b.y); x1 = Math.max(x1, b.x + b.w); y1 = Math.max(y1, b.y + b.h);
    }
    return isFinite(x0) ? { x0, y0, x1, y1 } : null;
  }, [klass, background.bounds]);

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
    if (e.button === 1 || (e.button === 0 && (e.target as Element).closest('.node') === null)) {
      setPan({ sx: e.clientX, sy: e.clientY, vx: view.x, vy: view.y });
      if (e.button === 0) select(klass?.name ?? null);
    }
  }

  function onMouseMove(e: React.MouseEvent) {
    if (pan) setView(v => ({ ...v, x: pan.vx + e.clientX - pan.sx, y: pan.vy + e.clientY - pan.sy }));
    else if (drag && klass) {
      const p = toScene(e);
      const gx = Math.round((p.x - drag.dx) / 8) * 8, gy = Math.round((p.y - drag.dy) / 8) * 8;
      updateClass({ ...klass, children: klass.children.map(c => c.handle === drag.handle ? { ...c, x: gx, y: gy } : c) });
    }
  }

  function onMouseUp() {
    if (drag && klass) {
      const c = klass.children.find(c => c.handle === drag.handle);
      if (c) api.moveChild(klass.name, c.handle, c.x, c.y).then(() => { showToast('Перемещено'); useStore.getState().markUnsaved(); });
    }
    setDrag(null); setPan(null);
  }

  if (!klass) return <div className="scheme" />;

  const centers = new Map(klass.children.map(c => { const { w, h } = nodeSize(c); return [c.handle, { x: c.x + w / 2, y: c.y + h / 2 }]; }));

  return (
    <div className="scheme" onWheel={onWheel}>
      <svg ref={svgRef} className="canvas" onMouseDown={onMouseDown} onMouseMove={onMouseMove} onMouseUp={onMouseUp} onMouseLeave={onMouseUp}
        onContextMenu={e => e.preventDefault()}>
        <defs>
          <pattern id="grid" width={16 * view.k} height={16 * view.k} patternUnits="userSpaceOnUse" x={view.x} y={view.y}>
            <path className="grid" d={`M ${16 * view.k} 0 L 0 0 0 ${16 * view.k}`} fill="none" />
          </pattern>
        </defs>
        <rect width="100%" height="100%" fill="url(#grid)" />
        <g transform={`translate(${view.x} ${view.y}) scale(${view.k})`}>
          {background.svg && background.bounds && (
            <g opacity="0.85" dangerouslySetInnerHTML={{ __html: background.svg.replace(/<svg[^>]*>/, `<svg x="${background.bounds.x}" y="${background.bounds.y}" width="${background.bounds.w}" height="${background.bounds.h}" viewBox="0 0 ${background.bounds.w} ${background.bounds.h}" xmlns="http://www.w3.org/2000/svg">`) }} />
          )}
          {klass.links.map(l => {
            const a = centers.get(l.source), b = centers.get(l.target);
            if (!a || !b) return null;
            const title = l.vars.map(([p, q]) => `${p} → ${q}`).join(', ');
            const mx = (a.x + b.x) / 2;
            return <path key={l.handle} className="link" d={`M ${a.x} ${a.y} C ${mx} ${a.y}, ${mx} ${b.y}, ${b.x} ${b.y}`}><title>{title}</title></path>;
          })}
          {klass.children.map(c => {
            const { w, h, label } = nodeSize(c);
            const cls = classByName(project, c.class);
            const selected = selectedClass === c.class;
            return (
              <g key={c.handle} className={`node${selected ? ' selected' : ''}`} transform={`translate(${c.x} ${c.y})`}
                onMouseDown={e => { e.stopPropagation(); const p = toScene(e); setDrag({ handle: c.handle, dx: p.x - c.x, dy: p.y - c.y }); select(c.class, instanceIndexFor(c.handle)); }}
                onDoubleClick={e => { e.stopPropagation(); if (cls && (cls.children.length || cls.hasScheme)) enterScheme(c.class); }}>
                <rect className="box" width={w} height={h} />
                <image href={api.iconUrl(c.class)} x={4} y={(h - ICON) / 2} width={ICON} height={ICON} />
                <text x={ICON + 10} y={h / 2 + 4}>{label}<title>{c.name ? `${c.name} [${c.class}]` : c.class}</title></text>
                {cls?.vars.filter(v => !v.local).slice(0, 6).map((v, i) => (
                  <circle key={v.name} cx={i % 2 ? w : 0} cy={8 + Math.floor(i / 2) * 10} r={4} fill={`var(--type-${v.type.toLowerCase()})`} stroke="var(--bg-surface)"><title>{v.name}: {v.type}</title></circle>
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
