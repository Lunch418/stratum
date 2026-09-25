// Свойства графического объекта окна модели (диалог «Свойства двухмерного
// объекта» оригинала как панель): положение, линия, заливка, точки.
import { useEffect, useState } from 'react';
import { api, type ObjectProps as Props } from '../api';
import { useStore } from '../store';
import { HyperFields } from './ObjectDialog';

export function ObjectProps() {
  const picked = useStore(s => s.pickedObject);
  const pickObject = useStore(s => s.pickObject);
  const frame = useStore(s => s.frame);
  const showToast = useStore(s => s.showToast);
  const [o, setO] = useState<Props | null>(null);

  useEffect(() => {
    if (!picked) { setO(null); return; }
    let alive = true;
    api.object(picked.win, picked.handle).then(r => alive && setO(r)).catch(() => alive && setO(null));
    return () => { alive = false; };
  }, [picked?.win, picked?.handle, frame?.tick]);

  if (!picked) return null;
  if (!o) return <div className="muted" style={{ padding: 10, fontSize: 12 }}>Объект #{picked.handle} исчез.</div>;

  const set = (field: string, value: string | number) =>
    api.objectSet(picked.win, picked.handle, field, value).then(() => { showToast('Объект изменён'); return api.object(picked.win, picked.handle); }).then(setO);
  const numField = (label: string, field: keyof Props & string, step = 1) => (
    <label className="prop">
      <span>{label}</span>
      <input type="number" step={step} className="mono" defaultValue={Number(o[field])} key={`${field}:${o[field]}`}
        onBlur={e => Number(e.target.value) !== Number(o[field]) && set(field, e.target.value)}
        onKeyDown={e => e.key === 'Enter' && (e.target as HTMLInputElement).blur()} />
    </label>
  );

  return (
    <>
      <div className="panel-title">Объект #{o.handle} · {kindName(o.kind)} <span className="spacer" />
        <button aria-label="Снять выбор" className="small ghost" onClick={() => pickObject(null)} title="Снять выбор">×</button>
      </div>
      <div className="props">
        <label className="prop"><span>Имя</span><input type="text" defaultValue={o.name} key={'name' + o.name} onBlur={e => e.target.value !== o.name && set('name', e.target.value)} /></label>
        <div className="prop-grid">
          {numField('X', 'x')}{numField('Y', 'y')}{numField('Ширина', 'w')}{numField('Высота', 'h')}
          {numField('Угол', 'angle', 0.1)}{numField('Прозр.', 'alpha')}
        </div>
        <label className="prop"><span>Показан</span><input type="checkbox" checked={o.visible} onChange={e => set('visible', e.target.checked ? 1 : 0)} /></label>
        {o.zorder !== null && numField('Z-порядок', 'zorder')}
        {o.pen && (
          <div className="prop-grid">
            <label className="prop"><span>Линия</span><input type="color" value={o.pen.color} onChange={e => set('pen.color', e.target.value)} /></label>
            <label className="prop"><span>Толщина</span><input type="number" className="mono" defaultValue={o.pen.width} key={'pw' + o.pen.width} onBlur={e => Number(e.target.value) !== o.pen!.width && set('pen.width', e.target.value)} /></label>
            <label className="prop"><span>Стиль</span>
              <select value={o.pen.style} onChange={e => set('pen.style', e.target.value)}>
                <option value={0}>сплошная</option><option value={1}>штрих</option><option value={2}>точки</option><option value={3}>штрих-точка</option><option value={5}>нет</option>
              </select>
            </label>
          </div>
        )}
        {o.brush && (
          <div className="prop-grid">
            <label className="prop"><span>Заливка</span><input type="color" value={o.brush.color} onChange={e => set('brush.color', e.target.value)} /></label>
            <label className="prop"><span>Стиль</span>
              <select value={o.brush.style} onChange={e => set('brush.style', e.target.value)}>
                <option value={0}>сплошная</option><option value={1}>нет</option><option value={2}>штриховка</option>
              </select>
            </label>
          </div>
        )}
        {o.text !== undefined && <label className="prop"><span>Текст</span><input type="text" defaultValue={o.text} key={'t' + o.handle + o.text} onBlur={e => e.target.value !== o.text && set('text', e.target.value)} /></label>}
        {o.font && (
          <div className="prop-grid">
            <label className="prop"><span>Шрифт</span><input type="text" defaultValue={o.font.face} key={'ff' + o.font.face} onBlur={e => e.target.value !== o.font!.face && set('font.face', e.target.value)} /></label>
            <label className="prop"><span>Кегль</span><input type="number" className="mono" defaultValue={o.font.size} key={'fs' + o.font.size} onBlur={e => Number(e.target.value) !== o.font!.size && set('font.size', e.target.value)} /></label>
            <label className="prop"><span>Цвет</span><input type="color" value={o.font.fg} onChange={e => set('text.fg', e.target.value)} /></label>
            <label className="prop"><span>Фон</span><input type="color" value={o.font.bg} onChange={e => set('text.bg', e.target.value)} /></label>
            <label className="check"><input type="checkbox" checked={o.font.bold} onChange={e => set('font.bold', e.target.checked ? 1 : 0)} />жирный</label>
            <label className="check"><input type="checkbox" checked={o.font.italic} onChange={e => set('font.italic', e.target.checked ? 1 : 0)} />курсив</label>
            <label className="check"><input type="checkbox" checked={o.font.underline} onChange={e => set('font.underline', e.target.checked ? 1 : 0)} />подчёркнутый</label>
          </div>
        )}
        {o.class && (
          <div className="prop-grid">
            <div className="prop"><span>Контрол</span><span className="mono">{o.class}</span></div>
            <label className="check"><input type="checkbox" checked={!!o.enabled} onChange={e => set('enabled', e.target.checked ? 1 : 0)} />доступен</label>
            {(o.class === 'CHECKBOX' || o.class === 'RADIOBUTTON') && <label className="check"><input type="checkbox" checked={!!o.checked} onChange={e => set('checked', e.target.checked ? 1 : 0)} />отмечен</label>}
          </div>
        )}
        {o.view3d && (
          <details open>
            <summary className="muted small">Проекция 3D · пространство #{o.view3d.space}, объектов {o.view3d.objects}</summary>
            {o.view3d.camera ? <>
              <div className="muted small">Камера #{o.view3d.camera.handle}{o.view3d.camera.name ? ' · ' + o.view3d.camera.name : ''}</div>
              {(['pos', 'target', 'up'] as const).map(k => (
                <label key={k} className="prop"><span>{{ pos: 'Положение', target: 'Цель', up: 'Верх' }[k]}</span>
                  <input type="text" className="mono" defaultValue={o.view3d!.camera![k].join(', ')} key={k + o.view3d!.camera![k].join()}
                    onBlur={e => e.target.value !== o.view3d!.camera![k].join(', ') && set('camera.' + k, e.target.value)} onKeyDown={e => e.key === 'Enter' && (e.target as HTMLInputElement).blur()} /></label>
              ))}
              <div className="prop-grid">
                <label className="prop"><span>Фокус</span><input type="number" className="mono" defaultValue={o.view3d.camera.focus} key={'f' + o.view3d.camera.focus} onBlur={e => set('camera.focus', e.target.value)} title="0 — ортогональная проекция" /></label>
                <label className="prop"><span>Кадр</span><input type="number" className="mono" defaultValue={o.view3d.camera.extent} key={'x' + o.view3d.camera.extent} onBlur={e => set('camera.extent', e.target.value)} title="Половина ширины кадра; 0 — по сцене" /></label>
              </div>
            </> : <div className="muted small">Камеры нет.</div>}
            <div style={{ display: 'flex', gap: 6, marginTop: 4 }}>
              <button className="small" onClick={() => set('camera.new', 1)}>Новая камера</button>
              <button className="small" onClick={() => set('camera.duplicate', 1)} disabled={!o.view3d.camera}>Дублировать камеру</button>
            </div>
          </details>
        )}
        {o.view3d && o.view3d.materials.length > 0 && (
          <details>
            <summary className="muted small">Материалы ({o.view3d.materials.length})</summary>
            {o.view3d.materials.map(m => (
              <label key={m.handle} className="prop"><span>{m.name || '#' + m.handle}</span><input type="color" value={m.color} onChange={e => set('material.' + m.handle, e.target.value)} /></label>
            ))}
          </details>
        )}
        <details open={!!o.hyper}>
          <summary className="muted small">Гипербаза{o.hyper ? ' · ' + (o.hyper.target || '—') : ''}</summary>
          <HyperFields o={o} set={set} />
        </details>
        {o.points && (
          <details>
            <summary className="muted small">Точки ({o.points.length})</summary>
            <table className="vars"><tbody>
              {o.points.slice(0, 200).map((p, i) => <tr key={i}><td className="muted">{i + 1}</td><td className="num">{p[0]}</td><td className="num">{p[1]}</td></tr>)}
            </tbody></table>
          </details>
        )}
      </div>
    </>
  );
}

function kindName(k: string) {
  return { polyline: 'линия', bitmap: 'растр', text: 'текст', control: 'контрол', group: 'группа', view3d: 'проекция 3D' }[k] ?? k;
}
