//! Designaciones de referencia EPLAN (IEC 81346).
//!
//! `=VA2+A1-FC1:6/T3` se lee como función `=VA2`, emplazamiento `+A1`,
//! dispositivo `-FC1` y punto de conexión `:6/T3`. Las páginas usan además
//! tipo de documento `&EFS1` y número de página `/1` (o `#01`).
//!
//! Reglas de lectura:
//!
//! - `=` y `+` pueden repetirse (`==EES`, `++Infra`): se conserva el texto tal
//!   cual, sin el prefijo inicial.
//! - El dispositivo va del primer `-` al primer `:`. Puede llevar más `-`
//!   (dispositivos anidados: `-QM1-MB1`).
//! - Todo lo que sigue al primer `:` tras el dispositivo es el punto de
//!   conexión, aunque lleve más `:` (`-KF2:X1:1`, `:RS485:17`).

use serde::Serialize;

/// Designación descompuesta. Las partes vacías (`=+A4-FC4`) son `None`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Designation {
    /// Texto original.
    pub full: String,
    /// Designación de función, sin `=` (`VA2`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<String>,
    /// Emplazamiento, sin `+` (`A1`, `B2.X1`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    /// Tipo de documento, sin `&` (`EFS1`). Solo en páginas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_type: Option<String>,
    /// Página (`1`), tras `/` o `#`. Solo en páginas.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page: Option<String>,
    /// Dispositivo, sin el primer `-` (`FC1`, `QM1-MB1`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device: Option<String>,
    /// Punto de conexión, sin `:` (`6/T3`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub point: Option<String>,
}

impl Designation {
    pub fn parse(text: &str) -> Self {
        let full = text.trim();
        let mut d = Designation {
            full: full.to_owned(),
            ..Default::default()
        };
        let mut rest = full;
        while let Some(prefix) = rest.chars().next() {
            let body = rest.trim_start_matches(prefix);
            let (value, next) = match prefix {
                '=' => split_at_any(body, &['+', '&', '#', '/', '-', ':']),
                '+' => split_at_any(body, &['&', '#', '/', '-', ':']),
                '&' => split_at_any(body, &['#', '/', '-', ':']),
                '-' => split_at_any(body, &[':']),
                // El punto de conexión y la página se quedan con el resto.
                ':' | '#' | '/' => (rest[1..].trim(), ""),
                _ => break,
            };
            let value = (!value.is_empty()).then(|| value.to_owned());
            match prefix {
                '=' => d.function = value,
                '+' => d.location = value,
                '&' => d.document_type = value,
                '-' => d.device = value,
                ':' => d.point = value,
                _ => d.page = value,
            }
            rest = next;
        }
        d
    }

    /// Dispositivo completo, sin el punto de conexión: `=VA2+A1-FC1`.
    pub fn device_tag(&self) -> Option<String> {
        let device = self.device.as_deref()?;
        let mut out = String::new();
        if let Some(f) = &self.function {
            out.push('=');
            out.push_str(f);
        }
        if let Some(l) = &self.location {
            out.push('+');
            out.push_str(l);
        }
        out.push('-');
        out.push_str(device);
        Some(out)
    }

    /// Mismo dispositivo (función, emplazamiento y dispositivo iguales).
    pub fn same_device(&self, other: &Designation) -> bool {
        self.device.is_some()
            && self.device == other.device
            && self.location == other.location
            && self.function == other.function
    }
}

/// Separa `text` en el primer carácter de `stops`: (valor, resto con el
/// separador).
fn split_at_any<'a>(text: &'a str, stops: &[char]) -> (&'a str, &'a str) {
    match text.find(stops) {
        Some(i) => (&text[..i], &text[i..]),
        None => (text, ""),
    }
}

/// Separa la dirección de salida que EPLAN añade tras un espacio en
/// ep.31144/31145 (`+A1-FC2:1 HR` → `+A1-FC2:1`, `HR`).
pub fn split_exit_direction(text: &str) -> (&str, Option<&str>) {
    match text.trim().rsplit_once(' ') {
        Some((designation, dir)) if !dir.is_empty() => (designation.trim_end(), Some(dir)),
        _ => (text.trim(), None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parts(d: &Designation) -> [Option<&str>; 6] {
        [
            d.function.as_deref(),
            d.location.as_deref(),
            d.document_type.as_deref(),
            d.page.as_deref(),
            d.device.as_deref(),
            d.point.as_deref(),
        ]
    }

    #[test]
    fn full_connection_point() {
        let d = Designation::parse("=VA2+A1-FC1:6/T3");
        assert_eq!(
            parts(&d),
            [
                Some("VA2"),
                Some("A1"),
                None,
                None,
                Some("FC1"),
                Some("6/T3")
            ]
        );
        assert_eq!(d.device_tag().as_deref(), Some("=VA2+A1-FC1"));
    }

    #[test]
    fn nested_devices_and_points_with_colons() {
        let d = Designation::parse("=KL1+B2.Y1-KF2:X1:1");
        assert_eq!(d.location.as_deref(), Some("B2.Y1"));
        assert_eq!(d.device.as_deref(), Some("KF2"));
        assert_eq!(d.point.as_deref(), Some("X1:1"));

        let d = Designation::parse("=GL1+B2.Y1-BP1-S1:1");
        assert_eq!(d.device.as_deref(), Some("BP1-S1"));
        assert_eq!(d.point.as_deref(), Some("1"));

        let d = Designation::parse("=GB1+A1-TB1:+");
        assert_eq!(d.point.as_deref(), Some("+"));
    }

    #[test]
    fn empty_parts_are_none() {
        let d = Designation::parse("=+A4-FC4:14");
        assert_eq!(
            parts(&d),
            [None, Some("A4"), None, None, Some("FC4"), Some("14")]
        );
        let d = Designation::parse("+-U101");
        assert_eq!(parts(&d), [None, None, None, None, Some("U101"), None]);
        assert_eq!(d.device_tag().as_deref(), Some("-U101"));
    }

    #[test]
    fn interruption_point_and_page() {
        let d = Designation::parse("=GB1-0V");
        assert_eq!(d.function.as_deref(), Some("GB1"));
        assert_eq!(d.device.as_deref(), Some("0V"));

        let d = Designation::parse("=GB1+A1&EFS1/1");
        assert_eq!(
            parts(&d),
            [Some("GB1"), Some("A1"), Some("EFS1"), Some("1"), None, None]
        );
    }

    #[test]
    fn not_a_designation() {
        let d = Designation::parse("Multi-Standard SC 2.2");
        assert_eq!(parts(&d), [None; 6]);
        assert_eq!(d.full, "Multi-Standard SC 2.2");
    }

    #[test]
    fn same_device() {
        let a = Designation::parse("+A1-XD1:1");
        let b = Designation::parse("+A1-XD1:4");
        let c = Designation::parse("=GB1+A1-XD1:4");
        assert!(a.same_device(&b));
        assert!(!a.same_device(&c));
    }

    #[test]
    fn exit_direction() {
        assert_eq!(
            split_exit_direction("+A1-FC2:1 HR"),
            ("+A1-FC2:1", Some("HR"))
        );
        assert_eq!(split_exit_direction("+A1-FC2:1"), ("+A1-FC2:1", None));
    }
}
