//! Preferencias que se recuerdan entre capturas (`~/.config/recorte/ajustes`).

use std::path::PathBuf;

use egui::Color32;

use crate::annotations::{Style, Thickness};

fn path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("recorte").join("ajustes"))
}

/// Estilo guardado, o el predeterminado si no hay archivo o está dañado.
pub fn load_style() -> Style {
    path()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|text| parse_style(&text))
        .unwrap_or_default()
}

/// Guarda el estilo. Un error aquí no debe interrumpir la captura.
pub fn save_style(style: Style) {
    let Some(path) = path() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let _ = std::fs::write(path, format_style(style));
}

fn format_style(style: Style) -> String {
    let [r, g, b, _] = style.color.to_srgba_unmultiplied();
    format!("color=#{r:02x}{g:02x}{b:02x}\ngrosor={}\n", style.thickness.name().to_lowercase())
}

fn parse_style(text: &str) -> Style {
    let mut style = Style::default();
    for line in text.lines() {
        match line.split_once('=') {
            Some(("color", value)) => {
                if let Some(color) = parse_hex(value.trim()) {
                    style.color = color;
                }
            }
            Some(("grosor", value)) => {
                if let Some(t) = Thickness::ALL.into_iter().find(|t| t.name().eq_ignore_ascii_case(value.trim())) {
                    style.thickness = t;
                }
            }
            _ => {}
        }
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

    #[test]
    fn round_trips_style() {
        let style = Style { color: PALETTE[3].1, thickness: Thickness::Thick };
        assert_eq!(parse_style(&format_style(style)), style);
    }

    #[test]
    fn ignores_garbage() {
        assert_eq!(parse_style("color=#zz\ngrosor=enorme\nbasura"), Style::default());
    }
}
