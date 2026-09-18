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
  bottomTab: 'messages' | 'graphs';
  setBottomTab: (t: 'messages' | 'graphs') => void;
  paletteOpen: boolean;
  setPaletteOpen: (v: boolean) => void;
  traceCount: number;
  setTraceCount: (n: number) => void;
  unsaved: boolean;
  markUnsaved: () => void;
  /// перечитать проект с сервера, сохранив выбор и путь по схемам
  reload: () => Promise<void>;
  undo: () => Promise<void>;
  redo: () => Promise<void>;
  saveProject: (dir?: string) => Promise<void>;
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
    set({ project, schemePath: [project.root], selectedClass: project.root, unsaved: project.unsaved });
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
  unsaved: false,
  bottomTab: 'messages',
  setBottomTab: bottomTab => set({ bottomTab }),
  paletteOpen: false,
  setPaletteOpen: paletteOpen => set({ paletteOpen }),
  traceCount: 0,
  setTraceCount: traceCount => set({ traceCount }),
  markUnsaved: () => set(s => ({ unsaved: true, project: s.project ? { ...s.project, canUndo: true, canRedo: false } : s.project })),
  reload: async () => {
    const project = await api.project();
    set(s => {
      const names = new Set(project.classes.map(c => c.name.toLowerCase()));
      const schemePath = s.schemePath.filter(p => names.has(p.toLowerCase()));
      const selectedClass = s.selectedClass && names.has(s.selectedClass.toLowerCase()) ? s.selectedClass : project.root;
      return { project, unsaved: project.unsaved, schemePath: schemePath.length ? schemePath : [project.root], selectedClass };
    });
  },
  undo: async () => { const r = await api.undo(); if (r.ok) { await get().reload(); get().showToast('Отменено'); } },
  redo: async () => { const r = await api.redo(); if (r.ok) { await get().reload(); get().showToast('Повторено'); } },
  saveProject: async dir => {
    const r = await api.save(dir);
    set(s => ({ unsaved: false, project: s.project ? { ...s.project, dir: r.dir, native: true } : s.project }));
    get().showToast('Проект сохранён');
    get().say({ level: 'info', where: 'проект', text: 'сохранено в ' + r.dir });
  },
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
