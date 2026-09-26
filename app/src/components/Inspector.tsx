// Инспектор: переменные имиджа (таблица как во вкладке «Переменные»
// диалога имиджа) и живые значения выбранного экземпляра (Watch).
import { useEffect, useRef, useState } from 'react';
import { api, type Variable } from '../api';
import { classByName, useStore } from '../store';
import { ObjectProps } from './ObjectProps';

export function Inspector() {
  const project = useStore(s => s.project);
  const selected = useStore(s => s.selectedClass);
  const instance = useStore(s => s.selectedInstance);
  const frame = useStore(s => s.frame);
  const updateClass = useStore(s => s.updateClass);
  const showToast = useStore(s => s.showToast);
  const klass = classByName(project, selected);
  const [live, setLive] = useState<Map<string, string>>(new Map());
  const [changed, setChanged] = useState<Set<string>>(new Set());
  const prev = useRef<Map<string, string>>(new Map());
  const [renaming, setRenaming] = useState<string | null>(null);
  // выражения-наблюдения: считаются ядром в контексте выбранного экземпляра
  const [watches, setWatches] = useState<string[]>(() => { try { return JSON.parse(localStorage.getItem('watches') ?? '[]'); } catch { return []; } });
  const [watchValues, setWatchValues] = useState<Map<string, string>>(new Map());
  const [newWatch, setNewWatch] = useState('');
  useEffect(() => { try { localStorage.setItem('watches', JSON.stringify(watches)); } catch { /* приватный режим */ } }, [watches]);
  useEffect(() => {
    if (instance === null || !watches.length) { setWatchValues(new Map()); return; }
    let alive = true;
    const run = async () => {
      const m = new Map<string, string>();
      for (const w of watches) {
        try { const r = await api.eval(instance, w); m.set(w, r.ok ? (r.value ?? '') : `⚠ ${r.error}`); } catch { m.set(w, '—'); }
      }
      if (alive) setWatchValues(m);
    };
    run();
    const id = setInterval(run, frame?.running ? 300 : 1500);
    return () => { alive = false; clearInterval(id); };
  }, [instance, watches.join('\n'), frame?.running, frame?.tick && !frame.running ? frame.tick : 0]);
  const [confirmDelete, setConfirmDelete] = useState(false);
  const reload = useStore(s => s.reload);
  const say = useStore(s => s.say);
  const select = useStore(s => s.select);
  useEffect(() => { setRenaming(null); setConfirmDelete(false); }, [selected]);

  async function doRename() {
    if (!klass || renaming === null) return;
    const to = renaming.trim();
    if (!to || to === klass.name) { setRenaming(null); return; }
    try { await api.renameClass(klass.name, to); setRenaming(null); await reload(); select(to, instance); showToast('Переименовано'); }
    catch (e) { say({ level: 'error', where: klass.name, text: String(e) }); }
  }
  async function doDelete() {
    if (!klass) return;
    try { await api.deleteClass(klass.name); setConfirmDelete(false); await reload(); showToast('Имидж удалён'); }
    catch (e) { setConfirmDelete(false); say({ level: 'error', where: klass.name, text: String(e) }); }
  }

  // живые значения: при работе — часто, на паузе — при смене такта
  useEffect(() => {
    if (instance === null) { setLive(new Map()); return; }
    let alive = true;
    const tickOnce = async () => {
      try {
        const w = await api.watch(instance);
        if (!alive) return;
        const m = new Map(w.vars);
        const diff = new Set<string>();
        for (const [k, v] of m) if (prev.current.get(k) !== undefined && prev.current.get(k) !== v) diff.add(k);
        prev.current = m; setLive(m); setChanged(diff);
      } catch { /* ядро перезапускается */ }
    };
    tickOnce();
    const id = setInterval(tickOnce, frame?.running ? 200 : 1000);
    return () => { alive = false; clearInterval(id); };
  }, [instance, frame?.running, frame?.tick && !frame.running ? frame.tick : 0]);

  if (!klass) return <><div className="panel-title">Инспектор</div><div className="empty">Выберите имидж в иерархии или блок на схеме — здесь появятся его переменные.</div></>;

  const vars: Variable[] = [
    ...klass.vars,
    ...klass.declared.filter(d => !klass.vars.some(v => v.name.toLowerCase() === d.name.toLowerCase()))
      .map(d => ({ name: d.name, type: d.type, default: '', description: '', local: false, flags: 0x20000 })),
  ];

  async function edit(index: number, field: keyof Variable, value: string) {
    if (!klass || klass.library) return;
    const next = klass.vars.map((v, i) => i === index ? { ...v, [field]: value } : v);
    updateClass({ ...klass, vars: next });
    await api.setVars(klass.name, next);
    useStore.getState().markUnsaved();
    showToast('Переменная изменена');
  }

  return (
    <>
      <div className="panel-title">Инспектор <span className="spacer" /><span className="muted">{klass.name}</span></div>
      <div className="scroll">
        <div style={{ padding: '8px 10px', fontSize: 12 }}>
          {klass.description && <div className="muted" style={{ marginBottom: 6 }}>{klass.description}</div>}
          <div className="muted">Детей на схеме: {klass.children.length} · связей: {klass.links.length}{klass.library ? ' · библиотечный, только чтение' : ''}</div>
          {!klass.library && (
            <div style={{ display: 'flex', gap: 6, marginTop: 8 }}>
              <button className="small" onClick={() => setRenaming(klass.name)}>Переименовать…</button>
              <button className="small" onClick={() => setConfirmDelete(true)} disabled={project?.root.toLowerCase() === klass.name.toLowerCase()}>Удалить имидж</button>
            </div>
          )}
          {renaming !== null && (
            <form style={{ display: 'flex', gap: 6, marginTop: 8 }} onSubmit={e => { e.preventDefault(); doRename(); }}>
              <input autoFocus type="text" value={renaming} onChange={e => setRenaming(e.target.value)} style={{ flex: 1 }} onKeyDown={e => e.key === 'Escape' && setRenaming(null)} />
              <button type="submit" className="small primary" style={{ minWidth: 0 }}>Ок</button>
            </form>
          )}
          {confirmDelete && (
            <div style={{ marginTop: 8, display: 'flex', gap: 6, alignItems: 'center' }}>
              <span>Удалить «{klass.name}»?</span>
              <button className="small" onClick={doDelete}>Да</button>
              <button className="small ghost" onClick={() => setConfirmDelete(false)}>Нет</button>
            </div>
          )}
        </div>
        <ObjectProps />
        {instance !== null && (
          <>
            <div className="panel-title">Наблюдение <span className="spacer" /><span className="muted">{watches.length}</span></div>
            <table className="vars">
              <tbody>
                {watches.map(w => (
                  <tr key={w}>
                    <td className="mono" style={{ width: '50%' }}>{w}</td>
                    <td className="num">{watchValues.get(w) ?? '…'}</td>
                    <td style={{ width: 28, padding: 0 }}><button aria-label="Убрать" className="small ghost" onClick={() => setWatches(ws => ws.filter(x => x !== w))} title="Убрать">×</button></td>
                  </tr>
                ))}
                <tr>
                  <td colSpan={3} style={{ padding: '3px 4px' }}>
                    <form onSubmit={e => { e.preventDefault(); const v = newWatch.trim(); if (v && !watches.includes(v)) setWatches(ws => [...ws, v]); setNewWatch(''); }}>
                      <input type="text" className="mono" placeholder="выражение, напр. sqrt(x*x+y*y)" value={newWatch} onChange={e => setNewWatch(e.target.value)} style={{ width: '100%' }} />
                    </form>
                  </td>
                </tr>
              </tbody>
            </table>
          </>
        )}
        <div className="panel-title">Переменные <span className="spacer" /><span className="muted">{vars.length}</span></div>
        {!vars.length && <div className="empty">У имиджа нет переменных. Объявите их в тексте имиджа — они появятся здесь и станут портами на схеме.</div>}
        {vars.length > 0 && <table className={`vars${instance !== null ? ' live' : ''}`}>
          <thead><tr><th>Имя</th><th>Тип</th><th>По умолчанию</th><th>{instance !== null ? 'Сейчас' : 'Описание'}</th>{instance !== null && <th></th>}</tr></thead>
          <tbody>
            {vars.map((v, i) => (
              <tr key={v.name}>
                <td title={`${v.description || v.name}\nДвойной щелчок — где используется`} style={{ cursor: 'help' }}
                  onDoubleClick={() => { const st = useStore.getState(); st.setSearchQuery(v.name); st.setBottomTab('search'); }}>
                  {v.name}{v.local && <span className="muted"> local</span>}</td>
                <td><span className={`chip ${v.type.toUpperCase()}`} title={v.type.toUpperCase()}>{v.type.toUpperCase()}</span></td>
                <td className="num">
                  {klass.library || i >= klass.vars.length ? v.default : <input defaultValue={v.default} onBlur={e => e.target.value !== v.default && edit(i, 'default', e.target.value)} />}
                </td>
                {instance !== null
                  ? <td className={`num${changed.has(v.name) ? ' changed' : ''}`}>
                      <input key={live.get(v.name)} defaultValue={live.get(v.name) ?? ''} disabled={!!frame?.running}
                        onBlur={e => { if (e.target.value !== live.get(v.name)) api.setValue(instance, v.name, e.target.value).then(() => showToast('Значение записано')); }} />
                    </td>
                  : <td className="muted">{v.description}</td>}
                {instance !== null && (
                  <td style={{ width: 28, padding: 0 }}>
                    {v.type.toUpperCase() === 'FLOAT' && (
                      <button aria-label="На график" className="small ghost" title="На график" onClick={() => api.traceAdd(instance, v.name).then(() => { useStore.getState().setBottomTab('graphs'); showToast('Добавлено на график'); })}>∿</button>
                    )}
                  </td>
                )}
              </tr>
            ))}
          </tbody>
        </table>}
      </div>
    </>
  );
}
