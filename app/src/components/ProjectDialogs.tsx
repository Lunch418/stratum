// Диалоги проекта: «Создание проекта» (папка и корневой имидж) и
// «Информация» (сводка: имиджи, экземпляры, связи, уравнения, функции).
import { useEffect, useState } from 'react';
import { useStore } from '../store';

export function NewProjectDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const load = useStore(s => s.load);
  const say = useStore(s => s.say);
  const showToast = useStore(s => s.showToast);
  const [dir, setDir] = useState(() => (project?.dir || '').replace(/[\\/]+[^\\/]*$/, '') + '/новый_проект');
  const [root, setRoot] = useState('Main');
  const [inMemory, setInMemory] = useState(false);
  const [busy, setBusy] = useState(false);

  async function create() {
    setBusy(true);
    try {
      const r = await fetch(`/api/new?root=${encodeURIComponent(root.trim() || 'Main')}${inMemory ? '' : '&dir=' + encodeURIComponent(dir.trim())}`, { method: 'POST' });
      const j = await r.json();
      if (!r.ok) throw new Error(j.error ?? r.statusText);
      await load();
      showToast('Проект создан');
      onClose();
    } catch (e) { say({ level: 'error', where: 'новый проект', text: String(e) }); }
    finally { setBusy(false); }
  }

  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <form className="modal" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); create(); }}>
        <div className="panel-title">Создание проекта</div>
        <div className="modal-body" style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          <label className="prop"><span>Корневой имидж</span><input autoFocus type="text" value={root} onChange={e => setRoot(e.target.value)} /></label>
          <label className="prop"><span>Папка</span><input type="text" className="mono" value={dir} onChange={e => setDir(e.target.value)} disabled={inMemory} /></label>
          <label style={{ display: 'flex', gap: 6, alignItems: 'center' }}><input type="checkbox" checked={inMemory} onChange={e => setInMemory(e.target.checked)} />только в памяти (папку задать при сохранении)</label>
          <div className="muted small">Проект сохраняется в родном формате: <span className="mono">project.json</span> и папка <span className="mono">classes/</span>; экспорт в <span className="mono">.spj/.cls</span> — из меню «Проект».</div>
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={onClose}>Отмена</button>
          <button type="submit" className="primary" disabled={busy}>Создать</button>
        </div>
      </form>
    </div>
  );
}

interface Info {
  root: string; dir: string; classes: number; libraryClasses: number; instances: number; children: number; links: number;
  equations: number; lines: number; matrices: number; windows: number; functions: [string, number][]; properties: [string, string][]; libraries: string[];
}

export function InfoDialog({ onClose }: { onClose: () => void }) {
  const [info, setInfo] = useState<Info | null>(null);
  const setHelpTopic = useStore(s => s.setHelpTopic);
  useEffect(() => { fetch('/api/info').then(r => r.json()).then(setInfo).catch(() => setInfo(null)); }, []);
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div className="modal" style={{ width: 'min(760px, 94vw)' }} onMouseDown={e => e.stopPropagation()}>
        <div className="panel-title">Информация о проекте <span className="spacer" /><button className="small ghost" onClick={onClose}>×</button></div>
        {!info ? <div className="muted" style={{ padding: 12 }}>Загрузка…</div> : (
          <div className="modal-body" style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 16, maxHeight: '70vh', overflow: 'auto', fontSize: 12 }}>
            <div>
              <table className="vars"><tbody>
                <tr><td className="muted">Корневой имидж</td><td>{info.root}</td></tr>
                <tr><td className="muted">Папка</td><td className="mono">{info.dir || '— (в памяти)'}</td></tr>
                <tr><td className="muted">Имиджей проекта</td><td className="num">{info.classes}</td></tr>
                <tr><td className="muted">Имиджей библиотек</td><td className="num">{info.libraryClasses}</td></tr>
                <tr><td className="muted">Экземпляров в модели</td><td className="num">{info.instances}</td></tr>
                <tr><td className="muted">Блоков на схемах</td><td className="num">{info.children}</td></tr>
                <tr><td className="muted">Связей</td><td className="num">{info.links}</td></tr>
                <tr><td className="muted">Уравнений</td><td className="num">{info.equations}</td></tr>
                <tr><td className="muted">Строк кода</td><td className="num">{info.lines}</td></tr>
                <tr><td className="muted">Матриц</td><td className="num">{info.matrices}</td></tr>
                <tr><td className="muted">Окон модели</td><td className="num">{info.windows}</td></tr>
              </tbody></table>
              {info.properties.length > 0 && <>
                <div className="panel-title" style={{ padding: '8px 0 4px', border: 0 }}>Свойства проекта</div>
                <table className="vars"><tbody>{info.properties.map(([k, v]) => <tr key={k}><td className="muted">{k}</td><td className="mono">{v}</td></tr>)}</tbody></table>
              </>}
              <div className="panel-title" style={{ padding: '8px 0 4px', border: 0 }}>Библиотеки</div>
              {info.libraries.map(l => <div key={l} className="mono muted" style={{ fontSize: 11 }}>{l}</div>)}
            </div>
            <div>
              <div className="panel-title" style={{ padding: '0 0 4px', border: 0 }}>Функции ({info.functions.length})</div>
              <table className="vars"><tbody>
                {info.functions.map(([n, c]) => <tr key={n} style={{ cursor: 'pointer' }} onClick={() => setHelpTopic(n)}><td>{n}</td><td className="num">{c}</td></tr>)}
              </tbody></table>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
