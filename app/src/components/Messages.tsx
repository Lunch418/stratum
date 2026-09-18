// Панель сообщений: компиляция, LogMessage, ошибки времени выполнения.
import { useStore } from '../store';

export function Messages() {
  const messages = useStore(s => s.messages);
  const frame = useStore(s => s.frame);
  const setTab = useStore(s => s.setTab);
  const log = frame?.log ?? [];
  return (
    <>
      <div className="panel-title">Сообщения <span className="spacer" /><span className="muted">{messages.length + log.length}</span></div>
      <div className="scroll messages">
        {log.map((l, i) => <div key={'l' + i} className={`row${l.startsWith('ошибка') ? ' error' : ''}`}><span className="where">модель</span><span>{l}</span></div>)}
        {messages.slice().reverse().map((m, i) => (
          <div key={i} className={`row ${m.level}`} onClick={() => m.level === 'error' && setTab('code')} style={{ cursor: m.level === 'error' ? 'pointer' : 'default' }}>
            <span className="where">{m.where}</span><span>{m.text}</span>
          </div>
        ))}
      </div>
    </>
  );
}
