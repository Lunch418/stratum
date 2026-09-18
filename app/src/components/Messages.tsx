// Панель сообщений: компиляция, LogMessage, ошибки времени выполнения.
import { useState } from 'react';
import { useStore } from '../store';

export function Messages() {
  const messages = useStore(s => s.messages);
  const frame = useStore(s => s.frame);
  const setTab = useStore(s => s.setTab);
  const [filter, setFilter] = useState('');
  const [onlyErrors, setOnlyErrors] = useState(false);
  const f = filter.toLowerCase();
  const log = (frame?.log ?? []).filter(l => (!f || l.toLowerCase().includes(f)) && (!onlyErrors || l.startsWith('ошибка')));
  const own = messages.filter(m => (!f || (m.where + ' ' + m.text).toLowerCase().includes(f)) && (!onlyErrors || m.level === 'error'));
  return (
    <>
      <div className="panel-title">Сообщения
        <input type="search" placeholder="фильтр" value={filter} onChange={e => setFilter(e.target.value)} style={{ marginLeft: 8, width: 160, textTransform: 'none', letterSpacing: 0, fontSize: 12 }} />
        <label style={{ textTransform: 'none', letterSpacing: 0, display: 'flex', gap: 4, alignItems: 'center' }}><input type="checkbox" checked={onlyErrors} onChange={e => setOnlyErrors(e.target.checked)} />только ошибки</label>
        <span className="spacer" /><span className="muted">{own.length + log.length}</span>
      </div>
      <div className="scroll messages">
        {log.map((l, i) => <div key={'l' + i} className={`row${l.startsWith('ошибка') ? ' error' : ''}`}><span className="where">модель</span><span>{l}</span></div>)}
        {own.slice().reverse().map((m, i) => (
          <div key={i} className={`row ${m.level}`} onClick={() => m.level === 'error' && setTab('code')} style={{ cursor: m.level === 'error' ? 'pointer' : 'default' }}>
            <span className="where">{m.where}</span><span>{m.text}</span>
          </div>
        ))}
      </div>
    </>
  );
}
