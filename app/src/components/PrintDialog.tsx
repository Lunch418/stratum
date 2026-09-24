// «Печать» и «Параметры печати» (диалоги PRINT_MAIN, PRINT_BORDER и
// PRINT_SOURCE оригинала): что печатать, масштаб, поля листа, исходный
// прямоугольник. Принтер, число копий и диапазон страниц выбираются в
// системном окне печати браузера — его открывает кнопка «Печать».
import { useState } from 'react';
import { useStore } from '../store';
import { Check, Frame, Num, Radio, Tabs } from './Options';
import { loadPrint, modelDrawing, PAPER, pictureDrawing, printWith, schemeDrawing, type PrintOptions, type Source } from '../print';

export function PrintDialog({ onClose }: { onClose: () => void }) {
  const tab0 = useStore(s => s.tab);
  const frame = useStore(s => s.frame);
  const cls = useStore(s => s.selectedClass);
  const [tab, setTab] = useState<'main' | 'margins' | 'source'>('main');
  const [o, setO] = useState<PrintOptions>(() => {
    const saved = loadPrint();
    // по умолчанию — активная вкладка
    const source: Source = tab0 === 'code' ? 'code' : tab0 === 'model' ? 'model' : tab0 === 'picture' ? 'picture' : tab0 === 'icon' ? 'icon' : 'scheme';
    return { ...saved, source };
  });
  const set = (p: Partial<PrintOptions>) => setO(v => ({ ...v, ...p }));
  const save = () => { try { localStorage.setItem('print', JSON.stringify(o)); } catch { /* приватный режим */ } };
  const graphic = o.source !== 'code';
  const m = (i: number) => (v: number) => { const next = [...o.margins] as PrintOptions['margins']; next[i] = Math.max(0, v); set({ margins: next }); };
  const r = (i: number) => (v: number) => { const next = [...o.rect] as PrintOptions['rect']; next[i] = v; set({ rect: next }); };
  return (
    <Frame title="Печать" width={560} onClose={onClose} action="Печать…" onSubmit={() => { save(); onClose(); printWith(o); }}>
      <Tabs tabs={[['main', 'Основное'], ['margins', 'Поля'], ['source', 'Исходный прямоугольник']]} value={tab} onChange={setTab} />
      <div className="modal-body dialog-page">
        {tab === 'main' && <>
          <fieldset><legend>Что печатать</legend>
            <select value={o.source} onChange={e => set({ source: e.target.value as Source })}>
              <option value="scheme">Схема текущего имиджа</option>
              <option value="picture">Рисунок имиджа {cls ?? ''}</option>
              <option value="icon">Иконка имиджа {cls ?? ''}</option>
              <option value="model" disabled={!frame?.windows.length}>Окно модели{frame?.windows.length ? '' : ' (нет открытых окон)'}</option>
              <option value="code">Текст имиджа {cls ?? ''}</option>
            </select>
            {o.source === 'model' && (frame?.windows.length ?? 0) > 1 && (
              <select value={o.window} onChange={e => set({ window: Number(e.target.value) })}>
                {frame!.windows.map((w, i) => <option key={w.id} value={i}>{w.name}</option>)}
              </select>
            )}
          </fieldset>
          <div className="dialog-cols">
            <fieldset disabled={!graphic}><legend>Масштаб</legend>
              <Radio label="100% от натуральной величины" on={o.scale === 'natural'} onChange={() => set({ scale: 'natural' })} />
              <Radio label="Разместить на одной странице" on={o.scale === 'page'} onChange={() => set({ scale: 'page' })} />
              <Radio label="По ширине страницы" on={o.scale === 'width'} onChange={() => set({ scale: 'width' })} />
              <Radio label="Произвольный масштаб, %" on={o.scale === 'custom'} onChange={() => set({ scale: 'custom' })} />
              {o.scale === 'custom' && <input type="number" min={5} max={1000} value={o.percent} onChange={e => set({ percent: Math.max(5, Number(e.target.value)) })} />}
            </fieldset>
            <fieldset><legend>Бумага</legend>
              <select value={o.paper} onChange={e => set({ paper: e.target.value as PrintOptions['paper'] })}>
                {Object.keys(PAPER).map(p => <option key={p} value={p}>{p}</option>)}
              </select>
              <Radio label="Книжная" on={!o.landscape} onChange={() => set({ landscape: false })} />
              <Radio label="Альбомная" on={o.landscape} onChange={() => set({ landscape: true })} />
              <Check label="Заголовок: имидж, проект, дата" value={o.header} onChange={v => set({ header: v })} />
              {graphic ? <Check label="Чёрно-белая печать" value={o.mono} onChange={v => set({ mono: v })} />
                : <Check label="Номера строк" value={o.lineNumbers} onChange={v => set({ lineNumbers: v })} />}
            </fieldset>
          </div>
          <div className="muted small">Принтер, число копий и страницы выбираются в системном окне печати.</div>
        </>}
        {tab === 'margins' && <fieldset><legend>Поля, мм</legend>
          <div className="dialog-cols">
            <Num label="Левое" value={o.margins[0]} onChange={m(0)} />
            <Num label="Верхнее" value={o.margins[1]} onChange={m(1)} />
            <Num label="Правое" value={o.margins[2]} onChange={m(2)} />
            <Num label="Нижнее" value={o.margins[3]} onChange={m(3)} />
          </div>
        </fieldset>}
        {tab === 'source' && <fieldset disabled={!graphic}><legend>Исходный прямоугольник</legend>
          <Check label="Всё пространство" value={o.whole} onChange={v => set({ whole: v })} />
          <div className="dialog-cols">
            <Num label="Слева" value={o.rect[0]} onChange={r(0)} />
            <Num label="Сверху" value={o.rect[1]} onChange={r(1)} />
            <Num label="Справа" value={o.rect[2]} onChange={r(2)} />
            <Num label="Снизу" value={o.rect[3]} onChange={r(3)} />
          </div>
          <button type="button" className="small" disabled={o.whole} onClick={async () => {
            const d = o.source === 'scheme' ? schemeDrawing() : o.source === 'model' ? modelDrawing(o.window) : cls ? await pictureDrawing(cls, o.source === 'icon' ? 'icon' : 'image') : null;
            if (d) set({ rect: [Math.round(d.box[0]), Math.round(d.box[1]), Math.round(d.box[0] + d.box[2]), Math.round(d.box[1] + d.box[3])] });
          }}>Обновить прямоугольник</button>
          <div className="muted small">Координаты листа; «Обновить» берёт границы всего нарисованного.</div>
        </fieldset>}
      </div>
    </Frame>
  );
}
