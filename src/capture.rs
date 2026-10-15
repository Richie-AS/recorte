//! Captura de pantalla mediante el portal XDG (`org.freedesktop.portal.Screenshot`).
//!
//! El portal devuelve un PNG con todos los monitores en el espacio lógico global.

use std::error::Error;
use std::path::PathBuf;

use ashpd::desktop::screenshot::Screenshot;
use image::RgbaImage;

pub fn capture_screen() -> Result<RgbaImage, Box<dyn Error>> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let uri = runtime.block_on(async {
        // Sin un identificador, el portal no recuerda el permiso de las apps que no
        // son Flatpak y vuelve a pedirlo cada vez. Portales antiguos no lo admiten.
        if let Ok(app_id) = ashpd::AppID::try_from(crate::APP_ID) {
            let _ = ashpd::register_host_app(app_id).await;
        }
        let response = Screenshot::request()
            .interactive(false)
            .modal(false)
            .send()
            .await?
            .response()?;
        Ok::<_, ashpd::Error>(response.uri().as_str().to_owned())
    })?;

    let path = file_uri_to_path(&uri).ok_or_else(|| format!("URI no soportada: {uri}"))?;
    let image = image::open(&path)?.to_rgba8();
    // El portal de GNOME guarda el PNG en ~/Imágenes; es temporal para nosotros.
    let _ = std::fs::remove_file(&path);
    Ok(image)
}

/// Convierte `file:///ruta%20con%20espacios.png` en una ruta local.
fn file_uri_to_path(uri: &str) -> Option<PathBuf> {
    let encoded = uri.strip_prefix("file://")?;
    let bytes = encoded.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            let hex = std::str::from_utf8(&bytes[i + 1..i + 3]).ok()?;
            decoded.push(u8::from_str_radix(hex, 16).ok()?);
            i += 3;
        } else {
            decoded.push(bytes[i]);
            i += 1;
        }
    }
    use std::os::unix::ffi::OsStringExt;
    Some(PathBuf::from(std::ffi::OsString::from_vec(decoded)))
}
