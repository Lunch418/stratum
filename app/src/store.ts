import { create } from 'zustand';
import { api, type ClassInfo, type Frame, type Instance, type Project } from './api';

export type Tab = 'scheme' | 'code' | 'model' | 'graph' | 'picture' | 'icon';
export type DialogId = 'saveAs' | 'export' | 'open' | 'stateSave' | 'stateLoad' | 'new' | 'info' | 'sheet' | 'projectOptions' | 'envOptions'
  | 'classProps' | 'calcOrder' | 'insertFile' | 'about' | 'newClass' | 'imageSave' | 'imageLoad' | 'linkStyle' | 'deleteClasses' | 'print' | 'exportVdr';
/// Слои схемы (меню «Формат → Слои» оригинала): сетка, имиджи, связи, графика.
export interface Layers { grid: boolean; images: boolean; links: boolean; graphics: boolean }

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
  checkProject: () => Promise<void>;
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
  bottomTab: 'messages' | 'graphs' | 'debug' | 'search';
  setBottomTab: (t: 'messages' | 'graphs' | 'debug' | 'search') => void;
  searchQuery: string;
  setSearchQuery: (q: string) => void;
  /// перейти к строке в редакторе кода после выбора имиджа
  gotoLine: { class: string; line: number } | null;
  setGotoLine: (g: { class: string; line: number } | null) => void;
  paletteOpen: boolean;
  setPaletteOpen: (v: boolean) => void;
  dialog: DialogId | null;
  setDialog: (d: DialogId | null) => void;
  /// связь, выбранная для диалога свойств (класс схемы + handle)
  dialogLink: { class: string; handle: number } | null;
  setDialogLink: (l: { class: string; handle: number } | null) => void;
  layers: Layers;
  toggleLayer: (k: keyof Layers) => void;
  traceCount: number;
  setTraceCount: (n: number) => void;
  openProject: (path: string) => Promise<void>;
  /// выбранный графический объект окна модели
  pickedObject: { win: string; handle: number } | null;
  pickObject: (p: { win: string; handle: number } | null) => void;
  /// буфер обмена схемы: экземпляры (класс, имя, смещение)
  schemeClipboard: { class: string; name: string; x: number; y: number }[];
  setSchemeClipboard: (c: { class: string; name: string; x: number; y: number }[]) => void;
  helpTopic: string | null;
  setHelpTopic: (t: string | null) => void;
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
  theme: (new URLSearchParams(location.search).get('theme') as 'light' | 'dark' | null) || (localStorage.getItem('theme') as 'light' | 'dark') || 'light',
  load: async () => {
    const project = await api.project();
    set({ project, schemePath: [project.root], selectedClass: project.root, unsaved: project.unsaved });
    await get().refreshInstances();
    get().checkProject();
  },
  /// сводка проверки компилятором Stratum 2000 — в «Сообщения»
  checkProject: async () => {
    try {
      const list = await api.check();
      for (const e of list) get().say({ level: e.stored ? 'info' : 'error', where: `${e.class}, строка ${e.line}`, text: e.stored ? `текущий компилятор Stratum 2000 не принимает текст (${e.message}); оригинал исполняет сохранённый байт-код` : `Stratum 2000 не примет текст: ${e.message}` });
    } catch { /* ядро без /api/check */ }
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
  searchQuery: '',
  setSearchQuery: searchQuery => set({ searchQuery }),
  gotoLine: null,
  setGotoLine: gotoLine => set({ gotoLine }),
  paletteOpen: false,
  setPaletteOpen: paletteOpen => set({ paletteOpen }),
  dialog: null,
  setDialog: dialog => set({ dialog }),
  dialogLink: null,
  setDialogLink: dialogLink => set({ dialogLink }),
  layers: { grid: true, images: true, links: true, graphics: true },
  toggleLayer: k => set(s => ({ layers: { ...s.layers, [k]: !s.layers[k] } })),
  traceCount: 0,
  setTraceCount: traceCount => set({ traceCount }),
  openProject: async path => {
    await api.open(path);
    const project = await api.project();
    set({ project, schemePath: [project.root], selectedClass: project.root, selectedInstance: null, unsaved: false, messages: [] });
    await get().refreshInstances();
    get().checkProject();
    get().showToast('Проект открыт');
  },
  pickedObject: null,
  pickObject: pickedObject => set({ pickedObject }),
  schemeClipboard: [],
  setSchemeClipboard: schemeClipboard => set({ schemeClipboard }),
  helpTopic: null,
  setHelpTopic: helpTopic => set({ helpTopic }),
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
