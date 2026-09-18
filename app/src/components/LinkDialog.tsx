// Редактор связи: пары переменных «источник → приёмник». Пустой список
// удаляет связь.
import { useState } from 'react';
import type { ClassInfo } from '../api';

interface Props {
  source: { label: string; cls: ClassInfo | undefined };
  target: { label: string; cls: ClassInfo | undefined };
  pairs: [string, string][];
  isNew: boolean;
  onSubmit: (pairs: [string, string][]) => void;
  onClose: () => void;
}

export function LinkDialog({ source, target, pairs: initial, isNew, onSubmit, onClose }: Props) {
  const [pairs, setPairs] = useState<[string, string][]>(initial.length ? initial : [['', '']]);
  const srcVars = source.cls?.vars.filter(v => !v.local) ?? [];
  const dstVars = target.cls?.vars.filter(v => !v.local) ?? [];

  function update(i: number, side: 0 | 1, value: string) {
    setPairs(ps => ps.map((p, k) => {
      if (k !== i) return p;
      const next: [string, string] = [...p] as [string, string];
      next[side] = value;
      // одноимённая переменная с другой стороны подставляется сама
      if (side === 0 && !next[1] && dstVars.some(v => v.name.toLowerCase() === value.toLowerCase())) next[1] = value;
      return next;
    }));
  }

  const clean = pairs.filter(p => p[0] && p[1]);
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <form className="modal" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); onSubmit(clean); }}>
        <div className="panel-title">{isNew ? 'Новая связь' : 'Связь'}: {source.label} → {target.label}</div>
        <div className="modal-body">
          <table className="vars">
            <thead><tr><th>{source.label}</th><th></th><th>{target.label}</th><th></th></tr></thead>
            <tbody>
              {pairs.map((p, i) => (
                <tr key={i}>
                  <td><VarPick value={p[0]} vars={srcVars} onChange={v => update(i, 0, v)} /></td>
                  <td className="muted">→</td>
                  <td><VarPick value={p[1]} vars={dstVars} onChange={v => update(i, 1, v)} /></td>
                  <td><button type="button" className="small ghost" onClick={() => setPairs(ps => ps.filter((_, k) => k !== i))} title="Убрать пару">×</button></td>
                </tr>
              ))}
            </tbody>
          </table>
          <button type="button" className="small" style={{ marginTop: 8 }} onClick={() => setPairs(ps => [...ps, ['', '']])}>+ пара</button>
          <div className="muted small" style={{ marginTop: 6 }}>Связанные переменные становятся одной ячейкой: запись с любой стороны видна с другой.</div>
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={onClose}>Отмена</button>
          <button type="submit" className={clean.length ? 'primary' : ''}>{clean.length ? (isNew ? 'Связать' : 'Сохранить') : 'Удалить связь'}</button>
        </div>
      </form>
    </div>
  );
}

function VarPick({ value, vars, onChange }: { value: string; vars: { name: string; type: string }[]; onChange: (v: string) => void }) {
  const known = vars.some(v => v.name === value);
  return (
    <span style={{ display: 'flex', gap: 4 }}>
      <select value={known ? value : ''} onChange={e => onChange(e.target.value)} style={{ flex: 1 }}>
        <option value="">—</option>
        {vars.map(v => <option key={v.name} value={v.name}>{v.name} · {v.type}</option>)}
      </select>
      {!known && value && <input type="text" className="mono" value={value} onChange={e => onChange(e.target.value)} style={{ width: 90 }} />}
    </span>
  );
}
