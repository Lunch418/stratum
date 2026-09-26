// Открытие проекта: обзор папок через ядро (/api/browse), недавние проекты,
// ввод пути. Папка с project.spj или project.json помечается как проект.
import { useEffect, useState } from 'react';
import { api } from '../api';
import { useStore } from '../store';
import { Icon } from './Icon';

interface Entry { name: string; path: string; kind: 'dir' | 'project' | 'file' }

function loadRecent(): string[] {
  try { return JSON.parse(localStorage.getItem('recent') ?? '[]'); } catch { return []; }
}
export function rememberRecent(path: string) {
  try {
    const list = [path, ...loadRecent().filter(p => p !== path)].slice(0, 10);
    localStorage.setItem('recent', JSON.stringify(list));
  } catch { /* приватный режим */ }
}

export function OpenDialog({ onClose, required }: { onClose: () => void; required: boolean }) {
  const openProject = useStore(s => s.openProject);
  const say = useStore(s => s.say);
  const [dir, setDir] = useState<string>(() => { try { return localStorage.getItem('browseDir') ?? ''; } catch { return ''; } });
  const [listing, setListing] = useState<{ dir: string; parent: string | null; entries: Entry[] } | null>(null);
  const [path, setPath] = useState('');
  const [busy, setBusy] = useState(false);
  const recent = loadRecent();

  useEffect(() => {
    let alive = true;
    api.browse(dir).then(l => { if (alive) { setListing(l); try { localStorage.setItem('browseDir', l.dir); } catch { /* */ } } }).catch(() => alive && setListing(null));
    return () => { alive = false; };
  }, [dir]);

  async function open(p: string) {
    if (!p.trim()) return;
    setBusy(true);
    try { await openProject(p.trim()); rememberRecent(p.trim()); onClose(); }
    catch (e) { say({ level: 'error', where: 'открытие', text: String(e) }); }
    finally { setBusy(false); }
  }

  return (
    <div className="modal-backdrop" onMouseDown={() => !required && onClose()}>
      <form className="modal open-dialog" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); open(path || listing?.dir || ''); }}>
        <div className="panel-title">Открыть проект</div>
        <div className="modal-body" style={{ display: 'grid', gridTemplateColumns: '200px 1fr', gap: 12 }}>
          <div>
            <div className="panel-title" style={{ padding: '0 0 4px', border: 0 }}>Недавние</div>
            {!recent.length && <div className="muted small">Открытые проекты появятся здесь.</div>}
            {recent.map(p => (
              <div key={p} className="tree-row" title={p} onClick={() => open(p)} style={{ padding: '3px 4px' }}>
                <span className="name">{p.split('/').filter(Boolean).pop()}</span>
              </div>
            ))}
            <div className="muted small" style={{ marginTop: 10 }}>Проект Stratum 2000 — папка с <span className="mono">project.spj</span>; проект Stratum Modern — с <span className="mono">project.json</span>.</div>
          </div>
          <div style={{ minWidth: 0 }}>
            <div style={{ display: 'flex', gap: 6, alignItems: 'center', marginBottom: 6 }}>
              <button type="button" className="small icon-only" onClick={() => listing?.parent && setDir(listing.parent)} disabled={!listing?.parent} title="На уровень выше" aria-label="На уровень выше"><Icon name="up" /></button>
              <span className="mono small" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap', flex: 1 }} title={listing?.dir}>{listing?.dir ?? '…'}</span>
            </div>
            <div className="tree browse">
              {listing?.entries.map(e => (
                <div key={e.path} className={`tree-row ${e.kind}`} title={e.path}
                  onClick={() => e.kind === 'dir' ? setDir(e.path) : setPath(e.path)}
                  onDoubleClick={() => e.kind === 'dir' ? setDir(e.path) : open(e.path)}>
                  <Icon className="kind" name={e.kind === 'dir' ? 'folder' : e.kind === 'project' ? 'project' : 'file'} />
                  <span className="name">{e.name}</span>
                  {e.kind === 'project' && <button type="button" className="small ghost" onClick={ev => { ev.stopPropagation(); open(e.path); }}>Открыть</button>}
                </div>
              ))}
              {listing && !listing.entries.length && <div className="muted" style={{ padding: 8 }}>Пусто</div>}
            </div>
            <input type="text" className="mono" placeholder="или путь к папке проекта / .spj / project.json" value={path} onChange={e => setPath(e.target.value)} style={{ width: '100%', marginTop: 6 }} />
          </div>
        </div>
        <div className="modal-actions">
          {!required && <button type="button" className="ghost" onClick={onClose}>Отмена</button>}
          <button type="submit" className="primary" disabled={busy}>{busy ? 'Открываю…' : 'Открыть'}</button>
        </div>
      </form>
    </div>
  );
}
