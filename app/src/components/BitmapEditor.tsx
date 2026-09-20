// Битовый редактор (панель «Битовый редактор» оригинала): карандаш, ластик,
// линия, прямоугольник, эллипс, закраска, распылитель, пипетка, цвет, сетка,
// зум. Пиксели читаются и пишутся операциями dibget/dibset редактора рисунка.
import { useEffect, useRef, useState } from 'react';
import { Icon } from './Icon';

type Tool = 'pencil' | 'line' | 'rect' | 'frect' | 'ellipse' | 'fill' | 'spray' | 'pick' | 'eraser';
const TOOLS: [Tool, string, string][] = [
  ['pencil', 'pencil', 'Карандаш'], ['eraser', 'eraser', 'Ластик (белый)'], ['line', 'line', 'Линия'], ['rect', 'rect', 'Прямоугольник'], ['frect', 'frect', 'Закрашенный прямоугольник'],
  ['ellipse', 'ellipse', 'Эллипс'], ['fill', 'fill', 'Закраска области'], ['spray', 'spray', 'Распылитель'], ['pick', 'pick', 'Пипетка'],
];
type RGB = [number, number, number];

interface Props { klass: string; kind: string; handle: number; onClose: () => void; onSaved: () => void }

export function BitmapEditor({ klass, kind, handle, onClose, onSaved }: Props) {
  const [size, setSize] = useState<[number, number]>([0, 0]);
  const pixels = useRef<Uint8ClampedArray | null>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const [tool, setTool] = useState<Tool>('pencil');
  const [color, setColor] = useState('#000000');
  const [zoom, setZoom] = useState(8);
  const [grid, setGrid] = useState(true);
  const start = useRef<[number, number] | null>(null);
  const setStart = (p: [number, number] | null) => { start.current = p; };
  const [error, setError] = useState('');
  const backup = useRef<Uint8ClampedArray | null>(null);
  const history = useRef<Uint8ClampedArray[]>([]);

  useEffect(() => {
    fetch(`/api/picture/${encodeURIComponent(klass)}?kind=${kind}`, { method: 'POST', body: JSON.stringify({ op: 'dibget', handle }) })
      .then(async r => { const j = await r.json(); if (!r.ok) throw new Error(j.error); return j as { w: number; h: number; rgb: string }; })
      .then(j => {
        const px = new Uint8ClampedArray(j.w * j.h * 4);
        for (let i = 0; i < j.w * j.h; i++) { px[i * 4] = parseInt(j.rgb.substr(i * 6, 2), 16); px[i * 4 + 1] = parseInt(j.rgb.substr(i * 6 + 2, 2), 16); px[i * 4 + 2] = parseInt(j.rgb.substr(i * 6 + 4, 2), 16); px[i * 4 + 3] = 255; }
        pixels.current = px; setSize([j.w, j.h]);
        setZoom(Math.max(1, Math.min(16, Math.floor(480 / Math.max(j.w, j.h)))));
      }).catch(e => setError(String(e)));
  }, [klass, kind, handle]);

  useEffect(() => { draw(); }, [size, zoom, grid]);

  function draw() {
    const c = canvas.current, px = pixels.current;
    if (!c || !px || !size[0]) return;
    const [w, h] = size;
    c.width = w * zoom; c.height = h * zoom;
    const ctx = c.getContext('2d')!;
    const off = document.createElement('canvas'); off.width = w; off.height = h;
    const img = off.getContext('2d')!.createImageData(w, h); img.data.set(px); off.getContext('2d')!.putImageData(img, 0, 0);
    ctx.imageSmoothingEnabled = false;
    ctx.drawImage(off, 0, 0, w * zoom, h * zoom);
    if (grid && zoom >= 4) {
      ctx.strokeStyle = 'rgba(0,0,0,.15)'; ctx.lineWidth = 1; ctx.beginPath();
      for (let x = 0; x <= w; x++) { ctx.moveTo(x * zoom + .5, 0); ctx.lineTo(x * zoom + .5, h * zoom); }
      for (let y = 0; y <= h; y++) { ctx.moveTo(0, y * zoom + .5); ctx.lineTo(w * zoom, y * zoom + .5); }
      ctx.stroke();
    }
  }

  const rgb = (hex: string): RGB => [parseInt(hex.slice(1, 3), 16), parseInt(hex.slice(3, 5), 16), parseInt(hex.slice(5, 7), 16)];
  function put(x: number, y: number, c: RGB) {
    const [w, h] = size; const px = pixels.current!;
    if (x < 0 || y < 0 || x >= w || y >= h) return;
    const o = (y * w + x) * 4; px[o] = c[0]; px[o + 1] = c[1]; px[o + 2] = c[2];
  }
  function line(a: [number, number], b: [number, number], c: RGB) {
    let [x0, y0] = a; const [x1, y1] = b;
    const dx = Math.abs(x1 - x0), dy = -Math.abs(y1 - y0), sx = x0 < x1 ? 1 : -1, sy = y0 < y1 ? 1 : -1; let err = dx + dy;
    for (;;) { put(x0, y0, c); if (x0 === x1 && y0 === y1) break; const e2 = 2 * err; if (e2 >= dy) { err += dy; x0 += sx; } if (e2 <= dx) { err += dx; y0 += sy; } }
  }
  function shape(a: [number, number], b: [number, number], c: RGB, t: Tool) {
    const x0 = Math.min(a[0], b[0]), x1 = Math.max(a[0], b[0]), y0 = Math.min(a[1], b[1]), y1 = Math.max(a[1], b[1]);
    if (t === 'line') line(a, b, c);
    else if (t === 'rect') { line([x0, y0], [x1, y0], c); line([x1, y0], [x1, y1], c); line([x1, y1], [x0, y1], c); line([x0, y1], [x0, y0], c); }
    else if (t === 'frect') { for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) put(x, y, c); }
    else if (t === 'ellipse') {
      const cx = (x0 + x1) / 2, cy = (y0 + y1) / 2, rx = Math.max(.5, (x1 - x0) / 2), ry = Math.max(.5, (y1 - y0) / 2);
      const n = Math.max(16, Math.round((rx + ry) * 4)); let prev: [number, number] | null = null;
      for (let i = 0; i <= n; i++) { const t2 = i / n * Math.PI * 2; const p: [number, number] = [Math.round(cx + rx * Math.cos(t2)), Math.round(cy + ry * Math.sin(t2))]; if (prev) line(prev, p, c); prev = p; }
    }
  }
  function fill(x: number, y: number, c: RGB) {
    const [w, h] = size; const px = pixels.current!;
    const o0 = (y * w + x) * 4; const target = [px[o0], px[o0 + 1], px[o0 + 2]];
    if (target[0] === c[0] && target[1] === c[1] && target[2] === c[2]) return;
    const stack = [[x, y]]; const seen = new Uint8Array(w * h);
    while (stack.length) {
      const [sx, sy] = stack.pop()!; if (sx < 0 || sy < 0 || sx >= w || sy >= h) continue;
      const i = sy * w + sx; if (seen[i]) continue; const o = i * 4;
      if (px[o] !== target[0] || px[o + 1] !== target[1] || px[o + 2] !== target[2]) continue;
      seen[i] = 1; px[o] = c[0]; px[o + 1] = c[1]; px[o + 2] = c[2];
      stack.push([sx + 1, sy], [sx - 1, sy], [sx, sy + 1], [sx, sy - 1]);
    }
  }
  function spray(p: [number, number], c: RGB) { for (let i = 0; i < 12; i++) { const a = Math.random() * Math.PI * 2, r = Math.random() * 4; put(Math.round(p[0] + Math.cos(a) * r), Math.round(p[1] + Math.sin(a) * r), c); } }

  const at = (e: React.MouseEvent): [number, number] => { const r = canvas.current!.getBoundingClientRect(); return [Math.floor((e.clientX - r.left) / zoom), Math.floor((e.clientY - r.top) / zoom)]; };
  const remember = () => { history.current.push(new Uint8ClampedArray(pixels.current!)); if (history.current.length > 50) history.current.shift(); };
  const current = (): RGB => tool === 'eraser' ? [255, 255, 255] : rgb(color);
  function onDown(e: React.MouseEvent) {
    if (!pixels.current) return;
    const p = at(e); const c = current();
    if (tool === 'pick') { const o = (p[1] * size[0] + p[0]) * 4; const px = pixels.current; setColor('#' + [px[o], px[o + 1], px[o + 2]].map(v => v.toString(16).padStart(2, '0')).join('')); return; }
    remember();
    if (tool === 'fill') { fill(p[0], p[1], c); draw(); return; }
    if (tool === 'pencil' || tool === 'eraser' || tool === 'spray') { setStart(p); if (tool === 'spray') spray(p, c); else put(p[0], p[1], c); draw(); return; }
    backup.current = new Uint8ClampedArray(pixels.current); setStart(p);
  }
  function onMove(e: React.MouseEvent) {
    const s0 = start.current;
    if (!s0 || !pixels.current) return;
    const p = at(e); const c = current();
    if (tool === 'pencil' || tool === 'eraser') { line(s0, p, c); setStart(p); }
    else if (tool === 'spray') spray(p, c);
    else if (backup.current) { pixels.current.set(backup.current); shape(s0, p, c, tool); }
    draw();
  }
  function onUp() { setStart(null); backup.current = null; }
  function undo() { const prev = history.current.pop(); if (prev && pixels.current) { pixels.current.set(prev); draw(); } }

  async function save() {
    const [w, h] = size; const px = pixels.current!;
    const parts: string[] = [];
    for (let i = 0; i < w * h; i++) parts.push(px[i * 4].toString(16).padStart(2, '0') + px[i * 4 + 1].toString(16).padStart(2, '0') + px[i * 4 + 2].toString(16).padStart(2, '0'));
    const r = await fetch(`/api/picture/${encodeURIComponent(klass)}?kind=${kind}`, { method: 'POST', body: JSON.stringify({ op: 'dibset', handle, w, h, rgb: parts.join('') }) });
    if (!r.ok) { setError((await r.json()).error ?? 'ошибка'); return; }
    onSaved(); onClose();
  }

  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div className="modal bitmap-editor" style={{ width: 'min(860px, 96vw)' }} onMouseDown={e => e.stopPropagation()}>
        <div className="panel-title">Битовый редактор <span className="muted">{size[0]}×{size[1]}</span><span className="spacer" /><button className="small ghost" onClick={onClose}>×</button></div>
        <div className="draw-tools">
          {TOOLS.map(([id, icon, hint]) => <button key={id} className={`small icon-only${tool === id ? ' active' : ''}`} title={hint} onClick={() => setTool(id)}><Icon name={icon} /></button>)}
          <span className="sep" />
          <input type="color" value={color} onChange={e => setColor(e.target.value)} title="Цвет" />
          <span className="sep" />
          <button className="small icon-only" onClick={undo} title="Отменить"><Icon name="undo" /></button>
          <button className={`small icon-only${grid ? ' active' : ''}`} onClick={() => setGrid(g => !g)} title="Сетка"><Icon name="grid" /></button>
          <button className="small ghost mono" onClick={() => setZoom(z => Math.max(1, z - 1))}>−</button>
          <span className="mono small">{zoom}×</span>
          <button className="small ghost mono" onClick={() => setZoom(z => Math.min(32, z + 1))}>+</button>
        </div>
        <div className="modal-body bitmap-body">
          {error ? <div className="muted">{error}</div> : <canvas ref={canvas} onMouseDown={onDown} onMouseMove={onMove} onMouseUp={onUp} onMouseLeave={onUp} onContextMenu={e => e.preventDefault()} />}
        </div>
        <div className="modal-actions">
          <button className="ghost" onClick={onClose}>Отмена</button>
          <button className="primary" onClick={save} disabled={!size[0]}>Сохранить в рисунок</button>
        </div>
      </div>
    </div>
  );
}
