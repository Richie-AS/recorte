//! Atajo global de GNOME (`recorte --instalar-atajo`).
//!
//! Un paquete del sistema no puede crear atajos por usuario, así que cada usuario
//! lo registra con este comando. Se añade sin borrar otros atajos personalizados.
//!
//! También concede a Recorte el permiso del portal para capturar la pantalla sin
//! preguntar: el diálogo de permiso solo puede mostrarse si la app tiene el foco,
//! y al lanzarse desde el atajo todavía no tiene ventana.

use std::error::Error;
use std::process::Command;

pub const INSTALL_ARG: &str = "--instalar-atajo";

const SCHEMA: &str = "org.gnome.settings-daemon.plugins.media-keys";
const PATH: &str = "/org/gnome/settings-daemon/plugins/media-keys/custom-keybindings/recorte/";
const BINDING: &str = "<Control>Print";

pub fn install() -> Result<(), Box<dyn Error>> {
    let current = gsettings(&["get", SCHEMA, "custom-keybindings"])?;
    let mut paths = parse_string_list(&current);
    if !paths.iter().any(|p| p == PATH) {
        paths.push(PATH.to_owned());
        let list = paths.iter().map(|p| format!("'{p}'")).collect::<Vec<_>>().join(", ");
        gsettings(&["set", SCHEMA, "custom-keybindings", &format!("[{list}]")])?;
    }

    let exe = std::env::current_exe()?;
    let keybinding = format!("{SCHEMA}.custom-keybinding:{PATH}");
    gsettings(&["set", &keybinding, "name", "Recorte"])?;
    gsettings(&["set", &keybinding, "command", &exe.to_string_lossy()])?;
    gsettings(&["set", &keybinding, "binding", BINDING])?;
    grant_screenshot_permission()?;
    Ok(())
}

fn grant_screenshot_permission() -> Result<(), Box<dyn Error>> {
    let output = Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest=org.freedesktop.impl.portal.PermissionStore",
            "--object-path=/org/freedesktop/impl/portal/PermissionStore",
            "--method=org.freedesktop.impl.portal.PermissionStore.SetPermission",
            "screenshot",
            "true",
            "screenshot",
            crate::APP_ID,
            "['yes']",
        ])
        .output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr);
        return Err(format!("no se pudo dar permiso de captura: {}", detail.trim()).into());
    }
    Ok(())
}

fn gsettings(args: &[&str]) -> Result<String, Box<dyn Error>> {
    let output = Command::new("gsettings").args(args).output()?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned().into());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Interpreta la salida de gsettings para listas de cadenas: `@as []` o `['a', 'b']`.
fn parse_string_list(value: &str) -> Vec<String> {
    value
        .trim_start_matches("@as")
        .trim()
        .trim_start_matches('[')
        .trim_end_matches(']')
        .split(',')
        .map(|item| item.trim().trim_matches('\'').to_owned())
        .filter(|item| !item.is_empty())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gsettings_lists() {
        assert!(parse_string_list("@as []").is_empty());
        assert_eq!(parse_string_list("['/a/', '/b/']"), vec!["/a/", "/b/"]);
    }
}
