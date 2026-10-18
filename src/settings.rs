//! Preferencias que se recuerdan entre capturas (`~/.config/recorte/ajustes`).
//!
//! Es un archivo de texto `clave=valor` que también se puede editar a mano:
//!
//! ```text
//! color=#e01b24
//! grosor=medio        # fino | medio | grueso
//! idioma=auto         # auto | es | en | pt
//! ```

use std::path::PathBuf;

use egui::Color32;

use crate::annotations::{Style, Thickness};
use crate::i18n::Choice;

fn path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("recorte").join("ajustes"))
}

fn read() -> String {
    path().and_then(|p| std::fs::read_to_string(p).ok()).unwrap_or_default()
}

/// Cambia solo las claves indicadas y conserva el resto del archivo.
/// Un error aquí no debe interrumpir la captura.
fn update(values: &[(&str, String)]) {
    let Some(path) = path() else { return };
    let text = set_values(&read(), values);
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, text);
}

fn get<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .find(|(k, _)| k.trim() == key)
        .map(|(_, v)| strip_comment(v.trim()))
}

/// Quita un comentario final ` # ...`. El `#` de un color como `#e01b24` no cuenta,
/// porque no va precedido de un espacio.
fn strip_comment(value: &str) -> &str {
    match value.find(" #").or_else(|| value.find("\t#")) {
        Some(i) => value[..i].trim_end(),
        None => value,
    }
}

fn set_values(text: &str, values: &[(&str, String)]) -> String {
    let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
    for (key, value) in values {
        let entry = format!("{key}={value}");
        match lines.iter().position(|l| l.split_once('=').is_some_and(|(k, _)| k.trim() == *key)) {
            Some(i) => lines[i] = entry,
            None => lines.push(entry),
        }
    }
    lines.join("\n") + "\n"
}

/// Estilo guardado, o el predeterminado si no hay archivo o está dañado.
pub fn load_style() -> Style {
    parse_style(&read())
}

pub fn save_style(style: Style) {
    let [r, g, b, _] = style.color.to_srgba_unmultiplied();
    update(&[("color", format!("#{r:02x}{g:02x}{b:02x}")), ("grosor", style.thickness.id().to_owned())]);
}

/// Idioma elegido; `auto` (seguir al sistema) si no hay nada guardado.
pub fn load_language() -> Choice {
    get(&read(), "idioma").and_then(Choice::from_code).unwrap_or(Choice::Auto)
}

pub fn save_language(choice: Choice) {
    update(&[("idioma", choice.code().to_owned())]);
}

fn parse_style(text: &str) -> Style {
    let mut style = Style::default();
    if let Some(color) = get(text, "color").and_then(parse_hex) {
        style.color = color;
    }
    if let Some(thickness) = get(text, "grosor").and_then(Thickness::from_id) {
        style.thickness = thickness;
    }
    style
}

fn parse_hex(value: &str) -> Option<Color32> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 {
        return None;
    }
    let channel = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).ok();
    Some(Color32::from_rgb(channel(0)?, channel(2)?, channel(4)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::annotations::PALETTE;
    use crate::i18n::Lang;

    #[test]
    fn round_trips_style() {
        let style = Style { color: PALETTE[3], thickness: Thickness::Thick };
        let [r, g, b, _] = style.color.to_srgba_unmultiplied();
        let text = set_values("", &[("color", format!("#{r:02x}{g:02x}{b:02x}")), ("grosor", "grueso".into())]);
        assert_eq!(parse_style(&text), style);
    }

    #[test]
    fn ignores_garbage() {
        assert_eq!(parse_style("color=#zz\ngrosor=enorme\nbasura"), Style::default());
    }

    #[test]
    fn updating_one_key_keeps_the_others() {
        let text = "color=#33d17a\ngrosor=fino\n";
        let text = set_values(text, &[("idioma", "pt".into())]);
        let text = set_values(&text, &[("grosor", "grueso".into())]);
        assert_eq!(text, "color=#33d17a\ngrosor=grueso\nidioma=pt\n");
        assert_eq!(get(&text, "idioma").and_then(Choice::from_code), Some(Choice::Fixed(Lang::Pt)));
    }

    #[test]
    fn hand_edited_values_with_comments() {
        let text = "idioma = en   # elegido a mano\ncolor=#ff7800 # naranja\n";
        assert_eq!(get(text, "idioma").and_then(Choice::from_code), Some(Choice::Fixed(Lang::En)));
        assert_eq!(get(text, "color"), Some("#ff7800"));
    }
}
