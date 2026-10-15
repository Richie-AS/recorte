//! Guardado de la captura en un archivo PNG.

use std::error::Error;
use std::path::PathBuf;

use image::RgbaImage;

/// Carpeta de capturas de GNOME (`~/Imágenes/Capturas de pantalla` en español),
/// o la carpeta de imágenes si no existe.
pub fn screenshots_dir() -> PathBuf {
    let pictures = dirs::picture_dir()
        .or_else(dirs::home_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    ["Capturas de pantalla", "Screenshots"]
        .iter()
        .map(|name| pictures.join(name))
        .find(|dir| dir.is_dir())
        .unwrap_or(pictures)
}

/// Guarda la imagen con la fecha y hora en el nombre. Devuelve la ruta final.
pub fn save_png(image: &RgbaImage) -> Result<PathBuf, Box<dyn Error>> {
    let dir = screenshots_dir();
    std::fs::create_dir_all(&dir)?;

    let stamp = chrono::Local::now().format("%Y-%m-%d %H-%M-%S");
    let mut path = dir.join(format!("Captura {stamp}.png"));
    // Dos capturas en el mismo segundo no deben pisarse.
    let mut n = 2;
    while path.exists() {
        path = dir.join(format!("Captura {stamp} ({n}).png"));
        n += 1;
    }

    image.save(&path)?;
    Ok(path)
}
