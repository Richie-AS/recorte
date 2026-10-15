//! Copia de imágenes al portapapeles.
//!
//! En Linux el contenido del portapapeles pertenece al proceso que lo copió y desaparece
//! cuando ese proceso termina. Por eso la imagen se entrega a un proceso hijo en segundo
//! plano que la sirve hasta que otra aplicación ocupe el portapapeles.

use std::borrow::Cow;
use std::error::Error;
use std::io::{Read, Write};
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

use arboard::{Clipboard, ImageData, SetExtLinux};
use image::RgbaImage;

/// Argumento con el que se relanza el ejecutable en modo "servidor de portapapeles".
pub const DAEMON_ARG: &str = "--clipboard-daemon";

/// Lanza el proceso en segundo plano y le envía la imagen por stdin.
pub fn copy_in_background(image: &RgbaImage) -> Result<(), Box<dyn Error>> {
    let mut child = Command::new(std::env::current_exe()?)
        .arg(DAEMON_ARG)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        // Grupo de procesos propio: sobrevive al cierre de la terminal.
        .process_group(0)
        .spawn()?;

    let mut stdin = child.stdin.take().ok_or("sin stdin en el proceso hijo")?;
    stdin.write_all(&image.width().to_le_bytes())?;
    stdin.write_all(&image.height().to_le_bytes())?;
    stdin.write_all(image.as_raw())?;
    Ok(())
}

/// Punto de entrada del proceso hijo: lee la imagen y la sirve hasta ser reemplazada.
pub fn run_daemon() -> Result<(), Box<dyn Error>> {
    let mut input = std::io::stdin().lock();
    let mut header = [0u8; 8];
    input.read_exact(&mut header)?;
    let width = u32::from_le_bytes(header[0..4].try_into()?) as usize;
    let height = u32::from_le_bytes(header[4..8].try_into()?) as usize;

    let mut bytes = vec![0u8; width * height * 4];
    input.read_exact(&mut bytes)?;

    Clipboard::new()?.set().wait().image(ImageData {
        width,
        height,
        bytes: Cow::Owned(bytes),
    })?;
    Ok(())
}
