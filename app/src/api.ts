// Обёртка над JSON-API ядра (core/src/player/api.rs).

export interface Variable {
  name: string; type: string; default: string; description: string; local: boolean; flags: number;
}
export interface Child { handle: number; class: string; name: string; x: number; y: number }
export interface Link { handle: number; source: number; target: number; vars: [string, string][] }
export interface ClassInfo {
  name: string; library: boolean; description: string; vars: Variable[];
  declared: { name: string; type: string }[]; text: string; children: Child[]; links: Link[];
  hasIcon: boolean; hasScheme: boolean; hasImage: boolean; source: string;
}
export interface Project { root: string; dir: string; empty: boolean; native: boolean; unsaved: boolean; canUndo: boolean; canRedo: boolean; classes: ClassInfo[] }
export interface Instance { index: number; path: string; name: string; class: string; parent: number | null; handle: number }
export interface Halt { kind: 'error' | 'breakpoint'; message: string; instance: number | null; path: string; class: string; line: number }
export interface Breakpoint { id: number; index: number | null; path: string; class: string; expr: string; enabled: boolean }
export interface Frame {
  tick: number; running: boolean; stopped: boolean; canBack: boolean; halt: Halt | null;
  windows: { id: number; name: string; w: number; h: number; svg: string; controls: Control[] }[];
  sounds: { cmd: 'play' | 'stop'; file: string; loop: boolean }[];
  log: string[];
}
export interface Trace { id: number; index: number; path: string; var: string; points: [number, number][] }
export interface Control { handle: number; class: string; text: string; style: number; checked: boolean; enabled: boolean; x: number; y: number; w: number; h: number }
export interface ParseError { line: number; column: number; message: string }

async function get<T>(url: string): Promise<T> {
  const r = await fetch(url);
  if (!r.ok) throw new Error(`${url}: ${r.status}`);
  return r.json();
}

export const api = {
  project: () => get<Project>('/api/project'),
  klass: (name: string) => get<ClassInfo>(`/api/class/${encodeURIComponent(name)}`),
  scheme: (name: string) => get<{ bounds: { x: number; y: number; w: number; h: number } | null; svg: string }>(`/api/scheme/${encodeURIComponent(name)}`),
  iconUrl: (name: string) => `/api/icon/${encodeURIComponent(name)}`,
  instances: () => get<Instance[]>('/api/instances'),
  watch: (index: number) => get<{ path: string; vars: [string, string][] }>(`/api/watch/${index}`),
  frame: () => get<Frame>('/frame'),
  event: (q: string) => fetch('/event?' + q, { method: 'POST' }),
  setText: async (name: string, text: string) => {
    const r = await fetch(`/api/class/${encodeURIComponent(name)}/text`, { method: 'POST', body: text });
    return r.json() as Promise<{ ok: boolean; live?: boolean; error?: ParseError }>;
  },
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
  setLink: async (klass: string, handle: number, source: number, target: number, pairs: [string, string][]) => {
    const r = await fetch(`/api/link/set?class=${encodeURIComponent(klass)}&handle=${handle}&source=${source}&target=${target}`, { method: 'POST', body: pairs.map(p => p.join('\t')).join('\n') });
    return r.json() as Promise<{ ok: boolean; handle: number }>;
  },
  removeLink: (klass: string, handle: number) =>
    fetch(`/api/link/remove?class=${encodeURIComponent(klass)}&handle=${handle}`, { method: 'POST' }),
  newClass: async (name: string) => {
    const r = await fetch(`/api/class/new?name=${encodeURIComponent(name)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
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
  eval: (index: number, expr: string) => get<{ ok: boolean; value?: string; error?: string }>(`/api/eval/${index}?expr=${encodeURIComponent(expr)}`),
  profile: () => get<{ tick: number; total: number; items: { index: number; path: string; class: string; ns: number }[] }>('/api/profile'),
  helpSearch: (q: string) => get<string[]>(`/api/help?q=${encodeURIComponent(q)}`),
  browse: (dir: string) => get<{ dir: string; parent: string | null; entries: { name: string; path: string; kind: 'dir' | 'project' | 'file' }[] }>(`/api/browse?dir=${encodeURIComponent(dir)}`),
  open: async (path: string) => {
    const r = await fetch(`/api/open?path=${encodeURIComponent(path)}`, { method: 'POST' });
    const j = await r.json();
    if (!r.ok) throw new Error(j.error ?? r.statusText);
    return j as { ok: boolean; dir: string };
  },
  newProject: async () => { await fetch('/api/new', { method: 'POST' }); },
  undo: () => fetch('/api/undo', { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; canUndo: boolean; canRedo: boolean }>),
  redo: () => fetch('/api/redo', { method: 'POST' }).then(r => r.json() as Promise<{ ok: boolean; canUndo: boolean; canRedo: boolean }>),
  setValue: (index: number, name: string, value: string) =>
    fetch(`/api/set/${index}?var=${encodeURIComponent(name)}&value=${encodeURIComponent(value)}`, { method: 'POST' }),
};
