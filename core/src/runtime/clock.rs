//! Системные часы для функций модели: местное время (`GetDate`, `GetTime`)
//! и счётчик миллисекунд (`GetTickCount`). Без внешних библиотек: смещение
//! часового пояса на Unix берётся из TZif-файла (`$TZ` или `/etc/localtime`),
//! на Windows — из `GetTimeZoneInformation`.

use std::time::{SystemTime, UNIX_EPOCH};

/// Секунды Unix-времени сейчас.
fn utc_now() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}

/// Местное время: секунды от 1970-01-01 00:00 по местным часам.
pub fn local_now() -> i64 {
    let now = utc_now();
    now + offset_at(now)
}

/// Сотые доли текущей секунды.
pub fn hundredths() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.subsec_millis() as i64 / 10).unwrap_or(0)
}

/// Разбивает местные секунды на (год, месяц, день, час, минута, секунда).
pub fn split(local: i64) -> (i64, i64, i64, i64, i64, i64) {
    let days = local.div_euclid(86400);
    let rest = local.rem_euclid(86400);
    // гражданский календарь от дня 0 = 1970-01-01 (алгоритм Хиннанта)
    let z = days + 719468;
    let era = z.div_euclid(146097);
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d, rest / 3600, rest / 60 % 60, rest % 60)
}

/// Миллисекунды с загрузки системы, как `GetTickCount` в Windows.
pub fn tick_count() -> f64 {
    static START: std::sync::OnceLock<(std::time::Instant, u64)> = std::sync::OnceLock::new();
    let (at, base) = START.get_or_init(|| (std::time::Instant::now(), uptime_ms()));
    // GetTickCount — 32-битный счётчик, через 49,7 суток он начинается с нуля
    ((base + at.elapsed().as_millis() as u64) % (1u64 << 32)) as f64
}

#[cfg(windows)]
fn uptime_ms() -> u64 {
    #[link(name = "kernel32")]
    extern "system" {
        fn GetTickCount64() -> u64;
    }
    // SAFETY: функция без аргументов, только читает системный счётчик
    unsafe { GetTickCount64() }
}

#[cfg(not(windows))]
fn uptime_ms() -> u64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|s| s.split_whitespace().next().and_then(|v| v.parse::<f64>().ok()))
        .map(|secs| (secs * 1000.0) as u64)
        .unwrap_or(0)
}

/// Смещение местного времени от UTC в секундах на момент `utc`.
#[cfg(windows)]
fn offset_at(_utc: i64) -> i64 {
    #[repr(C)]
    struct SystemTimeW([u16; 8]);
    #[repr(C)]
    struct TimeZoneInformation {
        bias: i32,
        standard_name: [u16; 32],
        standard_date: SystemTimeW,
        standard_bias: i32,
        daylight_name: [u16; 32],
        daylight_date: SystemTimeW,
        daylight_bias: i32,
    }
    #[link(name = "kernel32")]
    extern "system" {
        fn GetTimeZoneInformation(info: *mut TimeZoneInformation) -> u32;
    }
    let mut info = TimeZoneInformation {
        bias: 0,
        standard_name: [0; 32],
        standard_date: SystemTimeW([0; 8]),
        standard_bias: 0,
        daylight_name: [0; 32],
        daylight_date: SystemTimeW([0; 8]),
        daylight_bias: 0,
    };
    // SAFETY: передаём указатель на структуру нужной раскладки, функция её заполняет
    let mode = unsafe { GetTimeZoneInformation(&mut info) };
    // UTC = местное + bias (в минутах); 2 — сейчас летнее время
    let bias = info.bias + if mode == 2 { info.daylight_bias } else { info.standard_bias };
    -(bias as i64) * 60
}

#[cfg(not(windows))]
fn offset_at(utc: i64) -> i64 {
    let path = match std::env::var("TZ") {
        Ok(tz) if !tz.is_empty() => {
            let name = tz.trim_start_matches(':');
            if name.starts_with('/') { name.to_string() } else { format!("/usr/share/zoneinfo/{name}") }
        }
        _ => "/etc/localtime".to_string(),
    };
    std::fs::read(path).ok().and_then(|data| tzif_offset(&data, utc)).unwrap_or(0)
}

/// Смещение из TZif (RFC 8536) на момент `utc`: последний переход не позже
/// него; до первого перехода — первый тип без летнего времени. Берётся
/// 64-битная часть (версия 2+), если она есть.
#[cfg_attr(windows, allow(dead_code))]
fn tzif_offset(data: &[u8], utc: i64) -> Option<i64> {
    let be32 = |at: usize| -> Option<u32> { data.get(at..at + 4).map(|b| u32::from_be_bytes([b[0], b[1], b[2], b[3]])) };
    let header = |at: usize| -> Option<[usize; 6]> {
        if data.get(at..at + 4)? != b"TZif" {
            return None;
        }
        let mut c = [0usize; 6];
        for (i, v) in c.iter_mut().enumerate() {
            *v = be32(at + 20 + i * 4)? as usize;
        }
        Some(c)
    };
    let [isut, isstd, leap, time, types, chars] = header(0)?;
    let v1_len = 44 + time * 5 + types * 6 + chars + leap * 8 + isstd + isut;
    // версия 2+: вторая, 64-битная часть сразу за первой
    let (base, width, counts) = match data.get(4) {
        Some(b'2'..=b'9') => (v1_len, 8usize, header(v1_len)?),
        _ => (0, 4usize, [isut, isstd, leap, time, types, chars]),
    };
    let [_, _, _, time, types, _] = counts;
    let times_at = base + 44;
    let idx_at = times_at + time * width;
    let types_at = idx_at + time;
    let transition = |i: usize| -> Option<i64> {
        let at = times_at + i * width;
        let b = data.get(at..at + width)?;
        Some(if width == 8 {
            i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
        } else {
            i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64
        })
    };
    let ttinfo = |t: usize| -> Option<(i64, bool)> {
        let at = types_at + t * 6;
        let b = data.get(at..at + 6)?;
        Some((i32::from_be_bytes([b[0], b[1], b[2], b[3]]) as i64, b[4] != 0))
    };
    let mut current = None;
    for i in 0..time {
        if transition(i)? > utc {
            break;
        }
        current = Some(*data.get(idx_at + i)? as usize);
    }
    let t = match current {
        Some(t) => t,
        None => (0..types).find(|&t| ttinfo(t).is_some_and(|(_, dst)| !dst)).unwrap_or(0),
    };
    ttinfo(t).map(|(off, _)| off)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_local_seconds_into_calendar_fields() {
        // 2000-02-29 13:45:07
        assert_eq!(split(951_831_907), (2000, 2, 29, 13, 45, 7));
        assert_eq!(split(0), (1970, 1, 1, 0, 0, 0));
    }

    #[test]
    fn reads_offset_from_tzif() {
        // TZif v1: один переход в 1000 с на тип 1 (+3 ч), до него тип 0 (+2 ч)
        let mut d = b"TZif".to_vec();
        d.extend([0u8; 16]);
        for n in [0u32, 0, 0, 1, 2, 4] {
            d.extend(n.to_be_bytes());
        }
        d.extend(1000i32.to_be_bytes());
        d.push(1);
        d.extend(7200i32.to_be_bytes());
        d.extend([0, 0]);
        d.extend(10800i32.to_be_bytes());
        d.extend([0, 0]);
        d.extend(b"AB\0\0");
        assert_eq!(tzif_offset(&d, 500), Some(7200));
        assert_eq!(tzif_offset(&d, 2000), Some(10800));
    }

    #[test]
    fn tick_count_grows() {
        let a = tick_count();
        std::thread::sleep(std::time::Duration::from_millis(5));
        assert!(tick_count() >= a + 4.0);
    }
}
