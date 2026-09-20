// Строка меню в структуре оригинала (Файл, Правка, Вид, Вставка, Формат,
// Моделирование, Параметры, Помощь). Открытое меню переключается наведением,
// подменю раскрываются вправо, Escape закрывает.
import { useEffect, useState } from 'react';

export interface MenuItem {
  label?: string;
  hint?: string;
  run?: () => void;
  disabled?: boolean;
  checked?: boolean;
  sub?: MenuItem[];
  sep?: boolean;
  /// Причина недоступности — в подсказке; пункты оригинала, которым в новой
  /// среде нет места, помечаются так, а не пропадают молча.
  why?: string;
}
export interface Menu { title: string; items: MenuItem[] }

export function MenuBar({ menus }: { menus: Menu[] }) {
  const [open, setOpen] = useState<number | null>(null);
  useEffect(() => {
    if (open === null) return;
    const close = () => setOpen(null);
    const key = (e: KeyboardEvent) => { if (e.key === 'Escape') setOpen(null); };
    window.addEventListener('mousedown', close);
    window.addEventListener('keydown', key);
    return () => { window.removeEventListener('mousedown', close); window.removeEventListener('keydown', key); };
  }, [open]);
  return (
    <nav className="menubar" onMouseDown={e => e.stopPropagation()}>
      {menus.map((m, i) => (
        <div key={m.title} className={`menu-host${open === i ? ' open' : ''}`}>
          <button className="menu-title" onClick={() => setOpen(o => o === i ? null : i)} onMouseEnter={() => open !== null && setOpen(i)}>{m.title}</button>
          {open === i && <MenuList items={m.items} close={() => setOpen(null)} />}
        </div>
      ))}
    </nav>
  );
}

function MenuList({ items, close }: { items: MenuItem[]; close: () => void }) {
  const [sub, setSub] = useState<number | null>(null);
  return (
    <div className="context menu">
      {items.map((it, i) => it.sep ? <div key={i} className="menu-sep" /> : (
        <div key={i} className="menu-row" onMouseEnter={() => setSub(it.sub ? i : null)}>
          <button disabled={it.disabled} title={it.disabled ? it.why : undefined}
            onClick={() => { if (it.sub) { setSub(i); return; } if (it.run) { close(); it.run(); } }}>
            <span className="menu-check">{it.checked ? '✓' : ''}</span>
            <span className="menu-label">{it.label}</span>
            {it.hint && <span className="muted menu-hint">{it.hint}</span>}
            {it.sub && <span className="muted">›</span>}
          </button>
          {it.sub && sub === i && <div className="submenu"><MenuList items={it.sub} close={close} /></div>}
        </div>
      ))}
    </div>
  );
}
