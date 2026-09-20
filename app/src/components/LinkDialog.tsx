// Редактор связи: пары переменных «источник → приёмник». Пустой список
// удаляет связь.
import { useState } from 'react';
import type { ClassInfo, LinkStyle } from '../api';

export const defaultLinkStyle: LinkStyle = { color: '', width: 0, disabled: false, arrows: false, layer: 0 };

interface Props {
  source: { label: string; cls: ClassInfo | undefined };
  target: { label: string; cls: ClassInfo | undefined };
  pairs: [string, string][];
  style?: LinkStyle;
  isNew: boolean;
  onSubmit: (pairs: [string, string][], style: LinkStyle) => void;
  onClose: () => void;
}

export function LinkDialog({ source, target, pairs: initial, style: initialStyle, isNew, onSubmit, onClose }: Props) {
  const [pairs, setPairs] = useState<[string, string][]>(initial.length ? initial : [['', '']]);
  const [tab, setTab] = useState<'main' | 'extra'>('main');
  const [style, setStyle] = useState<LinkStyle>(initialStyle ?? defaultLinkStyle);
  const st = (p: Partial<LinkStyle>) => setStyle(s => ({ ...s, ...p }));
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
      <form className="modal" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); onSubmit(clean, style); }}>
        <div className="panel-title">{isNew ? 'Новая связь' : 'Связь'}: {source.label} → {target.label}</div>
        <div className="tabs small-tabs dialog-tabs">
          <button type="button" className={tab === 'main' ? 'active' : ''} onClick={() => setTab('main')}>Основное</button>
          <button type="button" className={tab === 'extra' ? 'active' : ''} onClick={() => setTab('extra')}>Дополнительно</button>
        </div>
        {tab === 'extra' && <div className="modal-body dialog-page">
          <div className="dialog-cols">
            <fieldset><legend>Линия</legend>
              <label className="prop"><span>Цвет</span><span style={{ display: 'flex', gap: 6 }}>
                <input type="color" value={style.color || '#5b6470'} onChange={e => st({ color: e.target.value })} style={{ width: 44, padding: 0 }} />
                <button type="button" className="small ghost" style={{ whiteSpace: 'nowrap' }} onClick={() => st({ color: '' })} disabled={!style.color}>сброс</button></span></label>
              <label className="prop"><span>Толщина</span><input type="number" min={0} max={9} value={style.width} onChange={e => st({ width: Math.max(0, Number(e.target.value)) })} /></label>
              <label className="prop"><span>Слой</span><input type="number" min={0} max={31} value={style.layer} onChange={e => st({ layer: Math.min(31, Math.max(0, Number(e.target.value))) })} /></label>
            </fieldset>
            <fieldset><legend>Поведение</legend>
              <label className="check"><input type="checkbox" checked={style.disabled} onChange={e => st({ disabled: e.target.checked })} />Связь не работает</label>
              <label className="check"><input type="checkbox" checked={style.arrows} onChange={e => st({ arrows: e.target.checked })} />Показывать стрелки</label>
              <div className="muted small">Выключенная связь остаётся на схеме, но переменные не объединяются.</div>
            </fieldset>
          </div>
        </div>}
        <div className="modal-body" style={tab === 'extra' ? { display: 'none' } : undefined}>
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
