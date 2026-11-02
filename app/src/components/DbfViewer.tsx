// Просмотр таблиц dBASE (.dbf) из папки проекта: страницы, заголовки
// столбцов, поиск. Только чтение.
import { useEffect, useState } from 'react';
import { api, type DbfPage } from '../api';

const PAGE = 100;

export function DbfViewer({ onClose }: { onClose: () => void }) {
  const [files, setFiles] = useState<string[]>([]);
  const [file, setFile] = useState('');
  const [query, setQuery] = useState('');
  const [offset, setOffset] = useState(0);
  const [page, setPage] = useState<DbfPage | null>(null);
  const [error, setError] = useState('');

  useEffect(() => {
    api.dbfFiles().then(f => { setFiles(f); if (f[0]) setFile(f[0]); }).catch(() => setError('ядро недоступно'));
  }, []);
  useEffect(() => {
    if (!file) return;
    let alive = true;
    api.dbf(file, offset, PAGE, query)
      .then(p => { if (alive) { setPage(p); setError(''); } })
      .catch(e => { if (alive) { setPage(null); setError(String(e.message ?? e)); } });
    return () => { alive = false; };
  }, [file, offset, query]);

  const matched = page?.matched ?? 0;
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div className="modal" style={{ width: 'min(900px, 94vw)' }} onMouseDown={e => e.stopPropagation()} onKeyDown={e => { if (e.key === 'Escape') onClose(); }}>
        <div className="panel-title">Таблицы DBF (только чтение)</div>
        <div className="modal-body">
          {files.length === 0 ? <div className="muted">В папке проекта нет файлов .dbf</div> : (
            <div style={{ display: 'flex', gap: 8, marginBottom: 8 }}>
              <select value={file} onChange={e => { setFile(e.target.value); setOffset(0); }}>
                {files.map(f => <option key={f} value={f}>{f}</option>)}
              </select>
              <input type="search" placeholder="Поиск" value={query} onChange={e => { setQuery(e.target.value); setOffset(0); }} style={{ flex: 1 }} />
            </div>
          )}
          {error && <div className="muted">{error}</div>}
          {page && (
            <div style={{ overflow: 'auto', maxHeight: '60vh' }}>
              <table className="mono small">
                <thead><tr><th>#</th>{page.fields.map(f => <th key={f.name} title={`${f.kind}(${f.len})`}>{f.name}</th>)}</tr></thead>
                <tbody>
                  {page.rows.map(r => (
                    <tr key={r.n} style={r.deleted ? { opacity: 0.5, textDecoration: 'line-through' } : undefined}>
                      <td>{r.n}</td>{r.cells.map((c, i) => <td key={i}>{c}</td>)}
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </div>
        <div className="modal-actions">
          {page && <span className="muted small" style={{ marginRight: 'auto' }}>
            {matched === 0 ? 'Нет записей' : `${offset + 1}-${Math.min(offset + PAGE, matched)} из ${matched}`} (всего {page.total}), кодировка {page.codepage}
          </span>}
          <button type="button" className="ghost" disabled={offset === 0} onClick={() => setOffset(Math.max(0, offset - PAGE))}>Назад</button>
          <button type="button" className="ghost" disabled={offset + PAGE >= matched} onClick={() => setOffset(offset + PAGE)}>Вперёд</button>
          <button type="button" className="primary" onClick={onClose}>Закрыть</button>
        </div>
      </div>
    </div>
  );
}
