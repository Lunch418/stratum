// Окно модели: живые окна из ядра; мышь и клавиатура уходят в модель.
import { useEffect, useRef } from 'react';
import { api, type Control } from '../api';
import { useStore } from '../store';

// звук модели: SndPlaySound и MCI приходят в кадре как команды
const playing = new Map<string, HTMLAudioElement>();
export function handleSounds(sounds: { cmd: string; file: string; loop: boolean }[] | undefined) {
  for (const snd of sounds ?? []) {
    const key = snd.file.toLowerCase();
    if (snd.cmd === 'stop') {
      if (!snd.file) { for (const a of playing.values()) a.pause(); playing.clear(); }
      else { playing.get(key)?.pause(); playing.delete(key); }
      continue;
    }
    playing.get(key)?.pause();
    const a = new Audio('/api/file?name=' + encodeURIComponent(snd.file));
    a.loop = snd.loop;
    a.play().catch(() => { /* MIDI и запрет автозапуска — молча */ });
    playing.set(key, a);
  }
}

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
        div.innerHTML = `<div class="title"></div><div class="body"><div class="controls"></div></div>`;
        attach(div, w.name);
        root.appendChild(div);
      }
      div.querySelector('.title')!.textContent = w.name;
      const body = div.querySelector<HTMLDivElement>('.body')!;
      if (last.current.get(w.id) !== w.svg) {
        body.querySelector('svg')?.remove();
        body.insertAdjacentHTML('afterbegin', w.svg);
        last.current.set(w.id, w.svg);
      }
      syncControls(body.querySelector<HTMLDivElement>('.controls')!, w.name, w.controls ?? []);
    }
    for (const div of [...root.children]) if (!frame.windows.some(w => String(w.id) === (div as HTMLElement).dataset.win)) div.remove();
  }, [frame]);

  if (!frame?.windows.length) return <div className="model"><div className="muted" style={{ color: '#eee' }}>Модель не открыла окон — нажмите Пуск или Шаг.</div></div>;
  return <div className="model" ref={host} tabIndex={0} title="Alt+щелчок — свойства объекта в инспекторе"
    onKeyDown={e => { e.preventDefault(); api.event(`type=key&msg=256&vk=${e.keyCode}`); }}
    onKeyUp={e => api.event(`type=key&msg=257&vk=${e.keyCode}`)} />;
}

// Настоящие кнопки, флажки, поля ввода и списки поверх SVG: события уходят
// в модель как WM_CONTROLNOTIFY (0 — нажатие, 768 — правка текста, 1 — выбор)
function syncControls(layer: HTMLDivElement, win: string, controls: Control[]) {
  const seen = new Set<string>();
  for (const c of controls) {
    const key = String(c.handle);
    seen.add(key);
    let el = layer.querySelector<HTMLElement>(`[data-ctl="${key}"]`);
    const kind = controlKind(c);
    if (!el || el.dataset.kind !== kind) {
      el?.remove();
      el = createControl(kind, win, c);
      el.dataset.ctl = key; el.dataset.kind = kind;
      layer.appendChild(el);
    }
    el.style.left = `${c.x}px`; el.style.top = `${c.y}px`; el.style.width = `${c.w}px`; el.style.height = `${c.h}px`;
    updateControl(el, kind, c);
  }
  for (const el of [...layer.children] as HTMLElement[]) if (!seen.has(el.dataset.ctl ?? '')) el.remove();
}

function controlKind(c: Control): string {
  const cls = c.class.toUpperCase();
  if (cls === 'BUTTON') {
    const type = c.style & 0xf;
    if (type === 2 || type === 3 || type === 5 || type === 6) return 'checkbox'; // BS_CHECKBOX/AUTOCHECKBOX/3STATE
    if (type === 4 || type === 9) return 'radio';
    return 'button';
  }
  if (cls === 'EDIT') return (c.style & 0x4) ? 'textarea' : 'edit'; // ES_MULTILINE
  if (cls === 'LISTBOX' || cls === 'COMBOBOX') return 'list';
  return 'button';
}

function createControl(kind: string, win: string, c: Control): HTMLElement {
  const send = (code: number, extra = '') => api.event(`type=control&win=${encodeURIComponent(win)}&handle=${c.handle}&code=${code}${extra}`);
  let el: HTMLElement;
  if (kind === 'button') {
    el = document.createElement('button');
    el.onclick = () => send(0);
  } else if (kind === 'checkbox' || kind === 'radio') {
    el = document.createElement('label');
    const input = document.createElement('input');
    input.type = kind; input.name = kind === 'radio' ? `radio-${win}` : '';
    input.onchange = () => send(0, `&checked=${input.checked ? 1 : 0}`);
    el.appendChild(input); el.appendChild(document.createElement('span'));
  } else if (kind === 'edit' || kind === 'textarea') {
    el = document.createElement(kind === 'edit' ? 'input' : 'textarea');
    if (kind === 'edit') (el as HTMLInputElement).type = 'text';
    el.oninput = () => send(768, `&text=${encodeURIComponent((el as HTMLInputElement).value)}`);
  } else {
    el = document.createElement('select');
    (el as HTMLSelectElement).size = 4;
    el.onchange = () => send(1, `&text=${encodeURIComponent((el as HTMLSelectElement).value)}`);
  }
  el.className = 'ctl';
  el.onmousedown = e => e.stopPropagation();
  el.onmouseup = e => e.stopPropagation();
  el.onkeydown = e => e.stopPropagation();
  return el;
}

function updateControl(el: HTMLElement, kind: string, c: Control) {
  const disabled = !c.enabled;
  if (kind === 'button') { el.textContent = c.text; (el as HTMLButtonElement).disabled = disabled; }
  else if (kind === 'checkbox' || kind === 'radio') {
    const input = el.querySelector('input')!;
    if (document.activeElement !== input) input.checked = c.checked;
    input.disabled = disabled;
    el.querySelector('span')!.textContent = c.text;
  } else if (kind === 'edit' || kind === 'textarea') {
    const input = el as HTMLInputElement;
    if (document.activeElement !== input && input.value !== c.text) input.value = c.text;
    input.disabled = disabled;
  } else {
    const sel = el as HTMLSelectElement;
    const items = c.text.split('\n').filter(Boolean);
    if ([...sel.options].map(o => o.value).join('\n') !== items.join('\n')) {
      sel.innerHTML = '';
      for (const it of items) { const o = document.createElement('option'); o.value = it; o.textContent = it; sel.appendChild(o); }
    }
    sel.disabled = disabled;
  }
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
  body.onmousedown = e => {
    (div.closest('.model') as HTMLElement)?.focus();
    // Alt+щелчок — выбрать объект для инспектора, не отдавая событие модели
    if (e.altKey && e.button === 0) {
      e.preventDefault();
      const svg = body.querySelector('svg');
      if (!svg) return;
      const r = svg.getBoundingClientRect();
      const vb = svg.viewBox.baseVal;
      // координаты страницы → координаты пространства (как в svg::render)
      const px = (e.clientX - r.left) * (vb.width / r.width), py = (e.clientY - r.top) * (vb.height / r.height);
      const g = svg.querySelector('g[transform]');
      const m = g?.getAttribute('transform')?.match(/translate\(([-\d.e]+) ([-\d.e]+)\) scale\(([-\d.e]+)\)/);
      const [ox, oy, k] = m ? [Number(m[1]), Number(m[2]), Number(m[3])] : [0, 0, 1];
      const x = (px - ox) / k, y = (py - oy) / k;
      api.objectAt(name, x, y).then(o => useStore.getState().pickObject(o ? { win: name, handle: o.handle } : null));
      return;
    }
    send(e.button === 0 ? 513 : e.button === 2 ? 516 : 519, e);
  };
  body.onmouseup = e => send(e.button === 0 ? 514 : e.button === 2 ? 517 : 520, e);
  body.ondblclick = e => send(515, e);
  body.oncontextmenu = e => e.preventDefault();
}
