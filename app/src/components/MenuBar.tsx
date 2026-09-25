// Строка меню в структуре оригинала (Файл, Правка, Вид, Вставка, Формат,
// Моделирование, Параметры, Помощь). Открытое меню переключается наведением,
// подменю раскрываются вправо, Escape закрывает. С клавиатуры: стрелки
// ходят по заголовкам и пунктам, Enter/пробел выбирают, → и ← открывают
// и закрывают подменю, Home/End — к первому и последнему пункту.
import { useEffect, useRef, useState } from 'react';

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

/// пункты одного уровня списка (без пунктов вложенных подменю)
const ownItems = (list: HTMLElement) =>
  [...list.querySelectorAll<HTMLButtonElement>(':scope > .menu-row > button')].filter(b => !b.disabled);

function moveFocus(list: HTMLElement, from: Element | null, step: number | 'first' | 'last') {
  const items = ownItems(list);
  if (!items.length) return;
  const i = items.indexOf(from as HTMLButtonElement);
  const next = step === 'first' ? 0 : step === 'last' ? items.length - 1 : (i + step + items.length) % items.length;
  items[next].focus();
}

export function MenuBar({ menus }: { menus: Menu[] }) {
  const [open, setOpen] = useState<number | null>(null);
  // меню, открытое с клавиатуры, сразу получает фокус на первом пункте
  const [kbd, setKbd] = useState(false);
  const titles = useRef<(HTMLButtonElement | null)[]>([]);
  useEffect(() => {
    if (open === null) return;
    const close = () => setOpen(null);
    const key = (e: KeyboardEvent) => { if (e.key === 'Escape') { setOpen(null); titles.current[open]?.focus(); } };
    window.addEventListener('mousedown', close);
    window.addEventListener('keydown', key);
    return () => { window.removeEventListener('mousedown', close); window.removeEventListener('keydown', key); };
  }, [open]);
  const shift = (i: number, d: number, keepOpen: boolean) => {
    const n = (i + d + menus.length) % menus.length;
    titles.current[n]?.focus();
    if (keepOpen) { setKbd(true); setOpen(n); }
  };
  return (
    <nav className="menubar" role="menubar" aria-label="Главное меню" onMouseDown={e => e.stopPropagation()}>
      {menus.map((m, i) => (
        <div key={m.title} className={`menu-host${open === i ? ' open' : ''}`}>
          <button ref={el => { titles.current[i] = el; }} className="menu-title" role="menuitem" aria-haspopup="menu" aria-expanded={open === i}
            tabIndex={i === 0 || open === i ? 0 : -1}
            onClick={e => { setKbd(e.detail === 0); setOpen(o => o === i ? null : i); }}
            onMouseEnter={() => open !== null && setOpen(i)}
            onKeyDown={e => {
              if (e.key === 'ArrowRight' || e.key === 'ArrowLeft') { e.preventDefault(); shift(i, e.key === 'ArrowRight' ? 1 : -1, open !== null); }
              else if (e.key === 'ArrowDown' || e.key === 'ArrowUp') { e.preventDefault(); setKbd(true); setOpen(i); }
            }}>{m.title}</button>
          {open === i && (
            <MenuList items={m.items} autoFocus={kbd} label={m.title}
              close={(refocus) => { setOpen(null); if (refocus) titles.current[i]?.focus(); }}
              side={d => shift(i, d, true)} />
          )}
        </div>
      ))}
    </nav>
  );
}

function MenuList({ items, close, autoFocus, label, side, back }: {
  items: MenuItem[];
  close: (refocus: boolean) => void;
  autoFocus: boolean;
  label: string;
  /// ← и → на верхнем уровне: к соседнему меню строки
  side?: (d: number) => void;
  /// ← в подменю: вернуться к пункту-родителю
  back?: () => void;
}) {
  const [sub, setSub] = useState<number | null>(null);
  const [subKbd, setSubKbd] = useState(false);
  const ref = useRef<HTMLDivElement>(null);
  useEffect(() => { if (autoFocus && ref.current) moveFocus(ref.current, null, 'first'); }, [autoFocus]);
  const rowButton = (i: number) => ref.current?.querySelector<HTMLButtonElement>(`:scope > .menu-row[data-i="${i}"] > button`);
  return (
    <div ref={ref} className="context menu" role="menu" aria-label={label}
      onKeyDown={e => {
        // событие из вложенного подменю обрабатывает само подменю
        if ((e.target as HTMLElement).closest('.context.menu') !== ref.current) return;
        const cur = document.activeElement;
        const i = Number((cur?.parentElement as HTMLElement | null)?.dataset.i);
        if (e.key === 'ArrowDown') { e.preventDefault(); moveFocus(ref.current!, cur, 1); }
        else if (e.key === 'ArrowUp') { e.preventDefault(); moveFocus(ref.current!, cur, -1); }
        else if (e.key === 'Home') { e.preventDefault(); moveFocus(ref.current!, cur, 'first'); }
        else if (e.key === 'End') { e.preventDefault(); moveFocus(ref.current!, cur, 'last'); }
        else if (e.key === 'ArrowRight') {
          e.preventDefault();
          if (items[i]?.sub && !items[i].disabled) { setSubKbd(true); setSub(i); } else side?.(1);
        }
        else if (e.key === 'ArrowLeft') { e.preventDefault(); if (back) back(); else side?.(-1); }
        else if (e.key === 'Tab') { close(false); }
        else if (e.key === 'Escape' && back) { e.preventDefault(); e.stopPropagation(); back(); }
      }}>
      {items.map((it, i) => it.sep ? <div key={i} className="menu-sep" role="separator" /> : (
        <div key={i} data-i={i} className="menu-row" onMouseEnter={() => { setSubKbd(false); setSub(it.sub ? i : null); }}>
          <button type="button" tabIndex={-1} disabled={it.disabled} title={it.disabled ? it.why : undefined}
            role={it.checked !== undefined ? 'menuitemcheckbox' : 'menuitem'} aria-checked={it.checked !== undefined ? !!it.checked : undefined}
            aria-haspopup={it.sub ? 'menu' : undefined} aria-expanded={it.sub ? sub === i : undefined}
            onClick={e => { if (it.sub) { setSubKbd(e.detail === 0); setSub(i); return; } if (it.run) { close(e.detail === 0); it.run(); } }}>
            <span className="menu-check" aria-hidden="true">{it.checked ? '✓' : ''}</span>
            <span className="menu-label">{it.label}</span>
            {it.hint && <span className="muted menu-hint">{it.hint}</span>}
            {it.sub && <span className="muted menu-more" aria-hidden="true">›</span>}
          </button>
          {it.sub && sub === i && (
            <div className="submenu">
              <MenuList items={it.sub} close={close} autoFocus={subKbd} label={it.label ?? ''}
                back={() => { setSub(null); rowButton(i)?.focus(); }} />
            </div>
          )}
        </div>
      ))}
    </div>
  );
}
