// Панель отладки: условные точки останова и профиль последнего такта.
import { useEffect, useState } from 'react';
import { api, type Breakpoint } from '../api';
import { useStore } from '../store';

export function Debug() {
  const frame = useStore(s => s.frame);
  const selectedInstance = useStore(s => s.selectedInstance);
  const selectedClass = useStore(s => s.selectedClass);
  const instances = useStore(s => s.instances);
  const say = useStore(s => s.say);
  const [bps, setBps] = useState<Breakpoint[]>([]);
  const [profile, setProfile] = useState<{ tick: number; total: number; items: { index: number; path: string; class: string; ns: number }[] } | null>(null);
  const [expr, setExpr] = useState('');
  const [scope, setScope] = useState<'instance' | 'class'>('instance');

  const refresh = () => api.breakpoints().then(setBps).catch(() => {});
  useEffect(() => { refresh(); }, [frame?.halt?.message]);
  useEffect(() => {
    let alive = true;
    const poll = async () => {
      try { const p = await api.profile(); if (alive) setProfile(p); } catch { /* ядро недоступно */ }
      if (alive) setTimeout(poll, frame?.running ? 1000 : 3000);
    };
    poll();
    return () => { alive = false; };
  }, [frame?.running]);

  const inst = selectedInstance !== null ? instances.find(i => i.index === selectedInstance) : undefined;

  async function add() {
    const e = expr.trim();
    if (!e) return;
    const target = scope === 'instance' && inst ? { index: inst.index } : selectedClass ? { class: selectedClass } : null;
    if (!target) { say({ level: 'error', where: 'отладка', text: 'выберите экземпляр или имидж' }); return; }
    await api.breakpointAdd(target, e);
    setExpr('');
    refresh();
  }

  return (
    <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', minHeight: 0, flex: 1 }}>
      <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0, borderRight: '1px solid var(--border)' }}>
        <div className="panel-title">Точки останова <span className="spacer" /><span className="muted">{bps.length}</span></div>
        <form style={{ display: 'flex', gap: 6, padding: '6px 8px', borderBottom: '1px solid var(--border)' }} onSubmit={e => { e.preventDefault(); add(); }}>
          <select value={scope} onChange={e => setScope(e.target.value as 'instance' | 'class')} title="Где проверять условие">
            <option value="instance">{inst ? inst.path.split('\\').pop() : 'экземпляр'}</option>
            <option value="class">все {selectedClass ?? 'имидж'}</option>
          </select>
          <input type="text" className="mono" placeholder="условие, напр. x > 100" value={expr} onChange={e => setExpr(e.target.value)} style={{ flex: 1 }} />
          <button type="submit" className="small">Добавить</button>
        </form>
        <div className="scroll">
          {!bps.length && <div className="muted" style={{ padding: 10, fontSize: 12 }}>Модель встанет на паузу, когда условие станет истинным после такта.</div>}
          {bps.map(b => (
            <div key={b.id} className="tree-row" style={{ gap: 8 }}>
              <input type="checkbox" checked={b.enabled} onChange={() => api.breakpointToggle(b.id).then(refresh)} title="Включена" />
              <span className="mono" style={{ flex: 1 }}>{b.expr}</span>
              <span className="muted small" title={b.path}>{b.index !== null ? b.path.split('\\').pop() : `все ${b.class}`}</span>
              <button className="small ghost" onClick={() => api.breakpointRemove(b.id).then(refresh)} title="Убрать">×</button>
            </div>
          ))}
        </div>
      </div>
      <div style={{ display: 'flex', flexDirection: 'column', minHeight: 0 }}>
        <div className="panel-title">Профиль такта <span className="spacer" /><span className="muted mono">{profile ? `${(profile.total / 1e6).toFixed(2)} мс` : ''}</span></div>
        <div className="scroll">
          <table className="vars">
            <thead><tr><th>Экземпляр</th><th>Имидж</th><th style={{ textAlign: 'right' }}>мкс</th><th style={{ textAlign: 'right' }}>%</th></tr></thead>
            <tbody>
              {(profile?.items ?? []).slice(0, 25).map(it => (
                <tr key={it.index}>
                  <td title={it.path}>{it.path.split('\\').pop()}</td>
                  <td className="muted">{it.class}</td>
                  <td className="num">{(it.ns / 1000).toFixed(1)}</td>
                  <td className="num">{profile && profile.total ? Math.round(it.ns / profile.total * 100) : 0}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
