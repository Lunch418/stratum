// Инспектор: переменные имиджа (таблица как во вкладке «Переменные»
// диалога имиджа) и живые значения выбранного экземпляра (Watch).
import { useEffect, useRef, useState } from 'react';
import { api, type Variable } from '../api';
import { classByName, useStore } from '../store';

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

  if (!klass) return <><div className="panel-title">Инспектор</div><div className="muted" style={{ padding: 12 }}>Ничего не выбрано.</div></>;

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
        </div>
        <div className="panel-title">Переменные</div>
        <table className="vars">
          <thead><tr><th>Имя</th><th>Тип</th><th>По умолчанию</th><th>{instance !== null ? 'Сейчас' : 'Описание'}</th></tr></thead>
          <tbody>
            {vars.map((v, i) => (
              <tr key={v.name}>
                <td title={v.description}>{v.name}{v.local && <span className="muted"> local</span>}</td>
                <td><span className={`chip ${v.type.toUpperCase()}`}>{v.type.toUpperCase()}</span></td>
                <td className="num">
                  {klass.library || i >= klass.vars.length ? v.default : <input defaultValue={v.default} onBlur={e => e.target.value !== v.default && edit(i, 'default', e.target.value)} />}
                </td>
                {instance !== null
                  ? <td className={`num${changed.has(v.name) ? ' changed' : ''}`}>
                      <input key={live.get(v.name)} defaultValue={live.get(v.name) ?? ''} disabled={!!frame?.running}
                        onBlur={e => { if (e.target.value !== live.get(v.name)) api.setValue(instance, v.name, e.target.value).then(() => showToast('Значение записано')); }} />
                    </td>
                  : <td className="muted">{v.description}</td>}
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </>
  );
}
