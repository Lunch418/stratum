// Графики наблюдаемых переменных: значения по тактам с сервера, SVG-линии
// с общей осью тактов и своей шкалой у каждой переменной.
import { useEffect, useRef, useState } from 'react';
import { api, type Trace } from '../api';
import { useStore } from '../store';

// цвета рядов — токены темы (--series-1…8), в тёмной теме светлее
const COLORS = Array.from({ length: 8 }, (_, i) => `var(--series-${i + 1})`);

export function Graphs() {
  const frame = useStore(s => s.frame);
  const setTraceCount = useStore(s => s.setTraceCount);
  const [traces, setTraces] = useState<Trace[]>([]);
  const [window_, setWindow] = useState(300);
  const [hidden, setHidden] = useState<Set<number>>(new Set());
  const box = useRef<HTMLDivElement>(null);
  const [size, setSize] = useState({ w: 600, h: 120 });

  useEffect(() => {
    let alive = true;
    const poll = async () => {
      try { const r = await api.traces(); if (alive) { setTraces(r.traces); setTraceCount(r.traces.length); } } catch { /* ядро недоступно */ }
      if (alive) setTimeout(poll, frame?.running ? 150 : 700);
    };
    poll();
    return () => { alive = false; };
  }, [frame?.running]);

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const ro = new ResizeObserver(() => setSize({ w: el.clientWidth, h: el.clientHeight }));
    ro.observe(el);
    return () => ro.disconnect();
  }, []);

  const shown = traces.filter(t => !hidden.has(t.id) && t.points.length);
  const tickMax = Math.max(0, ...traces.flatMap(t => t.points.length ? [t.points[t.points.length - 1][0]] : []));
  const tickMin = Math.max(0, tickMax - window_);
  const pad = { l: 8, r: 8, t: 8, b: 18 };
  const W = size.w, H = size.h;
  const px = (tick: number) => pad.l + ((tick - tickMin) / Math.max(1, tickMax - tickMin)) * (W - pad.l - pad.r);

  return (
    <>
      <div className="panel-title">Графики <span className="spacer" />
        <label className="muted" style={{ textTransform: 'none', letterSpacing: 0 }}>окно
          <select value={window_} onChange={e => setWindow(Number(e.target.value))} style={{ marginLeft: 4 }}>
            {[100, 300, 1000, 2000].map(n => <option key={n} value={n}>{n} тактов</option>)}
          </select>
        </label>
        <span className="muted">{traces.length}</span>
      </div>
      {!traces.length && <div className="empty">Графиков пока нет. Нажмите ∿ у переменной в инспекторе, чтобы следить за её значением по тактам.</div>}
      <div className="graphs" style={{ display: traces.length ? 'grid' : 'none' }}>
        <div className="legend">
          {traces.map((t, i) => {
            const last = t.points.length ? t.points[t.points.length - 1][1] : null;
            return (
              <div key={t.id} className={`item${hidden.has(t.id) ? ' off' : ''}`} onClick={() => setHidden(h => { const n = new Set(h); n.has(t.id) ? n.delete(t.id) : n.add(t.id); return n; })}>
                <span className="swatch" style={{ background: COLORS[i % COLORS.length] }} />
                <span className="name" title={`${t.path}.${t.var}`}>{t.path.split('\\').pop()}.{t.var}</span>
                <span className="mono val">{last === null ? '—' : fmt(last)}</span>
                <button aria-label="Убрать" className="small ghost" title="Убрать" onClick={e => { e.stopPropagation(); api.traceRemove(t.id); }}>×</button>
              </div>
            );
          })}
        </div>
        <div ref={box} className="plot">
          <svg width={W} height={H}>
            {[0, 0.5, 1].map(f => <line key={f} x1={pad.l} x2={W - pad.r} y1={pad.t + f * (H - pad.t - pad.b)} y2={pad.t + f * (H - pad.t - pad.b)} className="grid" />)}
            {shown.map(t => {
              const i = traces.indexOf(t);
              const pts = t.points.filter(p => p[0] >= tickMin);
              if (pts.length < 2) return null;
              let lo = Infinity, hi = -Infinity;
              for (const p of pts) { lo = Math.min(lo, p[1]); hi = Math.max(hi, p[1]); }
              if (hi - lo < 1e-9) { hi = lo + 1; lo -= 1; }
              const py = (v: number) => pad.t + (1 - (v - lo) / (hi - lo)) * (H - pad.t - pad.b);
              const d = pts.map((p, k) => `${k ? 'L' : 'M'} ${px(p[0]).toFixed(1)} ${py(p[1]).toFixed(1)}`).join(' ');
              return (
                <g key={t.id}>
                  <path d={d} fill="none" style={{ stroke: COLORS[i % COLORS.length] }} strokeWidth={1.5} strokeLinejoin="round" />
                  <text x={W - pad.r - 2} y={py(hi) + 10} textAnchor="end" style={{ fill: COLORS[i % COLORS.length] }} className="tick">{fmt(hi)}</text>
                  <text x={W - pad.r - 2} y={py(lo) - 2} textAnchor="end" style={{ fill: COLORS[i % COLORS.length] }} className="tick">{fmt(lo)}</text>
                </g>
              );
            })}
            <text x={pad.l} y={H - 4} className="tick muted">такт {tickMin}</text>
            <text x={W - pad.r} y={H - 4} textAnchor="end" className="tick muted">{tickMax}</text>
          </svg>
        </div>
      </div>
    </>
  );
}

function fmt(v: number) {
  if (!isFinite(v)) return String(v);
  const a = Math.abs(v);
  return a >= 1e6 || (a < 1e-3 && a > 0) ? v.toExponential(2) : String(Math.round(v * 1000) / 1000);
}
