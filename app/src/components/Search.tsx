// Поиск по всему проекту (меню «Правка → Поиск» оригинала): тексты,
// переменные и имена имиджей; щелчок открывает код на нужной строке.
import { useEffect, useState } from 'react';
import { useStore } from '../store';

interface Hit { class: string; line: number; text: string; kind: 'text' | 'var' | 'class' }

export function Search() {
  const select = useStore(s => s.select);
  const setTab = useStore(s => s.setTab);
  const setGotoLine = useStore(s => s.setGotoLine);
  const query = useStore(s => s.searchQuery);
  const setQuery = useStore(s => s.setSearchQuery);
  const [libs, setLibs] = useState(false);
  const [hits, setHits] = useState<Hit[]>([]);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (query.trim().length < 2) { setHits([]); return; }
    setBusy(true);
    const id = setTimeout(() => {
      fetch(`/api/search?q=${encodeURIComponent(query.trim())}&libs=${libs ? 1 : 0}`).then(r => r.json()).then(h => { setHits(h); setBusy(false); }).catch(() => setBusy(false));
    }, 200);
    return () => clearTimeout(id);
  }, [query, libs]);

  return (
    <>
      <div className="panel-title">Поиск
        <input type="search" autoFocus placeholder="текст, переменная или имидж" value={query} onChange={e => setQuery(e.target.value)} style={{ marginLeft: 8, width: 260, textTransform: 'none', letterSpacing: 0, fontSize: 12 }} />
        <label style={{ textTransform: 'none', letterSpacing: 0, display: 'flex', gap: 4, alignItems: 'center' }}><input type="checkbox" checked={libs} onChange={e => setLibs(e.target.checked)} />и в библиотеках</label>
        <span className="spacer" /><span className="muted">{busy ? '…' : hits.length}</span>
      </div>
      <div className="scroll messages">
        {hits.map((h, i) => (
          <div key={i} className="row" style={{ cursor: 'pointer' }} onClick={() => { select(h.class); setTab('code'); if (h.line) setGotoLine({ class: h.class, line: h.line }); }}>
            <span className="where">{h.class}{h.line ? `:${h.line}` : ''}</span>
            <span className={h.kind === 'text' ? '' : 'muted'}>{h.text}</span>
          </div>
        ))}
        {!hits.length && query.trim().length >= 2 && !busy && <div className="muted" style={{ padding: 10 }}>Ничего не найдено.</div>}
      </div>
    </>
  );
}
