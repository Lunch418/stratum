// Поиск по всему проекту (диалог «Поиск» оригинала как панель): где искать —
// текст модели, комментарии, имена переменных, их значения, имена имиджей,
// имена объектов (блоки схем и объекты рисунков), дескрипторы, описания;
// различать регистр, только слова целиком. Щелчок ведёт к найденному.
import { useEffect, useState } from 'react';
import { useStore } from '../store';

type Where = 'text' | 'comments' | 'vars' | 'values' | 'classes' | 'objects' | 'handles' | 'info';
interface Hit { class: string; line: number; text: string; kind: 'text' | 'comment' | 'var' | 'value' | 'class' | 'instance' | 'object'; handle?: number; picture?: 'image' | 'icon'; instance?: number; var?: string }
interface Opts { where: Where[]; case: boolean; word: boolean; libs: boolean }

const WHERE: [Where, string, string][] = [
  ['text', 'Текст модели', 'строки текста имиджей'],
  ['comments', 'Комментарии', 'часть строки после //'],
  ['vars', 'Имена переменных', 'переменные имиджей'],
  ['values', 'Значения переменных', 'текущие значения в работающей модели'],
  ['classes', 'Имена имиджей', ''],
  ['objects', 'Имена объектов', 'блоки на схемах и объекты рисунков'],
  ['handles', 'Дескриптор', 'номер блока или графического объекта'],
  ['info', 'Внутренние данные', 'описания имиджей и переменных, значения по умолчанию'],
];
const KIND: Record<Hit['kind'], string> = { text: 'текст', comment: 'комментарий', var: 'переменная', value: 'значение', class: 'имидж', instance: 'блок', object: 'объект' };
const defaults: Opts = { where: ['text', 'comments', 'vars', 'classes', 'objects', 'info'], case: false, word: false, libs: false };

function loadOpts(): Opts {
  try { return { ...defaults, ...JSON.parse(localStorage.getItem('searchOpts') ?? '{}') as Partial<Opts> }; } catch { return defaults; }
}

export function Search() {
  const select = useStore(s => s.select);
  const setTab = useStore(s => s.setTab);
  const setGotoLine = useStore(s => s.setGotoLine);
  const query = useStore(s => s.searchQuery);
  const setQuery = useStore(s => s.setSearchQuery);
  const [opts, setOpts] = useState<Opts>(loadOpts);
  const [showOpts, setShowOpts] = useState(false);
  const [hits, setHits] = useState<Hit[]>([]);
  const [busy, setBusy] = useState(false);
  const set = (p: Partial<Opts>) => setOpts(o => { const n = { ...o, ...p }; try { localStorage.setItem('searchOpts', JSON.stringify(n)); } catch { /* приватный режим */ } return n; });
  const minLen = opts.where.length === 1 && opts.where[0] === 'handles' ? 1 : 2;

  useEffect(() => {
    if (query.trim().length < minLen || !opts.where.length) { setHits([]); return; }
    setBusy(true);
    const id = setTimeout(() => {
      const q = new URLSearchParams({ q: query.trim(), in: opts.where.join(','), libs: opts.libs ? '1' : '0', case: opts.case ? '1' : '0', word: opts.word ? '1' : '0' });
      fetch(`/api/search?${q}`).then(r => r.json()).then(h => { setHits(h); setBusy(false); }).catch(() => setBusy(false));
    }, 200);
    return () => clearTimeout(id);
  }, [query, opts, minLen]);

  function open(h: Hit) {
    if (h.kind === 'instance') { useStore.setState({ schemePath: [h.class], selectedClass: h.class, selectedInstance: null, tab: 'scheme' }); return; }
    if (h.kind === 'object') { select(h.class); setTab(h.picture === 'icon' ? 'icon' : 'picture'); return; }
    if (h.kind === 'value') { select(h.class, h.instance ?? null); return; }
    select(h.class); setTab('code');
    if (h.line) setGotoLine({ class: h.class, line: h.line });
  }

  // «Создавать группу»: найденные объекты рисунка одного имиджа — в группу
  const objectHits = hits.filter(h => h.kind === 'object' && h.handle !== undefined);
  async function makeGroups() {
    const byPic = new Map<string, Hit[]>();
    for (const h of objectHits) { const k = h.class + '\n' + (h.picture ?? 'image'); byPic.set(k, [...(byPic.get(k) ?? []), h]); }
    let made = 0;
    for (const [key, list] of byPic) {
      const [cls, kind] = key.split('\n');
      const url = `/api/picture/${encodeURIComponent(cls)}?kind=${kind}`;
      const st = await (await fetch(url)).json() as { objects: { handle: number; parent: number | null }[] };
      // только объекты верхнего уровня: члены чужих групп не перетаскиваем
      const top = list.map(h => h.handle!).filter(h => st.objects.some(o => o.handle === h && o.parent === null));
      if (top.length < 2) continue;
      const r = await fetch(url, { method: 'POST', body: JSON.stringify({ op: 'group', handles: top }) });
      if (!r.ok) continue;
      const g = (await r.json()).handle as number;
      await fetch(url, { method: 'POST', body: JSON.stringify({ op: 'set', handle: g, field: 'name', value: query.trim() }) });
      made++;
    }
    const s = useStore.getState();
    if (made) { s.markUnsaved(); await s.reload(); }
    s.showToast(made ? `Создано групп: ${made}` : 'Нечего группировать: нужно хотя бы два объекта верхнего уровня в одном рисунке');
  }

  const toggle = (w: Where, v: boolean) => set({ where: v ? [...opts.where, w] : opts.where.filter(x => x !== w) });
  return (
    <>
      <div className="panel-title">Поиск
        <input type="search" autoFocus placeholder="строка для поиска" value={query} onChange={e => setQuery(e.target.value)} style={{ marginLeft: 8, width: 260, textTransform: 'none', letterSpacing: 0, fontSize: 12 }} />
        <label className="search-opt" title="Различать регистр"><input type="checkbox" checked={opts.case} onChange={e => set({ case: e.target.checked })} />Aa</label>
        <label className="search-opt" title="Только слова целиком"><input type="checkbox" checked={opts.word} onChange={e => set({ word: e.target.checked })} />слово</label>
        <label className="search-opt"><input type="checkbox" checked={opts.libs} onChange={e => set({ libs: e.target.checked })} />и в библиотеках</label>
        <button className={`small ghost${showOpts ? ' active' : ''}`} onClick={() => setShowOpts(v => !v)} title="Где искать">Где: {opts.where.length === WHERE.length ? 'везде' : opts.where.length}</button>
        {objectHits.length > 1 && <button className="small ghost" onClick={makeGroups} title="«Создавать группу»: найденные объекты рисунка одного имиджа объединяются в группу с именем строки поиска">Создать группу</button>}
        <span className="spacer" /><span className="muted">{busy ? '…' : hits.length}</span>
      </div>
      {showOpts && (
        <div className="search-where">
          {WHERE.map(([w, label, hint]) => (
            <label key={w} className="check" title={hint}><input type="checkbox" checked={opts.where.includes(w)} onChange={e => toggle(w, e.target.checked)} />{label}</label>
          ))}
          <button className="small" onClick={() => set({ where: WHERE.map(w => w[0]) })}>Везде</button>
        </div>
      )}
      <div className="scroll messages">
        {hits.map((h, i) => (
          <div key={i} className="row" style={{ cursor: 'pointer' }} onClick={() => open(h)}>
            <span className="where">{h.class}{h.line ? `:${h.line}` : ''}</span>
            <span className="muted search-kind">{KIND[h.kind]}</span>
            <span className={h.kind === 'text' ? 'mono' : ''}>{h.text}</span>
          </div>
        ))}
        {!hits.length && query.trim().length >= minLen && !busy && <div className="muted" style={{ padding: 10 }}>{opts.where.length ? 'Ничего не найдено.' : 'Не выбрано, где искать.'}</div>}
      </div>
    </>
  );
}
