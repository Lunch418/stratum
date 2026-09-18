// Иерархия проекта: дерево экземпляров с фильтром; выбор синхронен со
// схемой и инспектором. Ниже — библиотечные имиджи.
import { useMemo, useState } from 'react';
import { DRAG_CLASS } from './SchemeCanvas';
import { api } from '../api';
import { useStore } from '../store';

export function Hierarchy() {
  const instances = useStore(s => s.instances);
  const project = useStore(s => s.project);
  const selectedInstance = useStore(s => s.selectedInstance);
  const select = useStore(s => s.select);
  const setTab = useStore(s => s.setTab);
  const [filter, setFilter] = useState('');
  const [collapsed, setCollapsed] = useState<Set<number>>(new Set());
  const [libOpen, setLibOpen] = useState(false);
  const [ownOpen, setOwnOpen] = useState(true);
  const reload = useStore(s => s.reload);
  const say = useStore(s => s.say);
  const own = project?.classes.filter(c => !c.library) ?? [];

  async function newClass() {
    // имя формируется автоматически; переименование — позже, через инспектор
    let n = 1;
    while (project?.classes.some(c => c.name.toLowerCase() === `имидж${n}`)) n++;
    try { await api.newClass(`Имидж${n}`); await reload(); select(`Имидж${n}`, null); setTab('code'); }
    catch (e) { say({ level: 'error', where: 'проект', text: String(e) }); }
  }

  const children = useMemo(() => {
    const m = new Map<number | null, typeof instances>();
    for (const i of instances) { const list = m.get(i.parent) ?? []; list.push(i); m.set(i.parent, list); }
    return m;
  }, [instances]);

  const rows: { inst: (typeof instances)[0]; depth: number }[] = [];
  const f = filter.toLowerCase();
  function walk(parent: number | null, depth: number) {
    for (const i of children.get(parent) ?? []) {
      const kids = children.get(i.index) ?? [];
      const matches = !f || i.name.toLowerCase().includes(f) || i.class.toLowerCase().includes(f);
      if (matches || kids.length) rows.push({ inst: i, depth });
      if (!collapsed.has(i.index) || f) walk(i.index, depth + 1);
    }
  }
  walk(null, 0);

  const libs = project?.classes.filter(c => c.library) ?? [];
  const groups = new Map<string, typeof libs>();
  for (const c of libs) {
    const m = c.source.replace(/\\/g, '/').match(/\/([^/]+\.(?:LIB|SC|lib|sc))\//);
    const g = m ? m[1] : 'библиотека';
    (groups.get(g) ?? groups.set(g, []).get(g)!).push(c);
  }

  return (
    <>
      <div className="panel-title">Иерархия проекта <span className="spacer" /><span className="muted">{instances.length}</span></div>
      <div className="filter"><input type="search" placeholder="Фильтр по имени" value={filter} onChange={e => setFilter(e.target.value)} /></div>
      <div className="scroll">
        <div className="tree">
          {rows.map(({ inst, depth }) => {
            const kids = children.get(inst.index)?.length ?? 0;
            const isCollapsed = collapsed.has(inst.index);
            return (
              <div key={inst.index} className={`tree-row${selectedInstance === inst.index ? ' selected' : ''}`} style={{ paddingLeft: 8 + depth * 14 }}
                onClick={() => select(inst.class, inst.index)}
                onDoubleClick={() => setTab('code')}
                title={inst.path}>
                <span className="twisty" onClick={e => { e.stopPropagation(); setCollapsed(s => { const n = new Set(s); n.has(inst.index) ? n.delete(inst.index) : n.add(inst.index); return n; }); }}>
                  {kids ? (isCollapsed ? '▸' : '▾') : ''}
                </span>
                <img src={api.iconUrl(inst.class)} alt="" />
                <span className="name">{inst.name}{inst.name !== inst.class && <span className="muted"> · {inst.class}</span>}</span>
                {kids > 0 && <span className="count">{kids}</span>}
              </div>
            );
          })}
        </div>
        <div className="panel-title" style={{ cursor: 'pointer' }} onClick={() => setOwnOpen(o => !o)}>
          Имиджи проекта <span className="spacer" />
          <button className="small ghost" onClick={e => { e.stopPropagation(); newClass(); }} title="Создать пустой имидж">+ новый</button>
          <span className="muted">{own.length}</span>
        </div>
        {ownOpen && (
          <div className="tree">
            {own.filter(c => !f || c.name.toLowerCase().includes(f)).map(c => (
              <div key={c.name} className="tree-row" style={{ paddingLeft: 22 }} draggable
                onDragStart={e => { e.dataTransfer.setData(DRAG_CLASS, c.name); e.dataTransfer.effectAllowed = 'copy'; }}
                onClick={() => select(c.name, null)} onDoubleClick={() => setTab('code')} title="Перетащите на схему, чтобы добавить экземпляр">
                <img src={api.iconUrl(c.name)} alt="" /><span className="name">{c.name}</span>
                {c.children.length > 0 && <span className="count">{c.children.length}</span>}
              </div>
            ))}
          </div>
        )}
        <div className="panel-title" style={{ cursor: 'pointer' }} onClick={() => setLibOpen(o => !o)}>
          Библиотеки <span className="spacer" /><span className="muted">{libs.length}</span>
        </div>
        {libOpen && [...groups.entries()].map(([g, list]) => (
          <div key={g} className="tree">
            <div className="tree-row muted" style={{ paddingLeft: 8 }}>{g}</div>
            {list.filter(c => !f || c.name.toLowerCase().includes(f)).map(c => (
              <div key={c.name} className="tree-row" style={{ paddingLeft: 22 }} draggable
                onDragStart={e => { e.dataTransfer.setData(DRAG_CLASS, c.name); e.dataTransfer.effectAllowed = 'copy'; }}
                onClick={() => select(c.name, null)} onDoubleClick={() => setTab('code')} title={c.description || 'Перетащите на схему'}>
                <img src={api.iconUrl(c.name)} alt="" /><span className="name">{c.name}</span>
              </div>
            ))}
          </div>
        ))}
      </div>
    </>
  );
}
