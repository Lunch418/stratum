// «Выбор масштаба» (диалог «Масштаб» оригинала) — список масштабов у холста.
const PRESETS = [10, 25, 50, 75, 100, 150, 200, 300, 400, 800];

export function ZoomSelect({ k, onSet, onFit }: { k: number; onSet: (k: number) => void; onFit?: () => void }) {
  const cur = Math.round(k * 100);
  return (
    <select className="small mono zoom-select" value={PRESETS.includes(cur) ? cur : 'cur'} title="Масштаб"
      onChange={e => { const v = e.target.value; if (v === 'fit') onFit?.(); else if (v !== 'cur') onSet(Number(v) / 100); }}>
      {!PRESETS.includes(cur) && <option value="cur">{cur}%</option>}
      {PRESETS.map(p => <option key={p} value={p}>{p}%</option>)}
      {onFit && <option value="fit">Показать всё</option>}
    </select>
  );
}
