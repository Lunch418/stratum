// Векторные иконки интерфейса: один набор пиктограмм 16×16 в currentColor,
// чтобы жить в обеих темах.
const PATHS: Record<string, string> = {
  play: 'M4 2.5v11l9-5.5z',
  pause: 'M4 3h3v10H4zM9 3h3v10H9z',
  step: 'M3 3h2v10H3zM7 3l7 5-7 5z',
  back: 'M13 3h-2v10h2zM9 3 2 8l7 5z',
  reset: 'M8 2.5a5.5 5.5 0 1 0 5.5 5.5M13.5 2.5v4h-4',
  select: 'M3 2l10 5.5-4.5 1.2L6.3 13z',
  pan: 'M8 1.5l2 2h-4zM8 14.5l-2-2h4zM1.5 8l2-2v4zM14.5 8l-2 2V6zM8 5.5v5M5.5 8h5',
  line: 'M2.5 13.5l11-11',
  polyline: 'M2 12l4-7 3 4 5-6',
  rect: 'M2.5 3.5h11v9h-11z',
  roundrect: 'M4.5 3.5h7a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-7a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2z',
  ellipse: 'M8 3a5.5 5 0 1 0 0 10 5.5 5 0 1 0 0-10z',
  arc: 'M3 13A8 8 0 0 1 13 3',
  text: 'M3 3h10M8 3v10M6 13h4',
  points: 'M3 12l4-6 3 3 3-5M3 12h.01M7 6h.01M10 9h.01M13 4h.01',
  group: 'M2.5 2.5h6v6h-6zM7.5 7.5h6v6h-6z',
  ungroup: 'M2.5 2.5h5v5h-5zM8.5 8.5h5v5h-5z',
  up: 'M8 13V3M4 7l4-4 4 4',
  down: 'M8 3v10M4 9l4 4 4-4',
  trash: 'M3 4h10M6 4V2.5h4V4M5 4l.7 9h4.6L11 4',
  undo: 'M6 4H2v4M2 8a6 6 0 1 1 1.8 4.2',
  redo: 'M10 4h4v4M14 8a6 6 0 1 0-1.8 4.2',
  sun: 'M8 5a3 3 0 1 0 0 6 3 3 0 0 0 0-6zM8 1v2M8 13v2M1 8h2M13 8h2M3 3l1.5 1.5M11.5 11.5 13 13M3 13l1.5-1.5M11.5 4.5 13 3',
  moon: 'M13 9.5A5.5 5.5 0 0 1 6.5 3a5.5 5.5 0 1 0 6.5 6.5z',
};

const FILLED = new Set(['play', 'pause', 'step', 'back', 'select']);

export function Icon({ name, size = 16, className }: { name: string; size?: number; className?: string }) {
  const d = PATHS[name];
  if (!d) return null;
  const filled = FILLED.has(name);
  return (
    <svg className={className} width={size} height={size} viewBox="0 0 16 16" fill={filled ? 'currentColor' : 'none'} stroke="currentColor" strokeWidth={filled ? 0 : 1.5} strokeLinecap="round" strokeLinejoin="round" aria-hidden="true">
      <path d={d} />
    </svg>
  );
}
