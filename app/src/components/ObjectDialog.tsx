// «Параметры двухмерного объекта» оригинала для объекта рисунка имиджа:
// вкладки «Положение», «Линия», «Заливка», «Текст», «BMP», «Группа»,
// «Гипербаза», «Переменные», «Точки», «Информация».
// Правки уходят сразу (каждая — шаг Undo), как в панели инспектора.
import { useState } from 'react';
import type { ObjectProps } from '../api';
import { useStore } from '../store';
import { Frame, Tabs } from './Options';

type Tab = 'place' | 'pen' | 'brush' | 'text' | 'bmp' | 'group' | 'hyper' | 'vars' | 'points' | 'info';

/// «Переменные» объекта: `a,b;x,y` — переменной a этого имиджа сопоставлена
/// переменная (или псевдоним) x другого имиджа.
function parseVars(text: string): Map<string, string> {
  const [names, targets] = text.split(';');
  const a = (names ?? '').split(',').map(s => s.trim()), b = (targets ?? '').split(',').map(s => s.trim());
  return new Map(a.map((n, i): [string, string] => [n.toLowerCase(), b[i] ?? '']).filter(([n]) => n));
}
function VarsTab({ text, vars, onChange }: { text: string; vars: string[]; onChange: (t: string) => void }) {
  const map = parseVars(text);
  const [values, setValues] = useState<Record<string, string>>(() => Object.fromEntries(vars.map(v => [v, map.get(v.toLowerCase()) ?? ''])));
  const commit = (next: Record<string, string>) => {
    const pairs = Object.entries(next).filter(([, t]) => t.trim());
    onChange(pairs.length ? pairs.map(p => p[0]).join(',') + ';' + pairs.map(p => p[1].trim()).join(',') : '');
  };
  return <>
    <div className="muted small">Напротив переменных этого имиджа введите переменные (или псевдонимы) другого имиджа: связь, начатая и законченная на этом объекте, соединит их сама.</div>
    {vars.length === 0 && <div className="muted">У имиджа нет переменных.</div>}
    <table className="vars"><tbody>
      {vars.map(v => (
        <tr key={v}><td className="mono">{v}</td><td>
          <input type="text" className="mono" value={values[v] ?? ''} onChange={e => setValues({ ...values, [v]: e.target.value })}
            onBlur={() => commit(values)} onKeyDown={e => e.key === 'Enter' && (e.target as HTMLInputElement).blur()} />
        </td></tr>
      ))}
    </tbody></table>
  </>;
}
export type SetField = (field: string, value: string | number) => void;

const MODES: [number, string][] = [[0, 'Открыть окно'], [1, 'Запустить Windows-приложение'], [2, 'Загрузить новый проект'], [3, 'Ничего не делать'], [4, 'Выполнить системную команду']];

/// Поля закладки «Гипербаза» — общие для диалога и инспектора окна модели.
export function HyperFields({ o, set }: { o: ObjectProps; set: SetField }) {
  const project = useStore(s => s.project);
  const h = o.hyper;
  const used = !!h;
  const text = (label: string, field: 'target' | 'window' | 'object' | 'effect', hint: string, list?: string) => (
    <label className="prop"><span>{label}</span>
      <input type="text" list={list} disabled={!used} defaultValue={h?.[field] ?? ''} key={field + (h?.[field] ?? '') + o.handle} placeholder={hint}
        onBlur={e => used && e.target.value !== h![field] && set('hyper.' + field, e.target.value)} onKeyDown={e => e.key === 'Enter' && (e.target as HTMLInputElement).blur()} />
    </label>
  );
  return (
    <>
      <label className="check"><input type="checkbox" checked={used} onChange={e => set('hyper.mode', e.target.checked ? 0 : -1)} />Используется — объект открывает страницу по щелчку в окне модели</label>
      <label className="prop"><span>Стиль</span>
        <select disabled={!used} value={h?.mode ?? 0} onChange={e => set('hyper.mode', Number(e.target.value))}>
          {MODES.map(([v, t]) => <option key={v} value={v}>{t}</option>)}
        </select>
      </label>
      {text(h?.mode === 4 ? 'Команда' : h?.mode === 1 ? 'Имя файла' : h?.mode === 2 ? 'Файл проекта' : 'Цель', 'target',
        h?.mode === 4 ? 'CM_PREVPAGE' : 'имидж или файл .vdr', h?.mode === 4 ? 'hyper-commands' : 'hyper-targets')}
      {text('Окно', 'window', 'пусто — то же окно')}
      {text('Объект', 'object', 'имидж, получающий WM_HYPERJUMP', 'hyper-targets')}
      {text('Эффект', 'effect', 'смена страницы без эффекта')}
      <datalist id="hyper-targets">{(project?.classes ?? []).filter(c => !c.library).map(c => <option key={c.name} value={c.name} />)}</datalist>
      <datalist id="hyper-commands"><option value="CM_PREVPAGE">предыдущая страница</option></datalist>
      <div className="muted small">Ссылки работают, пока модель запущена. «Назад» в окне модели возвращает предыдущую страницу.</div>
    </>
  );
}

export interface ObjectVars { text: string; classVars: string[]; onChange: (text: string) => void }

export function ObjectDialog({ o, set, resize, onClose, vars, onOpen, all }: {
  o: ObjectProps; set: SetField; resize: (p: Partial<Record<'x' | 'y' | 'w' | 'h', number>>) => void; onClose: () => void;
  /// закладка «Переменные» (только для рисунка имиджа)
  vars?: ObjectVars;
  /// открыть свойства другого объекта (член группы)
  onOpen?: (handle: number) => void;
  all?: ObjectProps[];
}) {
  const tabs: [Tab, string][] = [['place', 'Положение']];
  if (o.pen) tabs.push(['pen', 'Линия']);
  if (o.brush) tabs.push(['brush', 'Заливка']);
  if (o.font || o.text !== undefined) tabs.push(['text', 'Текст']);
  if (o.src) tabs.push(['bmp', 'BMP']);
  if (o.members) tabs.push(['group', 'Группа']);
  tabs.push(['hyper', 'Гипербаза']);
  if (vars) tabs.push(['vars', 'Переменные']);
  if (o.points) tabs.push(['points', 'Точки']);
  tabs.push(['info', 'Информация']);
  const [tab, setTab] = useState<Tab>('place');
  const num = (label: string, value: number, apply: (v: number) => void, step = 1) => (
    <label className="prop"><span>{label}</span>
      <input type="number" step={step} className="mono" defaultValue={value} key={label + value}
        onBlur={e => Number(e.target.value) !== value && apply(Number(e.target.value))} onKeyDown={e => e.key === 'Enter' && (e.target as HTMLInputElement).blur()} />
    </label>
  );
  return (
    <Frame title={`Параметры объекта #${o.handle}${o.name ? ' · ' + o.name : ''}`} width={520} onClose={onClose}>
      <Tabs tabs={tabs} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'place' && <>
          <label className="prop"><span>Имя</span><input type="text" defaultValue={o.name} key={'n' + o.name} onBlur={e => e.target.value !== o.name && set('name', e.target.value)} /></label>
          <div className="dialog-cols">
            {num('X', o.x, v => resize({ x: v }))}{num('Y', o.y, v => resize({ y: v }))}
            {num('Ширина', o.w, v => resize({ w: v }))}{num('Высота', o.h, v => resize({ h: v }))}
          </div>
          <label className="check"><input type="checkbox" checked={o.visible} onChange={e => set('visible', e.target.checked ? 1 : 0)} />Показан</label>
          <div className="muted small">Тип: {o.kind}{o.parent !== null ? ` · в группе #${o.parent}` : ''}{o.zorder !== null ? ` · Z-порядок ${o.zorder}` : ''}</div>
        </>}
        {tab === 'pen' && o.pen && <>
          <label className="prop"><span>Цвет</span><input type="color" value={o.pen.color} onChange={e => set('pen.color', e.target.value)} /></label>
          {num('Толщина', o.pen.width, v => set('pen.width', v))}
          <label className="prop"><span>Стиль</span>
            <select value={o.pen.style} onChange={e => set('pen.style', e.target.value)}>
              <option value={0}>сплошная</option><option value={1}>штрих</option><option value={2}>точки</option><option value={3}>штрих-точка</option><option value={5}>нет</option>
            </select>
          </label>
        </>}
        {tab === 'brush' && o.brush && <>
          <label className="prop"><span>Цвет</span><input type="color" value={o.brush.color} onChange={e => set('brush.color', e.target.value)} /></label>
          <label className="prop"><span>Стиль</span>
            <select value={o.brush.style} onChange={e => set('brush.style', e.target.value)}>
              <option value={0}>сплошная</option><option value={1}>нет</option><option value={2}>штриховка</option>
            </select>
          </label>
        </>}
        {tab === 'text' && <>
          {o.text !== undefined && <label className="prop"><span>Текст</span><input type="text" defaultValue={o.text} key={'t' + o.text} onBlur={e => e.target.value !== o.text && set('text', e.target.value)} /></label>}
          {o.font && <>
            <label className="prop"><span>Шрифт</span><input type="text" defaultValue={o.font.face} key={'ff' + o.font.face} onBlur={e => e.target.value !== o.font!.face && set('font.face', e.target.value)} /></label>
            {num('Кегль', o.font.size, v => set('font.size', v))}
            <div className="dialog-cols">
              <label className="prop"><span>Цвет</span><input type="color" value={o.font.fg} onChange={e => set('text.fg', e.target.value)} /></label>
              <label className="prop"><span>Фон</span><input type="color" value={o.font.bg} onChange={e => set('text.bg', e.target.value)} /></label>
            </div>
            <label className="check"><input type="checkbox" checked={o.font.bold} onChange={e => set('font.bold', e.target.checked ? 1 : 0)} />жирный</label>
            <label className="check"><input type="checkbox" checked={o.font.italic} onChange={e => set('font.italic', e.target.checked ? 1 : 0)} />курсив</label>
            <label className="check"><input type="checkbox" checked={o.font.underline} onChange={e => set('font.underline', e.target.checked ? 1 : 0)} />подчёркнутый</label>
          </>}
        </>}
        {tab === 'bmp' && o.src && <>
          <div className="muted small">Часть растра, выводимая на экран (исходный прямоугольник, SetBitmapSrcRect2d).</div>
          <div className="dialog-cols">
            {(['X', 'Y', 'Ширина', 'Высота'] as const).map((l, i) => num(l, o.src![i], v => { const r = [...o.src!]; r[i] = v; set('src', r.join(',')); }))}
          </div>
          <button type="button" className="small" onClick={() => resize({ w: o.src![2], h: o.src![3] })}>Размер объекта по прямоугольнику</button>
        </>}
        {tab === 'group' && o.members && (
          <div className="scroll list" style={{ maxHeight: 280, border: '1px solid var(--border)', borderRadius: 6 }}>
            {o.members.map(h => { const m = all?.find(x => x.handle === h); return (
              <div key={h} className="row" style={{ cursor: onOpen ? 'pointer' : undefined }} onDoubleClick={() => onOpen?.(h)} title="Двойной щелчок — свойства объекта">
                <span className="mono muted">#{h}</span><span>{m ? m.kind : 'объект'}</span>{m?.name && <span className="muted">{m.name}</span>}
              </div>
            ); })}
          </div>
        )}
        {tab === 'hyper' && <HyperFields o={o} set={set} />}
        {tab === 'vars' && vars && <VarsTab text={vars.text} vars={vars.classVars} onChange={vars.onChange} />}
        {tab === 'info' && (
          <table className="vars"><tbody>
            <tr><td className="muted">Тип</td><td>{o.kind}</td></tr>
            <tr><td className="muted">Дескриптор</td><td className="mono">{o.handle}</td></tr>
            <tr><td className="muted">Z-порядок</td><td className="mono">{o.zorder ?? '— (член группы)'}</td></tr>
            {o.parent !== null && <tr><td className="muted">Группа</td><td className="mono">#{o.parent}</td></tr>}
            <tr><td className="muted">Габарит</td><td className="mono">{o.x}, {o.y} · {o.w}×{o.h}</td></tr>
            {o.points && <tr><td className="muted">Точек</td><td className="mono">{o.points.length}</td></tr>}
            {o.members && <tr><td className="muted">Членов группы</td><td className="mono">{o.members.length}</td></tr>}
            {o.src && <tr><td className="muted">Исходный прямоугольник</td><td className="mono">{o.src.join(', ')}</td></tr>}
            {o.hyper && <tr><td className="muted">Гиперссылка</td><td>{o.hyper.target || '—'}</td></tr>}
          </tbody></table>
        )}
        {tab === 'points' && o.points && (
          <div className="scroll" style={{ maxHeight: 320 }}>
            <table className="vars"><tbody>
              {o.points.map((p, i) => <tr key={i}><td className="muted">{i + 1}</td><td className="num">{p[0]}</td><td className="num">{p[1]}</td></tr>)}
            </tbody></table>
            <div className="muted small">Точки правятся на листе: перетаскивание, Ctrl — добавить, Shift — удалить.</div>
          </div>
        )}
      </div>
    </Frame>
  );
}
