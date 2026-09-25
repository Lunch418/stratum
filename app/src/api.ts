// Обёртка над JSON-API ядра (core/src/player/api.rs).

export interface Variable {
  name: string; type: string; default: string; description: string; local: boolean; flags: number;
}
export interface Child { handle: number; class: string; name: string; x: number; y: number }
export interface LinkStyle { color: string; width: number; disabled: boolean; arrows: boolean; layer: number }
/// pad — контактная площадка со стороны самого имиджа (handle 0); 0 — не указана
export interface Link { handle: number; source: number; target: number; vars: [string, string][]; style: LinkStyle; pad: number }
/// Контактная площадка схемы: через неё связи идут к переменным самого имиджа.
export interface Pad { id: number; x: number; y: number }
/// Параметры листа (диалог «Параметры листа»): сетка, окно модели, слои.
export interface Sheet {
  gridOrigin: [number, number]; gridStep: [number, number]; gridVisible: boolean; gridSnap: boolean;
  windowStyle: 'mdi' | 'dialog' | 'popup' | 'default'; windowSize: 'max' | 'min' | 'default' | 'space' | 'fixed'; windowWh: [number, number];
  windowFixed: boolean; hscroll: boolean; vscroll: boolean; autoOrigin: boolean; layers: number; noSubwindows: boolean;
}
export interface ClassInfo {
  name: string; library: boolean; description: string; vars: Variable[];
  declared: { name: string; type: string }[]; text: string; children: Child[]; links: Link[]; pads: Pad[];
  hasIcon: boolean; hasScheme: boolean; hasImage: boolean; source: string; flags: number; sheet: Sheet;
}
export interface ProjectProperty { key: string; int?: number; text?: string }
export interface Project { root: string; dir: string; empty: boolean; native: boolean; unsaved: boolean; canUndo: boolean; canRedo: boolean; classes: ClassInfo[] }
export interface Instance { index: number; path: string; name: string; class: string; parent: number | null; handle: number }
export interface Halt { kind: 'error' | 'breakpoint' | 'warning' | 'math'; message: string; instance: number | null; path: string; class: string; line: number }
export interface Breakpoint { id: number; index: number | null; path: string; class: string; expr: string; enabled: boolean }
export interface ModelDialog { kind: 'message' | 'input' | 'open' | 'save' | 'folder' | 'color'; title: string; text: string; style: number; default: string }
export interface Frame {
  tick: number; running: boolean; stopped: boolean; canBack: boolean; canHyperBack?: boolean; halt: Halt | null; dialog: ModelDialog | null;
  /// вызовы, которые ядро не выполняет (базы данных, Ogre3D, анализатор текста…)
  unsupported?: { name: string; count: number }[];
  windows: { id: number; name: string; w: number; h: number; size?: string; style?: string; svg: string; controls: Control[] }[];
  sounds: { cmd: 'play' | 'stop'; file: string; loop: boolean }[];
  /// гиперпереходы кадра: смена страницы с эффектом (0), запуск приложения (1), загрузка проекта (2)
  hyper?: { mode: number; target: string; window: string; effect: string }[];
  log: string[];
}
export interface Trace { id: number; index: number; path: string; var: string; points: [number, number][] }
export interface Control { handle: number; class: string; text: string; style: number; checked: boolean; enabled: boolean; x: number; y: number; w: number; h: number }
export interface ObjectProps {
  handle: number; name: string; kind: string; x: number; y: number; w: number; h: number; angle: number; visible: boolean; alpha: number;
  zorder: number | null; parent: number | null; pen?: { color: string; width: number; style: number }; brush?: { color: string; style: number };
  points?: [number, number][]; text?: string; class?: string; children?: number;
  font?: { face: string; size: number; bold: boolean; italic: boolean; underline: boolean; fg: string; bg: string }; enabled?: boolean; checked?: boolean;
  /// гиперссылка (закладка «Гипербаза»): режим 0 окно, 1 приложение, 2 проект, 3 ничего, 4 команда
  hyper?: { mode: number; target: string; window: string; object: string; effect: string };
  /// члены группы и исходный прямоугольник растра
  members?: number[]; src?: [number, number, number, number];
}
export interface ParseError { line: number; column: number; message: string }

async function get<T>(url: string): Promise<T> {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url}: ${r.status}`);
  return r.json();
}

// версия иконок: меняется после выбора иконки, чтобы сбросить кэш браузера
let iconVersion = 0;

export const api = {
  project: () => get<Project>('/api/project'),
  klass: (name: string) => get<ClassInfo>(`/api/class/${encodeURIComponent(name)}`),
  scheme: (name: string) => get<{ bounds: { x: number; y: number; w: number; h: number } | null; svg: string }>(`/api/scheme/${encodeURIComponent(name)}`),
  iconUrl: (name: string) => `/api/icon/${encodeURIComponent(name)}${iconVersion ? '?v=' + iconVersion : ''}`,
  bumpIcons: () => { iconVersion++; },
  iconSets: () => get<{ file: string; count: number; cols: number }[]>('/api/icons'),
  setClassIcon: (klass: string, file: string, index: number) => fetch(`/api/class/${encodeURIComponent(klass)}/icon?file=${encodeURIComponent(file)}&index=${index}`, { method: 'POST' }),
  instances: () => get<Instance[]>('/api/instances'),
  watch: (index: number) => get<{ path: string; vars: [string, string][] }>(`/api/watch/${index}`),
  frame: () => get<Frame>('/frame'),
  event: (q: string) => fetch('/event?' + q, { method: 'POST' }),
  setText: async (name: string, text: string) => {
    const r = await fetch(`/api/class/${encodeURIComponent(name)}/text`, { method: 'POST', body: text });
    return r.json() as Promise<{ ok: boolean; live?: boolean; error?: ParseError; compileError?: { line: number; message: string } | null }>;
  },
  /// имиджи, которые компилятор в правилах Stratum 2000 не принимает
  check: () => get<{ class: string; line: number; message: string; stored: boolean }[]>('/api/check'),
  setVars: async (name: string, vars: Variable[]) => {
    const body = vars.map(v => [v.name, v.type, v.default, v.description, v.flags].join('\t')).join('\n');
    const r = await fetch(`/api/class/${encodeURIComponent(name)}/vars`, { method: 'POST', body });
    return r.json();
  },
  moveChild: (klass: string, handle: number, x: number, y: number) =>
    fetch(`/api/child/move?class=${encodeURIComponent(klass)}&handle=${handle}&x=${x}&y=${y}`, { method: 'POST' }),
  // сохранить проект в родном формате (project.json); dir — новая папка
  save: async (dir?: string) => {
    const r = await fetch('/api/save' + (dir ? '?dir=' + encodeURIComponent(dir) : ''), { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; dir: string };
  },
  exportProject: async (dir: string) => {
    const r = await fetch('/api/export?dir=' + encodeURIComponent(dir), { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; dir: string; classes: number };
  },
  addChild: async (klass: string, child: string, x: number, y: number, name = '') => {
    const r = await fetch(`/api/child/add?class=${encodeURIComponent(klass)}&child=${encodeURIComponent(child)}&x=${x}&y=${y}&name=${encodeURIComponent(name)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; handle: number };
  },
  removeChild: (klass: string, handle: number) =>
    fetch(`/api/child/remove?class=${encodeURIComponent(klass)}&handle=${handle}`, { method: 'POST' }),
  renameChild: (klass: string, handle: number, name: string) =>
    fetch(`/api/child/rename?class=${encodeURIComponent(klass)}&handle=${handle}&name=${encodeURIComponent(name)}`, { method: 'POST' }),
  // handle 0 — новая связь; пустой список пар удаляет связь
  setLink: async (klass: string, handle: number, source: number, target: number, pairs: [string, string][], pad = 0) => {
    const r = await fetch(`/api/link/set?class=${encodeURIComponent(klass)}&handle=${handle}&source=${source}&target=${target}${pad ? '&pad=' + pad : ''}`, { method: 'POST', body: pairs.map(p => p.join('\t')).join('\n') });
    return r.json() as Promise<{ ok: boolean; handle: number }>;
  },
  padAdd: (klass: string, x: number, y: number) =>
    fetch(`/api/pad/add?class=${encodeURIComponent(klass)}&x=${x}&y=${y}`, { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; id: number }>),
  padMove: (klass: string, id: number, x: number, y: number) =>
    fetch(`/api/pad/move?class=${encodeURIComponent(klass)}&id=${id}&x=${x}&y=${y}`, { method: 'POST' }),
  padRemove: (klass: string, id: number) =>
    fetch(`/api/pad/remove?class=${encodeURIComponent(klass)}&id=${id}`, { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; links: number }>),
  removeLink: (klass: string, handle: number) =>
    fetch(`/api/link/remove?class=${encodeURIComponent(klass)}&handle=${handle}`, { method: 'POST' }),
  newClass: async (name: string) => {
    const r = await fetch(`/api/class/new?name=${encodeURIComponent(name)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    // «Параметры среды → 2D-редактор → Шаблоны»: рисунок и схема нового имиджа из .vdr
    let tpl: { image?: string; scheme?: string } = {};
    try { tpl = (JSON.parse(localStorage.getItem('env') ?? '{}') as { templates?: typeof tpl }).templates ?? {}; } catch { /* нет настроек */ }
    for (const kind of ['image', 'scheme'] as const) {
      const file = tpl[kind]?.trim();
      if (file) await fetch(`/api/picture/${encodeURIComponent(name)}?kind=${kind}`, { method: 'POST', body: JSON.stringify({ op: 'insert', file, x: 0, y: 0 }) });
    }
  },
  traces: (since = 0) => get<{ tick: number; traces: Trace[] }>(`/api/traces?since=${since}`),
  traceAdd: (index: number, name: string) =>
    fetch(`/api/trace/add?index=${index}&var=${encodeURIComponent(name)}`, { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; id: number }>),
  traceRemove: (id: number) => fetch(`/api/trace/remove?id=${id}`, { method: 'POST' }),
  renameClass: async (name: string, to: string) => {
    const r = await fetch(`/api/class/${encodeURIComponent(name)}/rename?to=${encodeURIComponent(to)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
  },
  deleteClass: async (name: string) => {
    const r = await fetch(`/api/class/${encodeURIComponent(name)}/delete`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
  },
  breakpoints: () => get<Breakpoint[]>('/api/breakpoints'),
  breakpointAdd: (target: { index: number } | { class: string }, expr: string) =>
    fetch(`/api/breakpoint/add?${'index' in target ? 'index=' + target.index : 'class=' + encodeURIComponent(target.class)}&expr=${encodeURIComponent(expr)}`, { method: 'POST' }).then(r => r.json()),
  breakpointRemove: (id: number) => fetch(`/api/breakpoint/remove?id=${id}`, { method: 'POST' }),
  breakpointToggle: (id: number) => fetch(`/api/breakpoint/toggle?id=${id}`, { method: 'POST' }),
  eval: (index: number, expr: string) => fetch(`/api/eval/${index}?expr=${encodeURIComponent(expr)}`, { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; value?: string; error?: string }>),
  profile: () => get<{ tick: number; total: number; items: { index: number; path: string; class: string; ns: number }[] }>('/api/profile'),
  helpSearch: (q: string) => get<string[]>(`/api/help?q=${encodeURIComponent(q)}`),
  browse: (dir: string, ext = '') => get<{ dir: string; parent: string | null; entries: { name: string; path: string; kind: 'dir' | 'project' | 'file' }[] }>(`/api/browse?dir=${encodeURIComponent(dir)}${ext ? '&ext=' + ext : ''}`),
  setLinkStyle: (klass: string, handle: number, style: LinkStyle) =>
    fetch(`/api/link/style?class=${encodeURIComponent(klass)}&handle=${handle}`, { method: 'POST', body: JSON.stringify(style) }),
  setSheet: (klass: string, sheet: Sheet) =>
    fetch(`/api/class/${encodeURIComponent(klass)}/sheet`, { method: 'POST', body: JSON.stringify(sheet) }),
  setClassProps: (klass: string, props: { description?: string; flags?: number }) => {
    const q = new URLSearchParams();
    if (props.description !== undefined) q.set('description', props.description);
    if (props.flags !== undefined) q.set('flags', String(props.flags));
    return fetch(`/api/class/${encodeURIComponent(klass)}/props?${q}`, { method: 'POST' });
  },
  replaceChild: (klass: string, handle: number, child: string) =>
    fetch(`/api/child/replace?class=${encodeURIComponent(klass)}&handle=${handle}&child=${encodeURIComponent(child)}`, { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; droppedPairs: number }>),
  moveChildren: (klass: string, items: { handle: number; x: number; y: number }[]) =>
    fetch(`/api/child/moveall?class=${encodeURIComponent(klass)}`, { method: 'POST', body: items.map(i => `${i.handle}\t${i.x}\t${i.y}`).join('\n') }),
  mergeChildren: (klass: string, name: string, handles: number[]) =>
    fetch(`/api/child/merge?class=${encodeURIComponent(klass)}&name=${encodeURIComponent(name)}`, { method: 'POST', body: handles.join('\n') }).then(async r => { const j = await r.json(); if (!r.ok) throw new Error(j.error); return j as { ok: boolean; handle: number }; }),
  reorderChildren: (klass: string, handles: number[]) =>
    fetch(`/api/child/reorder?class=${encodeURIComponent(klass)}`, { method: 'POST', body: handles.join('\n') }),
  projectProperties: () => get<ProjectProperty[]>('/api/project/properties'),
  setProjectProperty: (key: string, value: number | string | null) => {
    const q = new URLSearchParams({ key });
    if (typeof value === 'number') q.set('int', String(value));
    else if (typeof value === 'string') q.set('text', value);
    return fetch(`/api/project/properties?${q}`, { method: 'POST' });
  },
  open: async (path: string) => {
    const r = await fetch(`/api/open?path=${encodeURIComponent(path)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; dir: string };
  },
  newProject: async () => { await fetch('/api/new', { method: 'POST' }); },
  objectAt: (win: string, x: number, y: number) => get<ObjectProps | null>(`/api/object?win=${encodeURIComponent(win)}&x=${x}&y=${y}`),
  object: (win: string, handle: number) => get<ObjectProps | null>(`/api/object?win=${encodeURIComponent(win)}&handle=${handle}`),
  objectSet: (win: string, handle: number, field: string, value: string | number) =>
    fetch(`/api/object/set?win=${encodeURIComponent(win)}&handle=${handle}&field=${field}&value=${encodeURIComponent(String(value))}`, { method: 'POST' }),
  stateAction: async (action: 'save' | 'load' | 'keep' | 'default', path = '', klass = '') => {
    const r = await fetch(`/api/state/${action}?path=${encodeURIComponent(path)}${klass ? '&class=' + encodeURIComponent(klass) : ''}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; images?: number };
  },
  undo: () => fetch('/api/undo', { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; canUndo: boolean; canRedo: boolean }>),
  redo: () => fetch('/api/redo', { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; canUndo: boolean; canRedo: boolean }>),
  setValue: (index: number, name: string, value: string) =>
    fetch(`/api/set/${index}?var=${encodeURIComponent(name)}&value=${encodeURIComponent(value)}`, { method: 'POST' }),
};
