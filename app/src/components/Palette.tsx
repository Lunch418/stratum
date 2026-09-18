// Командная палитра (Ctrl+Shift+P / Ctrl+K): команды IDE и переход к
// имиджам по нечёткому поиску.
import { useEffect, useMemo, useRef, useState } from 'react';

export interface Command {
  id: string;
  title: string;
  hint?: string;
  group?: string;
  run: () => void;
}

interface Props {
  commands: Command[];
  onClose: () => void;
}

function score(query: string, text: string): number {
  // подпоследовательность с бонусом за начало слова и подряд идущие символы
  const q = query.toLowerCase(), t = text.toLowerCase();
  if (!q) return 1;
  let ti = 0, s = 0, streak = 0;
  for (const ch of q) {
    const at = t.indexOf(ch, ti);
    if (at < 0) return 0;
    streak = at === ti ? streak + 1 : 0;
    s += 1 + streak + (at === 0 || /[\s._\-/]/.test(t[at - 1]) ? 2 : 0);
    ti = at + 1;
  }
  return s / (1 + t.length / 40);
}

export function Palette({ commands, onClose }: Props) {
  const [query, setQuery] = useState('');
  const [cursor, setCursor] = useState(0);
  const input = useRef<HTMLInputElement>(null);
  const list = useRef<HTMLDivElement>(null);
  useEffect(() => { input.current?.focus(); }, []);

  const items = useMemo(() => {
    const scored = commands.map(c => ({ c, s: score(query, c.title + ' ' + (c.hint ?? '')) })).filter(x => x.s > 0);
    scored.sort((a, b) => b.s - a.s);
    return scored.slice(0, 40).map(x => x.c);
  }, [query, commands]);

  useEffect(() => { setCursor(0); }, [query]);
  useEffect(() => {
    list.current?.querySelector('.active')?.scrollIntoView({ block: 'nearest' });
  }, [cursor]);

  function onKey(e: React.KeyboardEvent) {
    if (e.key === 'ArrowDown') { e.preventDefault(); setCursor(c => Math.min(items.length - 1, c + 1)); }
    else if (e.key === 'ArrowUp') { e.preventDefault(); setCursor(c => Math.max(0, c - 1)); }
    else if (e.key === 'Enter') { e.preventDefault(); const it = items[cursor]; if (it) { onClose(); it.run(); } }
    else if (e.key === 'Escape') { e.preventDefault(); onClose(); }
  }

  return (
    <div className="modal-backdrop palette-backdrop" onMouseDown={onClose}>
      <div className="modal palette" onMouseDown={e => e.stopPropagation()}>
        <input ref={input} type="text" placeholder="Команда или имя имиджа…" value={query} onChange={e => setQuery(e.target.value)} onKeyDown={onKey} spellCheck={false} />
        <div className="list" ref={list}>
          {items.map((c, i) => (
            <div key={c.id} className={`row${i === cursor ? ' active' : ''}`} onMouseEnter={() => setCursor(i)} onClick={() => { onClose(); c.run(); }}>
              {c.group && <span className="group muted">{c.group}</span>}
              <span className="title">{c.title}</span>
              {c.hint && <span className="hint mono muted">{c.hint}</span>}
            </div>
          ))}
          {!items.length && <div className="row muted">Ничего не найдено</div>}
        </div>
      </div>
    </div>
  );
}
