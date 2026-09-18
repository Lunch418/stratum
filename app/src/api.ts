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
export interface Project { root: string; dir: string; native: boolean; unsaved: boolean; classes: ClassInfo[] }
export interface Instance { index: number; path: string; name: string; class: string; parent: number | null; handle: number }
export interface Frame {
  tick: number; running: boolean; stopped: boolean;
  windows: { id: number; name: string; w: number; h: number; svg: string }[];
  log: string[];
}
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
    return r.json() as Promise<{ ok: boolean; error?: ParseError }>;
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
  setValue: (index: number, name: string, value: string) =>
    fetch(`/api/set/${index}?var=${encodeURIComponent(name)}&value=${encodeURIComponent(value)}`, { method: 'POST' }),
};
