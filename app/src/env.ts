// Параметры среды: тип, значения по умолчанию и чтение из localStorage.
export interface UserInfo { name: string; org: string; email: string; addr: string; phone: string; notes: string }
export interface EnvOptions {
  autosave: number; pollMs: number; circlePoints: number; gridAuto: boolean; fontSize: number; confirmClose: boolean;
  /// "Параметры среды -> Вычисления": пройденные шаги и индикатор производительности в строке статуса
  statusTicks: boolean; statusPerf: boolean;
  /// "2D-редактор": курсор-рука при перемещении объектов
  handCursor: boolean;
  /// "Редактор": выделение синтаксиса, кегль, перенос строк, мини-карта
  syntax: boolean; editorFontSize: number; wordWrap: boolean; minimap: boolean;
  /// "Пользователь": данные автора и копирование их в новый проект
  user: UserInfo; copyUser: boolean;
  /// "2D-редактор -> Шаблоны": файлы .vdr для рисунка и схемы нового имиджа
  templates: { image: string; scheme: string };
  /// "Методы" и "Переменные" по умолчанию - свойства нового проекта
  projectDefaults: ProjectDefaults;
}
export interface ProjectDefaults { lin_method: string; nl_method: string; newton_iter: number; newton_eps: number; vars_logset: number; vars_preload: number }
const defaultProjectDefaults: ProjectDefaults = { lin_method: 'gauss', nl_method: 'newton', newton_iter: 50, newton_eps: 6, vars_logset: 0, vars_preload: 0 };
const defaultEnv: EnvOptions = {
  autosave: 30, pollMs: 40, circlePoints: 32, gridAuto: false, fontSize: 13, confirmClose: true,
  statusTicks: true, statusPerf: false, handCursor: true, syntax: true, editorFontSize: 13, wordWrap: false, minimap: false,
  user: { name: '', org: '', email: '', addr: '', phone: '', notes: '' }, copyUser: true,
  templates: { image: '', scheme: '' },
  projectDefaults: defaultProjectDefaults,
};
export function loadEnv(): EnvOptions {
  try {
    const saved = JSON.parse(localStorage.getItem('env') ?? '{}') as Partial<EnvOptions>;
    return { ...defaultEnv, ...saved, user: { ...defaultEnv.user, ...(saved.user ?? {}) }, templates: { ...defaultEnv.templates, ...(saved.templates ?? {}) }, projectDefaults: { ...defaultProjectDefaults, ...(saved.projectDefaults ?? {}) } };
  } catch { return defaultEnv; }
}
