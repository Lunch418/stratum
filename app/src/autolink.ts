// Автосоединение переменных по закладке «Переменные» объектов рисунка
// (справка оригинала «Object2D options | Vars»): напротив переменной имиджа
// записано имя переменной другого имиджа или общий псевдоним (pin1). При
// проведении связи совпавшие переменные одного типа соединяются сами.
import type { ClassInfo } from './api';

/// Сопоставления переменных имиджа: имя переменной (в нижнем регистре) → имя или псевдоним у другого имиджа.
async function varMap(cls: ClassInfo): Promise<Map<string, string>> {
  const map = new Map<string, string>();
  for (const kind of ['image', 'scheme']) {
    try {
      const r = await fetch(`/api/picture/${encodeURIComponent(cls.name)}?kind=${kind}`);
      if (!r.ok) continue;
      const j = await r.json() as { objectVars?: Record<string, string> };
      for (const text of Object.values(j.objectVars ?? {})) {
        // пары «переменная,переменная другого имиджа» через «;»
        for (const pair of text.split(';')) {
          const [n, t] = pair.split(',').map(x => (x ?? '').trim());
          if (n && t) map.set(n.toLowerCase(), t);
        }
      }
    } catch { /* рисунка нет */ }
  }
  return map;
}

/// Пары переменных для новой связи между экземплярами имиджей `src` и `dst`.
export async function suggestPairs(src: ClassInfo | undefined, dst: ClassInfo | undefined): Promise<[string, string][]> {
  if (!src || !dst) return [];
  const [ms, md] = await Promise.all([varMap(src), varMap(dst)]);
  if (!ms.size && !md.size) return [];
  const type = (c: ClassInfo, n: string) => c.vars.find(v => v.name.toLowerCase() === n.toLowerCase())?.type.toLowerCase();
  const pairs: [string, string][] = [];
  for (const sv of src.vars) {
    const alias = ms.get(sv.name.toLowerCase());
    if (!alias) continue;
    const same = (v: ClassInfo['vars'][number]) => type(dst, v.name) === type(src, sv.name);
    // у приёмника: сначала переменная с именем из сопоставления, затем — с тем же псевдонимом
    const dv = dst.vars.find(v => same(v) && v.name.toLowerCase() === alias.toLowerCase())
      ?? dst.vars.find(v => same(v) && md.get(v.name.toLowerCase())?.toLowerCase() === alias.toLowerCase());
    if (dv && !pairs.some(p => p[1] === dv.name)) pairs.push([sv.name, dv.name]);
  }
  return pairs;
}
