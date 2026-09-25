// Справка по функциям и темам (F1 в редакторе, поиск): темы из SC3.HLP,
// переведённые в markdown; ссылки между темами открываются здесь же.
import { useEffect, useState } from 'react';
import { api } from '../api';
import { useStore } from '../store';
import { Icon } from './Icon';

function escapeHtml(s: string) {
  return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

// достаточное подмножество markdown: заголовки, ссылки, код, списки, абзацы
function render(md: string): string {
  const lines = md.split('\n');
  const out: string[] = [];
  let para: string[] = [];
  let list = false;
  const inline = (t: string) => escapeHtml(t)
    .replace(/\[([^\]]+)\]\(([^)]+)\.md\)/g, (_m, text, topic) => `<a href="#help/${encodeURIComponent(topic)}" data-topic="${topic}">${text}</a>`)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<b>$1</b>');
  const flush = () => {
    if (para.length) { out.push(`<p>${para.map(inline).join('<br>')}</p>`); para = []; }
    if (list) { out.push('</ul>'); list = false; }
  };
  for (const raw of lines) {
    const line = raw.replace(/\s+$/, '');
    const h = line.match(/^(#{1,4})\s+(.*)/);
    if (h) { flush(); out.push(`<h${h[1].length + 1}>${inline(h[2])}</h${h[1].length + 1}>`); continue; }
    if (/^\s*[-*]\s+/.test(line)) { if (para.length) { out.push(`<p>${para.map(inline).join('<br>')}</p>`); para = []; } if (!list) { out.push('<ul>'); list = true; } out.push(`<li>${inline(line.replace(/^\s*[-*]\s+/, ''))}</li>`); continue; }
    if (!line.trim()) { flush(); continue; }
    if (list) { out.push('</ul>'); list = false; }
    para.push(line);
  }
  flush();
  return out.join('\n');
}

export function Help() {
  const topic = useStore(s => s.helpTopic);
  const setHelpTopic = useStore(s => s.setHelpTopic);
  const [query, setQuery] = useState('');
  const [hits, setHits] = useState<string[]>([]);
  const [page, setPage] = useState<{ topic: string; html: string } | { error: string } | null>(null);

  useEffect(() => {
    if (!topic) { setPage(null); return; }
    let alive = true;
    fetch(`/api/help/${encodeURIComponent(topic)}`).then(async r => {
      const j = await r.json();
      if (!alive) return;
      setPage(r.ok ? { topic: j.topic, html: render(j.markdown) } : { error: j.error ?? 'нет темы' });
    }).catch(e => alive && setPage({ error: String(e) }));
    return () => { alive = false; };
  }, [topic]);

  useEffect(() => {
    if (query.trim().length < 2) { setHits([]); return; }
    const id = setTimeout(() => api.helpSearch(query.trim()).then(setHits).catch(() => setHits([])), 150);
    return () => clearTimeout(id);
  }, [query]);

  return (
    <>
      <div className="panel-title">Справка <span className="spacer" />{topic && <button aria-label="Закрыть тему" title="Закрыть тему" className="small ghost icon-only" onClick={() => setHelpTopic(null)}><Icon name="close" /></button>}</div>
      <div className="filter"><input type="search" placeholder="Функция или тема (F1 в коде)" value={query} onChange={e => setQuery(e.target.value)} /></div>
      {hits.length > 0 && (
        <div className="tree" style={{ maxHeight: 160, overflow: 'auto', borderBottom: '1px solid var(--border)' }}>
          {hits.map(h => <div key={h} className="tree-row" onClick={() => { setHelpTopic(h); setQuery(''); setHits([]); }}>{h}</div>)}
        </div>
      )}
      <div className="scroll help-body" onClick={e => {
        const a = (e.target as HTMLElement).closest('a[data-topic]') as HTMLElement | null;
        if (a) { e.preventDefault(); setHelpTopic(a.dataset.topic!); }
      }}>
        {!page && <div className="muted" style={{ padding: 10, fontSize: 12 }}>Поставьте курсор на функцию в коде и нажмите F1, или введите название.</div>}
        {page && 'error' in page && <div className="muted" style={{ padding: 10 }}>{page.error}</div>}
        {page && 'html' in page && <div dangerouslySetInnerHTML={{ __html: page.html }} />}
      </div>
    </>
  );
}
