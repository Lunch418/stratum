// «Выбор имиджа» оригинала: список имиджей проекта и библиотек с фильтром;
// выбранный ставится на текущую схему справа от остальных блоков.
import { useMemo, useState } from 'react';
import { api } from '../api';
import { classByName, useStore } from '../store';

export function ChooseClassDialog({ onClose }: { onClose: () => void }) {
  const project = useStore(s => s.project);
  const path = useStore(s => s.schemePath);
  const [filter, setFilter] = useState('');
  const [picked, setPicked] = useState<string | null>(null);
  const scheme = classByName(project, path[path.length - 1]);
  const list = useMemo(() => {
    const f = filter.trim().toLowerCase();
    return (project?.classes ?? []).filter(c => !f || c.name.toLowerCase().includes(f) || c.description.toLowerCase().includes(f)).slice(0, 400);
  }, [project, filter]);
  const own = list.filter(c => !c.library), libs = list.filter(c => c.library);

  async function insert(name: string) {
    const s = useStore.getState();
    if (!scheme || scheme.library) { s.showToast('Откройте схему своего имиджа'); return; }
    const x = scheme.children.length ? Math.max(...scheme.children.map(c => c.x)) + 160 : 0;
    const y = scheme.children.length ? Math.min(...scheme.children.map(c => c.y)) : 0;
    try {
      await api.addChild(scheme.name, name, Math.round(x / 8) * 8, Math.round(y / 8) * 8);
      s.markUnsaved(); await s.reload(); s.setTab('scheme'); s.showToast(`${name} поставлен на схему ${scheme.name}`);
      onClose();
    } catch (e) { s.say({ level: 'error', where: 'схема', text: String(e) }); }
  }

  const row = (c: typeof list[number]) => (
    <div key={c.name} className={`row${picked === c.name ? ' selected' : ''}`} style={{ cursor: 'pointer', alignItems: 'center' }}
      onClick={() => setPicked(c.name)} onDoubleClick={() => insert(c.name)}>
      <img src={api.iconUrl(c.name)} width={20} height={20} alt="" />
      <span>{c.name}</span>
      {c.description && <span className="muted small" style={{ overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' }}>{c.description.split('\n')[0]}</span>}
    </div>
  );
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <form className="modal" style={{ width: 'min(560px, 94vw)' }} onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); if (picked) insert(picked); }}>
        <div className="panel-title">Выбор имиджа{scheme ? ` · на схему ${scheme.name}` : ''}</div>
        <div className="modal-body">
          <input autoFocus type="search" placeholder="имя или описание" value={filter} onChange={e => setFilter(e.target.value)} />
          <div className="scroll list" style={{ height: 340, border: '1px solid var(--border)', borderRadius: 6, marginTop: 8 }}>
            {own.length > 0 && <div className="muted small" style={{ padding: '6px 10px' }}>Имиджи проекта</div>}
            {own.map(row)}
            {libs.length > 0 && <div className="muted small" style={{ padding: '6px 10px' }}>Библиотеки</div>}
            {libs.map(row)}
            {!list.length && <div className="muted" style={{ padding: 10 }}>Ничего не найдено.</div>}
          </div>
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={onClose}>Отмена</button>
          <button type="submit" className="primary" disabled={!picked}>Вставить</button>
        </div>
      </form>
    </div>
  );
}
