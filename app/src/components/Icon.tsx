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
  file: 'M4 1.5h5l3 3v10H4zM9 1.5v3h3',
  grid: 'M2.5 2.5h11v11h-11zM6 2.5v11M10 2.5v11M2.5 6h11M2.5 10h11',
  save: 'M2.5 2.5h9l2 2v9h-11zM5 2.5v4h5v-4M5 13.5v-4h6v4',
  open: 'M1.5 4.5h5l1.5 1.5h6.5v7h-13zM1.5 7h13',
  print: 'M4 6V2.5h8V6M3 6h10a1 1 0 0 1 1 1v4h-2.5M4.5 11H2V7a1 1 0 0 1 1-1M4.5 9.5h7v4h-7z',
  info: 'M8 1.5a6.5 6.5 0 1 0 0 13 6.5 6.5 0 0 0 0-13zM8 7v4M8 5h.01',
  search: 'M7 2.5a4.5 4.5 0 1 0 0 9 4.5 4.5 0 0 0 0-9zM10.5 10.5l3 3',
  stop: 'M3.5 3.5h9v9h-9z',
  close: 'M4 4l8 8M12 4l-8 8',
  folder: 'M1.5 3.5h4.5l1.5 1.5h7v8h-13z',
  project: 'M4 4.5h8v7H4zM1.5 8H4M12 8h2.5',
  bitmap: 'M2.5 2.5h11v11h-11zM2.5 10l3-3 3 3 2-2 3 3M10.5 5.5h.01',
  pencil: 'M3 13l1-4 7-7 3 3-7 7zM10 3l3 3',
  eraser: 'M2.5 10.5l6-6 4 4-6 6h-4zM6 14.5h8',
  frect: 'M2.5 3.5h11v9h-11zM2.5 6.5h11M2.5 9.5h11',
  fill: 'M3 8l5-5 5 5-5 5zM13 10c0 1.5-1 2.5-1 2.5s-1-1-1-2.5 1-1.5 1-1.5 1 .5 1 1.5',
  spray: 'M6 6h4v8H6zM8 2v2M5 3l1 1M11 3l-1 1M3 6h1M12 6h1',
  pick: 'M2.5 13.5l6-6M8 5l3 3M10 3l3 3M9 4l1-1 3 3-1 1',
  scissors: 'M4.5 4.5a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM4.5 9.5a2 2 0 1 0 0 4 2 2 0 0 0 0-4zM6.5 7.5l7-4M6.5 10.5l7 4',
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
