// Окно модели: живые окна из ядра; мышь и клавиатура уходят в модель.
import { useEffect, useRef } from 'react';
import { api } from '../api';
import { useStore } from '../store';

export function ModelView() {
  const frame = useStore(s => s.frame);
  const last = useRef<Map<number, string>>(new Map());
  const host = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const root = host.current;
    if (!root || !frame) return;
    for (const w of frame.windows) {
      let div = root.querySelector<HTMLDivElement>(`[data-win="${w.id}"]`);
      if (!div) {
        div = document.createElement('div');
        div.className = 'win'; div.dataset.win = String(w.id);
        div.innerHTML = `<div class="title"></div><div class="body"></div>`;
        attach(div, w.name);
        root.appendChild(div);
      }
      div.querySelector('.title')!.textContent = w.name;
      if (last.current.get(w.id) !== w.svg) { div.querySelector('.body')!.innerHTML = w.svg; last.current.set(w.id, w.svg); }
    }
    for (const div of [...root.children]) if (!frame.windows.some(w => String(w.id) === (div as HTMLElement).dataset.win)) div.remove();
  }, [frame]);

  if (!frame?.windows.length) return <div className="model"><div className="muted" style={{ color: '#eee' }}>Модель не открыла окон — нажмите Пуск или Шаг.</div></div>;
  return <div className="model" ref={host} tabIndex={0}
    onKeyDown={e => { e.preventDefault(); api.event(`type=key&msg=256&vk=${e.keyCode}`); }}
    onKeyUp={e => api.event(`type=key&msg=257&vk=${e.keyCode}`)} />;
}

function attach(div: HTMLDivElement, name: string) {
  const body = div.querySelector<HTMLDivElement>('.body')!;
  const send = (msg: number, e: MouseEvent) => {
    const svg = body.querySelector('svg');
    if (!svg) return;
    const r = svg.getBoundingClientRect();
    const vb = svg.viewBox.baseVal;
    const x = (e.clientX - r.left) * (vb.width / r.width), y = (e.clientY - r.top) * (vb.height / r.height);
    const keys = (e.buttons & 1 ? 1 : 0) | (e.buttons & 2 ? 2 : 0) | (e.buttons & 4 ? 16 : 0);
    api.event(`type=mouse&win=${encodeURIComponent(name)}&msg=${msg}&x=${x.toFixed(2)}&y=${y.toFixed(2)}&keys=${keys}`);
  };
  body.onmousemove = e => send(512, e);
  body.onmousedown = e => { (div.closest('.model') as HTMLElement)?.focus(); send(e.button === 0 ? 513 : e.button === 2 ? 516 : 519, e); };
  body.onmouseup = e => send(e.button === 0 ? 514 : e.button === 2 ? 517 : 520, e);
  body.ondblclick = e => send(515, e);
  body.oncontextmenu = e => e.preventDefault();
}
