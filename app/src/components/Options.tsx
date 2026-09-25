// Диалоги параметров в структуре оригинала: «Параметры листа» (сетка, окно,
// слои, разное), «Параметры проекта» (вычисления, методы, переменные,
// информация), «Параметры среды», «Свойства имиджа», «Порядок вычислений»,
// выбор файла и «О программе».
import { useEffect, useState } from 'react';
import { api, type ProjectProperty, type Sheet } from '../api';
import { classByName, useStore } from '../store';

export function Tabs<T extends string>({ tabs, value, onChange }: { tabs: [T, string][]; value: T; onChange: (t: T) => void }) {
  return (
    <div className="tabs small-tabs dialog-tabs">
      {tabs.map(([id, title]) => <button key={id} type="button" className={value === id ? 'active' : ''} onClick={() => onChange(id)}>{title}</button>)}
    </div>
  );
}

export function Frame({ title, width, children, onClose, onSubmit, action = 'OK' }: { title: string; width?: number; children: React.ReactNode; onClose: () => void; onSubmit?: () => void; action?: string }) {
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <form className="modal" style={width ? { width: `min(${width}px, 94vw)` } : undefined} onMouseDown={e => e.stopPropagation()}
        onSubmit={e => { e.preventDefault(); onSubmit?.(); }}>
        <div className="panel-title">{title}<span className="spacer" /><button type="button" className="small ghost" onClick={onClose}>×</button></div>
        {children}
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={onClose}>{onSubmit ? 'Отмена' : 'Закрыть'}</button>
          {onSubmit && <button type="submit" className="primary">{action}</button>}
        </div>
      </form>
    </div>
  );
}

export const Check = ({ label, value, onChange, disabled }: { label: string; value: boolean; onChange: (v: boolean) => void; disabled?: boolean }) => (
  <label className="check"><input type="checkbox" checked={value} disabled={disabled} onChange={e => onChange(e.target.checked)} />{label}</label>
);
export const Radio = ({ label, on, onChange, disabled }: { label: string; on: boolean; onChange: () => void; disabled?: boolean }) => (
  <label className="check"><input type="radio" checked={on} disabled={disabled} onChange={onChange} />{label}</label>
);
export const Num = ({ label, value, onChange, step = 1 }: { label: string; value: number; onChange: (v: number) => void; step?: number }) => (
  <label className="prop"><span>{label}</span><input type="number" step={step} value={value} onChange={e => onChange(Number(e.target.value))} /></label>
);

export const defaultSheet: Sheet = { gridOrigin: [0, 0], gridStep: [10, 10], gridVisible: false, gridSnap: false, windowStyle: 'default', windowSize: 'space', windowWh: [0, 0], windowFixed: false, hscroll: false, vscroll: false, autoOrigin: true, layers: 0xffffffff, noSubwindows: false };

// ── Параметры листа ────────────────────────────────────────────────────────
export function SheetDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const name = useStore(s => s.selectedClass);
  const reload = useStore(s => s.reload);
  const markUnsaved = useStore(s => s.markUnsaved);
  const cls = classByName(project, name);
  const [tab, setTab] = useState<'grid' | 'window' | 'layers' | 'misc'>('grid');
  const [sh, setSh] = useState<Sheet>(() => cls?.sheet ?? defaultSheet);
  if (!cls) return null;
  const set = (p: Partial<Sheet>) => setSh(s => ({ ...s, ...p }));
  const layerOn = (i: number) => ((sh.layers >>> i) & 1) === 1;
  const layersMask = (v: boolean, i: number) => set({ layers: (v ? (sh.layers | (1 << i)) : (sh.layers & ~(1 << i))) >>> 0 });
  async function submit() {
    await api.setSheet(cls!.name, sh);
    markUnsaved(); await reload(); onClose();
  }
  return (
    <Frame title={`Параметры листа — ${cls.name}`} width={560} onClose={onClose} onSubmit={submit}>
      <Tabs tabs={[['grid', 'Сетка'], ['window', 'Окно'], ['layers', 'Слои'], ['misc', 'Разное']]} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'grid' && (
          <div className="dialog-cols">
            <fieldset><legend>Сетка</legend>
              <Num label="Начало по X" value={sh.gridOrigin[0]} onChange={v => set({ gridOrigin: [v, sh.gridOrigin[1]] })} />
              <Num label="Начало по Y" value={sh.gridOrigin[1]} onChange={v => set({ gridOrigin: [sh.gridOrigin[0], v] })} />
              <Num label="Шаг по X" value={sh.gridStep[0]} onChange={v => set({ gridStep: [Math.max(1, v), sh.gridStep[1]] })} />
              <Num label="Шаг по Y" value={sh.gridStep[1]} onChange={v => set({ gridStep: [sh.gridStep[0], Math.max(1, v)] })} />
            </fieldset>
            <fieldset><legend>Показ</legend>
              <Check label="Видимая" value={sh.gridVisible} onChange={v => set({ gridVisible: v })} />
              <Check label="Привязка" value={sh.gridSnap} onChange={v => set({ gridSnap: v })} />
              <div className="muted small">Привязка действует при перемещении блоков схемы и объектов рисунка.</div>
            </fieldset>
          </div>
        )}
        {tab === 'window' && <>
          <div className="dialog-cols">
            <fieldset><legend>Стиль</legend>
              {([['default', 'По умолчанию'], ['mdi', 'Окно среды'], ['dialog', 'Диалог'], ['popup', 'Всплывающее окно']] as const).map(([v, t]) => <Radio key={v} label={t} on={sh.windowStyle === v} onChange={() => set({ windowStyle: v })} />)}
              <Check label="Нельзя менять размер" value={sh.windowFixed} onChange={v => set({ windowFixed: v })} />
            </fieldset>
            <fieldset><legend>Размер</legend>
              {([['space', 'По размеру пространства'], ['default', 'По умолчанию'], ['max', 'Максимизированное'], ['min', 'Минимизированное'], ['fixed', 'Установить']] as const).map(([v, t]) => <Radio key={v} label={t} on={sh.windowSize === v} onChange={() => set({ windowSize: v })} />)}
              {sh.windowSize === 'fixed' && <div style={{ display: 'flex', gap: 8 }}>
                <Num label="Ширина" value={sh.windowWh[0]} onChange={v => set({ windowWh: [v, sh.windowWh[1]] })} />
                <Num label="Высота" value={sh.windowWh[1]} onChange={v => set({ windowWh: [sh.windowWh[0], v] })} />
              </div>}
            </fieldset>
          </div>
          <fieldset><legend>Прокрутка</legend>
            <Check label="Горизонтальная прокрутка" value={sh.hscroll} onChange={v => set({ hscroll: v })} />
            <Check label="Вертикальная прокрутка" value={sh.vscroll} onChange={v => set({ vscroll: v })} />
            <Check label="Автоматическое определение начала" value={sh.autoOrigin} onChange={v => set({ autoOrigin: v })} />
          </fieldset>
        </>}
        {tab === 'layers' && <>
          <div className="muted small" style={{ marginBottom: 6 }}>Видимые слои листа (0–31). Объекты и связи со скрытого слоя не показываются в окне модели.</div>
          <div className="layer-grid">
            {Array.from({ length: 32 }, (_, i) => <Check key={i} label={String(i)} value={layerOn(i)} onChange={v => layersMask(v, i)} />)}
          </div>
          <div style={{ display: 'flex', gap: 8, marginTop: 8 }}>
            <button type="button" className="small" onClick={() => set({ layers: 0xffffffff })}>Все</button>
            <button type="button" className="small" onClick={() => set({ layers: 0 })}>Ни одного</button>
          </div>
        </>}
        {tab === 'misc' && <fieldset><legend>Параметры</legend>
          <Check label="Запретить работу подокон" value={sh.noSubwindows} onChange={v => set({ noSubwindows: v })} />
          <div className="muted small">Параметры хранятся в родном формате проекта (<span className="mono">sheet</span> в <span className="mono">.strat.json</span>) и не попадают в <span className="mono">.cls</span> при экспорте.</div>
        </fieldset>}
      </div>
    </Frame>
  );
}

// ── Параметры проекта ──────────────────────────────────────────────────────
type Props = Record<string, number | string | undefined>;
function propsFrom(list: ProjectProperty[]): Props {
  const o: Props = {};
  for (const p of list) o[p.key] = p.int ?? p.text;
  return o;
}

export function ProjectOptionsDialog({ onClose }: { onClose: () => void }) {
  const [tab, setTab] = useState<'calc' | 'method' | 'vars' | 'info'>('calc');
  const [p, setP] = useState<Props | null>(null);
  const [orig, setOrig] = useState<Props>({});
  const markUnsaved = useStore(s => s.markUnsaved);
  const showToast = useStore(s => s.showToast);
  useEffect(() => { api.projectProperties().then(l => { setP(propsFrom(l)); setOrig(propsFrom(l)); }); }, []);
  if (!p) return null;
  const n = (k: string, d = 0) => typeof p[k] === 'number' ? p[k] as number : d;
  const t = (k: string) => typeof p[k] === 'string' ? p[k] as string : '';
  const set = (k: string, v: number | string | undefined) => setP(s => ({ ...s, [k]: v }));
  const bit = (k: string, b: number) => (n(k) & b) !== 0;
  const setBit = (k: string, b: number, v: boolean) => set(k, v ? n(k) | b : n(k) & ~b);
  async function submit() {
    for (const k of new Set([...Object.keys(p!), ...Object.keys(orig)])) {
      if (p![k] !== orig[k]) await api.setProjectProperty(k, p![k] ?? null);
    }
    markUnsaved(); showToast('Параметры проекта применены'); onClose();
  }
  const runMode = n('run_mode');
  const mathMode = p.MathMode === undefined ? -1 : n('MathMode');
  return (
    <Frame title="Параметры проекта" width={600} onClose={onClose} onSubmit={submit}>
      <Tabs tabs={[['calc', 'Вычисления'], ['method', 'Методы'], ['vars', 'Переменные'], ['info', 'Информация']]} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'calc' && <div className="dialog-cols">
          <fieldset><legend>Вычисления</legend>
            <Radio label="Параметры определяются настройками среды" on={p.run_mode === undefined} onChange={() => { set('run_mode', undefined); set('runtimer', undefined); }} />
            <Radio label="В свободное время" on={runMode === 0 && p.run_mode !== undefined} onChange={() => set('run_mode', 0)} />
            <Radio label="По таймеру" on={runMode === 1} onChange={() => { set('run_mode', 1); if (!n('runtimer')) set('runtimer', 40); }} />
            {runMode === 1 && <Num label="Через, мс" value={n('runtimer', 40)} onChange={v => set('runtimer', Math.max(1, v))} />}
            <Radio label="Всё время (без пауз между тактами)" on={runMode === 2} onChange={() => set('run_mode', 2)} />
            <Num label="Тактов за один цикл" value={n('PerIdle', 1)} onChange={v => set('PerIdle', Math.max(1, v))} />
          </fieldset>
          <fieldset><legend>Обработка математических ошибок</legend>
            <Radio label="Определяется настройками среды" on={mathMode === -1} onChange={() => set('MathMode', undefined)} />
            <Radio label="Остановка вычислений после ошибки" on={mathMode === 0} onChange={() => set('MathMode', 0)} />
            <Radio label="Выдача предупреждений" on={mathMode === 1} onChange={() => set('MathMode', 1)} />
            <Radio label="Занесение в протокол" on={mathMode === 2} onChange={() => set('MathMode', 2)} />
            <Radio label="Не замечать" on={mathMode === 3} onChange={() => set('MathMode', 3)} />
          </fieldset>
        </div>}
        {tab === 'method' && <fieldset><legend>Решение уравнений</legend>
          <label className="prop"><span>Линейные уравнения</span><select value={t('lin_method') || 'gauss'} onChange={e => set('lin_method', e.target.value)}><option value="gauss">Метод Гаусса</option><option value="lsq">Наименьшие квадраты</option></select></label>
          <label className="prop"><span>Нелинейные уравнения</span><select value={t('nl_method') || 'newton'} onChange={e => set('nl_method', e.target.value)}><option value="newton">Ньютон — Рафсон</option></select></label>
          <Num label="Максимальное число итераций" value={n('newton_iter', 50)} onChange={v => set('newton_iter', Math.max(1, v))} />
          <Num label="Допустимая ошибка, 10⁻ⁿ" value={n('newton_eps', 6)} onChange={v => set('newton_eps', v)} />
          <div className="muted small">Дифференциальные уравнения решаются методом Эйлера с шагом такта.</div>
        </fieldset>}
        {tab === 'vars' && <fieldset><legend>Устанавливаемые переменные</legend>
          <Check label="Запоминать устанавливаемые переменные" value={bit('vars_logset', 1)} onChange={v => setBit('vars_logset', 1, v)} />
          <Check label="Автоматически считывать при загрузке проекта" value={bit('vars_preload', 1)} onChange={v => setBit('vars_preload', 1, v)} />
          <Check label="Автоматически записывать при закрытии проекта" value={bit('vars_preload', 2)} onChange={v => setBit('vars_preload', 2, v)} />
          <Check label="Запоминать только до первого шага" value={bit('vars_logset', 2)} onChange={v => setBit('vars_logset', 2, v)} />
          <Check label="Устанавливать при установке по умолчанию" value={bit('vars_logset', 4)} onChange={v => setBit('vars_logset', 4, v)} />
          <label className="prop"><span>Имя файла переменных</span><input type="text" className="mono" value={t('vars_file') || '_preload.stt'} onChange={e => set('vars_file', e.target.value)} /></label>
        </fieldset>}
        {tab === 'info' && <fieldset><legend>Автор и описание</legend>
          <label className="prop"><span>Имя</span><input type="text" value={t('user_name')} onChange={e => set('user_name', e.target.value)} /></label>
          <label className="prop"><span>Почта</span><input type="text" value={t('user_email')} onChange={e => set('user_email', e.target.value)} /></label>
          <label className="prop"><span>Адрес</span><input type="text" value={t('user_addr')} onChange={e => set('user_addr', e.target.value)} /></label>
          <label className="prop" style={{ alignItems: 'start' }}><span>Описание</span><textarea rows={5} value={t('info')} onChange={e => set('info', e.target.value)} /></label>
        </fieldset>}
      </div>
    </Frame>
  );
}

// ── Параметры среды ────────────────────────────────────────────────────────
export interface UserInfo { name: string; org: string; email: string; addr: string; phone: string; notes: string }
export interface EnvOptions {
  autosave: number; pollMs: number; circlePoints: number; gridAuto: boolean; fontSize: number; confirmClose: boolean;
  /// «Параметры среды → Вычисления»: пройденные шаги и индикатор производительности в строке статуса
  statusTicks: boolean; statusPerf: boolean;
  /// «2D-редактор»: курсор-рука при перемещении объектов
  handCursor: boolean;
  /// «Редактор»: выделение синтаксиса, кегль, перенос строк, мини-карта
  syntax: boolean; editorFontSize: number; wordWrap: boolean; minimap: boolean;
  /// «Пользователь»: данные автора и копирование их в новый проект
  user: UserInfo; copyUser: boolean;
}
export const defaultEnv: EnvOptions = {
  autosave: 30, pollMs: 40, circlePoints: 32, gridAuto: false, fontSize: 13, confirmClose: true,
  statusTicks: true, statusPerf: false, handCursor: true, syntax: true, editorFontSize: 13, wordWrap: false, minimap: false,
  user: { name: '', org: '', email: '', addr: '', phone: '', notes: '' }, copyUser: true,
};
export function loadEnv(): EnvOptions {
  try {
    const saved = JSON.parse(localStorage.getItem('env') ?? '{}') as Partial<EnvOptions>;
    return { ...defaultEnv, ...saved, user: { ...defaultEnv.user, ...(saved.user ?? {}) } };
  } catch { return defaultEnv; }
}

export function EnvOptionsDialog({ onClose }: { onClose: () => void }) {
  const [tab, setTab] = useState<'calc' | '2d' | 'editor' | 'libs' | 'user' | 'view'>('calc');
  const [env, setEnv] = useState<EnvOptions>(loadEnv);
  const theme = useStore(s => s.theme);
  const toggleTheme = useStore(s => s.toggleTheme);
  const project = useStore(s => s.project);
  const showToast = useStore(s => s.showToast);
  const libs = [...new Set((project?.classes ?? []).filter(c => c.library).map(c => c.source.replace(/[\\/][^\\/]*$/, '')))];
  const set = (p: Partial<EnvOptions>) => setEnv(e => ({ ...e, ...p }));
  const setUser = (p: Partial<UserInfo>) => setEnv(e => ({ ...e, user: { ...e.user, ...p } }));
  const userField = (label: string, k: keyof UserInfo) => (
    <label className="prop"><span>{label}</span><input type="text" value={env.user[k]} onChange={e => setUser({ [k]: e.target.value })} /></label>
  );
  function submit() {
    try { localStorage.setItem('env', JSON.stringify(env)); } catch { /* приватный режим */ }
    document.documentElement.style.setProperty('--font-size', env.fontSize + 'px');
    window.dispatchEvent(new CustomEvent('env-changed', { detail: env }));
    showToast('Параметры среды сохранены'); onClose();
  }
  return (
    <Frame title="Параметры среды" width={600} onClose={onClose} onSubmit={submit}>
      <Tabs tabs={[['calc', 'Вычисления'], ['2d', '2D-редактор'], ['editor', 'Редактор'], ['libs', 'Библиотеки'], ['user', 'Пользователь'], ['view', 'Вид']]} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'calc' && <fieldset><legend>Вычисления</legend>
          <Num label="Опрос окна модели, мс" value={env.pollMs} onChange={v => set({ pollMs: Math.max(10, v) })} />
          <Num label="Автосохранение проекта, с (0 — выключено)" value={env.autosave} onChange={v => set({ autosave: Math.max(0, v) })} />
          <Check label="Пройденные шаги в строке статуса" value={env.statusTicks} onChange={v => set({ statusTicks: v })} />
          <Check label="Индикатор производительности (тактов в секунду) в строке статуса" value={env.statusPerf} onChange={v => set({ statusPerf: v })} />
          <div className="muted small">Режим вычислений и обработка ошибок задаются на проект: «Параметры → Параметры проекта…».</div>
        </fieldset>}
        {tab === '2d' && <fieldset><legend>Двумерный редактор</legend>
          <Num label="Точек в окружности" value={env.circlePoints} onChange={v => set({ circlePoints: Math.min(360, Math.max(8, v)) })} />
          <Check label="Сетка автоматически включается" value={env.gridAuto} onChange={v => set({ gridAuto: v })} />
          <Check label="Курсор-рука при перемещении объектов" value={env.handCursor} onChange={v => set({ handCursor: v })} />
        </fieldset>}
        {tab === 'editor' && <fieldset><legend>Редактор текста</legend>
          <Check label="Использовать редактор с выделением синтаксиса" value={env.syntax} onChange={v => set({ syntax: v })} />
          <Num label="Кегль текста" value={env.editorFontSize} onChange={v => set({ editorFontSize: Math.min(28, Math.max(9, v)) })} />
          <Check label="Переносить длинные строки" value={env.wordWrap} onChange={v => set({ wordWrap: v })} />
          <Check label="Мини-карта текста" value={env.minimap} onChange={v => set({ minimap: v })} />
          <div className="muted small">Цветовая схема текста следует теме среды («Вид»).</div>
        </fieldset>}
        {tab === 'user' && <fieldset><legend>Ваши данные</legend>
          {userField('Имя', 'name')}{userField('Организация', 'org')}{userField('Почта', 'email')}{userField('Адрес', 'addr')}{userField('Телефон/факс', 'phone')}
          <label className="prop" style={{ alignItems: 'start' }}><span>Примечания</span><textarea rows={3} value={env.user.notes} onChange={e => setUser({ notes: e.target.value })} /></label>
          <Check label="Копировать атрибуты в создаваемый проект" value={env.copyUser} onChange={v => set({ copyUser: v })} />
        </fieldset>}
        {tab === 'libs' && <fieldset><legend>Библиотеки</legend>
          {libs.length ? libs.map(l => <div key={l} className="mono small">{l}</div>) : <div className="muted">Библиотеки не подключены.</div>}
          <div className="muted small" style={{ marginTop: 6 }}>Папки библиотек задаются при запуске ядра (<span className="mono">--lib</span>) и в <span className="mono">project.json → libraries</span>.</div>
        </fieldset>}
        {tab === 'view' && <fieldset><legend>Вид</legend>
          <label className="prop"><span>Тема</span><button type="button" className="small" onClick={toggleTheme}>{theme === 'light' ? 'Светлая → тёмная' : 'Тёмная → светлая'}</button></label>
          <Num label="Размер шрифта интерфейса" value={env.fontSize} onChange={v => set({ fontSize: Math.min(20, Math.max(10, v)) })} />
          <Check label="Предупреждать о несохранённых правках при закрытии" value={env.confirmClose} onChange={v => set({ confirmClose: v })} />
          <label className="prop"><span>Раскладка панелей</span><button type="button" className="small" onClick={() => { try { localStorage.removeItem('layout'); } catch { /* */ } location.reload(); }}>Сбросить</button></label>
        </fieldset>}
      </div>
    </Frame>
  );
}

// ── Свойства имиджа ────────────────────────────────────────────────────────
// Биты флагов имиджа — по корпусу проектов оригинала; названия из диалога
// «Дополнительные свойства имиджа».
const CLASS_FLAGS: [number, string][] = [
  [0x40, 'Имидж нельзя менять'],
  [0x200, 'Системный имидж'],
  [0x1000, 'Имидж является структурированной переменной'],
  [0x10, 'Не записывать переменные'],
  [0x400, 'Автоматически создавать связь при перемещении'],
  [0x800, 'Автоматически удалять связь при перемещении'],
  [0x4000, 'Всегда использовать эту иконку'],
  [0x8000, 'Можно менять размер имиджа'],
];

export function ClassPropsDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const name = useStore(s => s.selectedClass);
  const reload = useStore(s => s.reload);
  const select = useStore(s => s.select);
  const markUnsaved = useStore(s => s.markUnsaved);
  const say = useStore(s => s.say);
  const cls = classByName(project, name);
  const [tab, setTab] = useState<'main' | 'extra' | 'icon' | 'vars'>('main');
  const [newName, setNewName] = useState(cls?.name ?? '');
  const [sets, setSets] = useState<{ file: string; count: number; cols: number }[]>([]);
  const [set, setSet] = useState('');
  useEffect(() => { api.iconSets().then(l => { setSets(l); if (l.length && !set) setSet(l.find(x => /default/i.test(x.file))?.file ?? l[0].file); }).catch(() => {}); }, []);
  const [desc, setDesc] = useState(cls?.description ?? '');
  const [flags, setFlags] = useState(cls?.flags ?? 0);
  if (!cls) return null;
  async function submit() {
    try {
      if (desc !== cls!.description || flags !== cls!.flags) await api.setClassProps(cls!.name, { description: desc, flags });
      if (newName.trim() && newName.trim() !== cls!.name) { await api.renameClass(cls!.name, newName.trim()); select(newName.trim()); }
      markUnsaved(); await reload(); onClose();
    } catch (e) { say({ level: 'error', where: 'имидж', text: String(e) }); }
  }
  return (
    <Frame title={`Свойства имиджа — ${cls.name}`} width={560} onClose={onClose} onSubmit={cls.library ? undefined : submit}>
      <Tabs tabs={[['main', 'Основное'], ['extra', 'Дополнительно'], ['icon', 'Иконка'], ['vars', 'Переменные']]} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'main' && <>
          <div style={{ display: 'flex', gap: 12, alignItems: 'flex-start' }}>
            <img src={api.iconUrl(cls.name)} width={48} height={48} alt="" style={{ borderRadius: 6, border: '1px solid var(--border)' }} />
            <div style={{ flex: 1 }}>
              <label className="prop"><span>Имя класса</span><input type="text" value={newName} disabled={cls.library} onChange={e => setNewName(e.target.value)} /></label>
              <label className="prop"><span>Имя файла</span><input type="text" className="mono" value={cls.source} readOnly /></label>
              <label className="prop" style={{ alignItems: 'start' }}><span>Примечания</span><textarea rows={4} value={desc} disabled={cls.library} onChange={e => setDesc(e.target.value)} /></label>
            </div>
          </div>
          <div className="muted small">Переменных: {cls.vars.length} · блоков на схеме: {cls.children.length} · связей: {cls.links.length} · строк кода: {cls.text.split('\n').length}</div>
        </>}
        {tab === 'extra' && <fieldset><legend>Параметры</legend>
          {CLASS_FLAGS.map(([bit, label]) => <Check key={bit} label={label} value={(flags & bit) !== 0} disabled={cls.library} onChange={v => setFlags(f => v ? f | bit : f & ~bit)} />)}
          <label className="prop"><span>Флаги</span><input type="text" className="mono" value={'0x' + flags.toString(16)} readOnly /></label>
        </fieldset>}
        {tab === 'icon' && <>
          <div style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
            <img src={api.iconUrl(cls.name)} width={32} height={32} alt="" style={{ border: '1px solid var(--border)', borderRadius: 4 }} />
            <label className="prop" style={{ flex: 1 }}><span>Библиотека иконок</span>
              <select value={set} onChange={e => setSet(e.target.value)}>{sets.map(x => <option key={x.file} value={x.file}>{x.file} · {x.count}</option>)}</select></label>
            <button type="button" className="small ghost" disabled={cls.library} onClick={async () => { await api.setClassIcon(cls.name, '', 0); api.bumpIcons(); markUnsaved(); await reload(); }}>По умолчанию</button>
          </div>
          <div className="icon-grid">
            {(() => { const cur = sets.find(x => x.file === set); if (!cur) return null; const url = `/api/file?name=${encodeURIComponent(set)}`;
              return Array.from({ length: cur.count }, (_, i) => (
                <div key={set + i} className="icon-cell" title={`${set} #${i}`}
                  style={{ backgroundImage: `url(${url})`, backgroundPosition: `-${(i % cur.cols) * 32}px -${Math.floor(i / cur.cols) * 32}px` }}
                  onClick={async () => { if (cls.library) return; await api.setClassIcon(cls.name, set, i); api.bumpIcons(); markUnsaved(); await reload(); }} />
              )); })()}
          </div>
          {!sets.length && <div className="muted">Наборы иконок (.dbm) не найдены рядом с библиотеками.</div>}
        </>}
        {tab === 'vars' && <table className="vars">
          <thead><tr><th>Имя</th><th>Тип</th><th>По умолчанию</th><th>Описание</th></tr></thead>
          <tbody>{cls.vars.map(v => <tr key={v.name}><td className="mono">{v.name}</td><td className="muted">{v.type}</td><td className="mono">{v.default}</td><td>{v.description}</td></tr>)}</tbody>
        </table>}
      </div>
    </Frame>
  );
}

// ── Порядок вычислений ─────────────────────────────────────────────────────
export function CalcOrderDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const name = useStore(s => s.selectedClass);
  const reload = useStore(s => s.reload);
  const markUnsaved = useStore(s => s.markUnsaved);
  const cls = classByName(project, name);
  const [order, setOrder] = useState(() => cls?.children.map(c => c.handle) ?? []);
  const [sel, setSel] = useState<number | null>(null);
  if (!cls) return null;
  const move = (d: -1 | 1) => {
    if (sel === null) return;
    const i = order.indexOf(sel); const j = i + d;
    if (i < 0 || j < 0 || j >= order.length) return;
    const next = [...order]; [next[i], next[j]] = [next[j], next[i]]; setOrder(next);
  };
  async function submit() { await api.reorderChildren(cls!.name, order); markUnsaved(); await reload(); onClose(); }
  return (
    <Frame title={`Порядок вычислений — ${cls.name}`} width={480} onClose={onClose} onSubmit={submit}>
      <div className="modal-body">
        <div className="muted small" style={{ marginBottom: 6 }}>Имиджи схемы вычисляются сверху вниз в каждом такте.</div>
        <div style={{ display: 'flex', gap: 8 }}>
          <div className="scroll list" style={{ flex: 1, maxHeight: 320, border: '1px solid var(--border)', borderRadius: 6 }}>
            {order.map((h, i) => { const c = cls.children.find(c => c.handle === h)!; return (
              <div key={h} className={`row${sel === h ? ' selected' : ''}`} style={{ cursor: 'pointer' }} onClick={() => setSel(h)}>
                <span className="muted mono" style={{ width: 28 }}>{i + 1}</span>
                <span>{c.name || c.class}</span>{c.name && <span className="muted"> · {c.class}</span>}
              </div>); })}
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            <button type="button" className="small" disabled={sel === null} onClick={() => move(-1)}>▲ Выше</button>
            <button type="button" className="small" disabled={sel === null} onClick={() => move(1)}>▼ Ниже</button>
          </div>
        </div>
      </div>
    </Frame>
  );
}

// ── Выбор файла ────────────────────────────────────────────────────────────
export function FilePickDialog({ title, ext, onPick, onClose, save }: { title: string; ext: string; onPick: (path: string) => void; onClose: () => void; save?: string }) {
  const [dir, setDir] = useState<string>(() => { try { return localStorage.getItem('browseDir') ?? ''; } catch { return ''; } });
  const [listing, setListing] = useState<{ dir: string; parent: string | null; entries: { name: string; path: string; kind: string }[] } | null>(null);
  const [path, setPath] = useState(save ?? '');
  useEffect(() => { let alive = true; api.browse(dir, ext).then(l => alive && setListing(l)).catch(() => alive && setListing(null)); return () => { alive = false; }; }, [dir, ext]);
  return (
    <Frame title={title} width={640} onClose={onClose} onSubmit={() => path.trim() && onPick(path.trim())} action={save ? 'Сохранить' : 'Открыть'}>
      <div className="modal-body">
        <div className="mono small muted" style={{ display: 'flex', gap: 8, alignItems: 'center' }}>
          <button type="button" className="small" disabled={!listing?.parent} onClick={() => listing?.parent && setDir(listing.parent)}>↑</button>{listing?.dir}
        </div>
        <div className="scroll list" style={{ maxHeight: 300, border: '1px solid var(--border)', borderRadius: 6, marginTop: 6 }}>
          {listing?.entries.map(e => (
            <div key={e.path} className={`row${path === e.path ? ' selected' : ''}`} style={{ cursor: 'pointer' }}
              onClick={() => e.kind === 'file' ? setPath(e.path) : setDir(e.path)} onDoubleClick={() => e.kind === 'file' && onPick(e.path)}>
              <span className="muted" style={{ width: 18 }}>{e.kind === 'file' ? '▫' : '▸'}</span>{e.name}
            </div>
          ))}
          {!listing?.entries.length && <div className="muted" style={{ padding: 10 }}>Нет файлов {ext.split(',').map(e => '.' + e).join(', ')}.</div>}
        </div>
        <input type="text" className="mono" style={{ marginTop: 8 }} value={path} onChange={e => setPath(e.target.value)} placeholder="или полный путь" />
      </div>
    </Frame>
  );
}

// ── О программе ────────────────────────────────────────────────────────────
export function AboutDialog({ onClose }: { onClose: () => void }) {
  return (
    <Frame title="О программе" width={440} onClose={onClose}>
      <div className="modal-body about">
        <div className="about-name">Stratum Modern</div>
        <div className="muted">Среда визуального моделирования — современная реализация Stratum 2000.</div>
        <div className="muted small" style={{ marginTop: 10 }}>Ядро на Rust: форматы <span className="mono">.cls/.spj/.stt/.vdr</span>, интерпретатор, 2D/3D-графика. Интерфейс: React, Monaco, Tauri.</div>
        <div className="muted small" style={{ marginTop: 6 }}>Исходный код: <span className="mono">github.com/Lunch418/stratum</span></div>
      </div>
    </Frame>
  );
}

// ── Удаление имиджей ───────────────────────────────────────────────────────
// Диалог 229 оригинала: список имиджей проекта с числом экземпляров;
// неиспользуемые отмечены заранее.
export function DeleteClassesDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const instances = useStore(s => s.instances);
  const reload = useStore(s => s.reload);
  const say = useStore(s => s.say);
  const showToast = useStore(s => s.showToast);
  const own = (project?.classes ?? []).filter(c => !c.library && c.name !== project?.root);
  const used = new Map<string, number>();
  for (const i of instances) used.set(i.class.toLowerCase(), (used.get(i.class.toLowerCase()) ?? 0) + 1);
  // упоминание в схемах или текстах других имиджей (CreateObject, GetClassFile…)
  // — тоже использование, такие имиджи заранее не отмечаются
  const mentioned = new Set<string>();
  for (const c of project?.classes ?? []) {
    for (const ch of c.children) mentioned.add(ch.class.toLowerCase());
    const t = c.text.toLowerCase();
    for (const o of own) if (o.name !== c.name && t.includes(o.name.toLowerCase())) mentioned.add(o.name.toLowerCase());
  }
  const [picked, setPicked] = useState<Set<string>>(() => new Set(own.filter(c => !used.get(c.name.toLowerCase()) && !mentioned.has(c.name.toLowerCase())).map(c => c.name)));
  const [busy, setBusy] = useState(false);
  async function submit() {
    setBusy(true);
    try {
      for (const n of picked) await api.deleteClass(n);
      await reload(); showToast(`Удалено имиджей: ${picked.size}`); onClose();
    } catch (e) { say({ level: 'error', where: 'имиджи', text: String(e) }); }
    finally { setBusy(false); }
  }
  return (
    <Frame title="Удаление имиджей" width={520} onClose={onClose} onSubmit={picked.size && !busy ? submit : undefined} action={`Удалить (${picked.size})`}>
      <div className="modal-body">
        <div className="muted small" style={{ marginBottom: 6 }}>Отмечены имиджи, у которых нет экземпляров в модели. Корневой имидж удалить нельзя.</div>
        <div className="scroll list" style={{ maxHeight: 340, border: '1px solid var(--border)', borderRadius: 6 }}>
          {own.map(c => { const n = used.get(c.name.toLowerCase()) ?? 0; return (
            <label key={c.name} className="row check" style={{ cursor: 'pointer' }}>
              <input type="checkbox" checked={picked.has(c.name)} onChange={e => setPicked(p => { const s = new Set(p); e.target.checked ? s.add(c.name) : s.delete(c.name); return s; })} />
              <img src={api.iconUrl(c.name)} width={16} height={16} alt="" />
              <span style={{ flex: 1 }}>{c.name}</span>
              <span className="muted" style={{ color: n || mentioned.has(c.name.toLowerCase()) ? undefined : 'var(--state-error)' }}>{n ? `экземпляров: ${n}` : mentioned.has(c.name.toLowerCase()) ? 'упоминается' : 'не используется'}</span>
            </label>); })}
          {!own.length && <div className="muted" style={{ padding: 10 }}>Кроме корневого имиджа удалять нечего.</div>}
        </div>
      </div>
    </Frame>
  );
}
