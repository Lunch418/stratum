import { create } from 'zustand';
import { api, type ClassInfo, type Frame, type Instance, type Project } from './api';

export type Tab = 'scheme' | 'code' | 'model';

interface Message { level: 'info' | 'error'; where: string; text: string }

interface State {
  project: Project | null;
  instances: Instance[];
  frame: Frame | null;
  selectedClass: string | null;
  selectedInstance: number | null;
  /// Путь по схемам: имена классов от корня.
  schemePath: string[];
  tab: Tab;
  messages: Message[];
  toast: string | null;
  theme: 'light' | 'dark';
  load: () => Promise<void>;
  refreshInstances: () => Promise<void>;
  select: (klass: string | null, instance?: number | null) => void;
  enterScheme: (klass: string) => void;
  goToScheme: (depth: number) => void;
  setTab: (t: Tab) => void;
  setFrame: (f: Frame) => void;
  updateClass: (c: ClassInfo) => void;
  say: (m: Message) => void;
  showToast: (t: string) => void;
  toggleTheme: () => void;
}

export const useStore = create<State>((set, get) => ({
  project: null,
  instances: [],
  frame: null,
  selectedClass: null,
  selectedInstance: null,
  schemePath: [],
  tab: 'scheme',
  messages: [],
  toast: null,
  theme: (localStorage.getItem('theme') as 'light' | 'dark') || 'light',
  load: async () => {
    const project = await api.project();
    set({ project, schemePath: [project.root], selectedClass: project.root });
    await get().refreshInstances();
  },
  refreshInstances: async () => set({ instances: await api.instances() }),
  select: (klass, instance = null) => set({ selectedClass: klass, selectedInstance: instance }),
  enterScheme: klass => set(s => ({ schemePath: [...s.schemePath, klass], selectedClass: klass, tab: 'scheme' })),
  goToScheme: depth => set(s => ({ schemePath: s.schemePath.slice(0, depth + 1), selectedClass: s.schemePath[depth] })),
  setTab: tab => set({ tab }),
  setFrame: frame => set({ frame }),
  updateClass: c => set(s => s.project ? { project: { ...s.project, classes: s.project.classes.map(k => k.name === c.name ? c : k) } } : {}),
  say: m => set(s => ({ messages: [...s.messages.slice(-199), m] })),
  showToast: t => { set({ toast: t }); setTimeout(() => set({ toast: null }), 1800); },
  toggleTheme: () => set(s => {
    const theme = s.theme === 'light' ? 'dark' : 'light';
    localStorage.setItem('theme', theme);
    return { theme };
  }),
}));

export function classByName(project: Project | null, name: string | null | undefined): ClassInfo | undefined {
  if (!project || !name) return undefined;
  const lower = name.toLowerCase();
  return project.classes.find(c => c.name.toLowerCase() === lower);
}
