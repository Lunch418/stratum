// Ввод пути к папке: «Сохранить как» и «Экспорт в Stratum 2000».
// Свой диалог вместо window.prompt — его нет в WebView Tauri.
import { useEffect, useRef, useState } from 'react';

interface Props {
  title: string;
  hint?: string;
  initial: string;
  action: string;
  onSubmit: (path: string) => void;
  onClose: () => void;
}

export function PathDialog({ title, hint, initial, action, onSubmit, onClose }: Props) {
  const [value, setValue] = useState(initial);
  const input = useRef<HTMLInputElement>(null);
  useEffect(() => { input.current?.focus(); input.current?.select(); }, []);
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <form className="modal" onMouseDown={e => e.stopPropagation()} onSubmit={e => { e.preventDefault(); if (value.trim()) onSubmit(value.trim()); }}>
        <div className="panel-title">{title}</div>
        <div className="modal-body">
          <input ref={input} type="text" className="mono" value={value} onChange={e => setValue(e.target.value)}
            onKeyDown={e => { if (e.key === 'Escape') onClose(); }} spellCheck={false} />
          {hint && <div className="muted small" style={{ marginTop: 6 }}>{hint}</div>}
        </div>
        <div className="modal-actions">
          <button type="button" className="ghost" onClick={onClose}>Отмена</button>
          <button type="submit" className="primary">{action}</button>
        </div>
      </form>
    </div>
  );
}
