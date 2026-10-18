//! Icono en la bandeja del sistema (`recorte --bandeja`).
//!
//! Proceso residente que muestra el icono hasta que el usuario elige "Salir".
//! Cada captura sigue siendo un proceso aparte, así un fallo en la captura no
//! cierra el icono. En GNOME requiere la extensión AppIndicator (incluida en Ubuntu).
//!
//! El submenú "Configuración" permite elegir el idioma (automático o fijo) e
//! iniciar el icono con la sesión.

use std::fs::File;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

use ksni::blocking::TrayMethods;
use ksni::menu::{CheckmarkItem, RadioGroup, RadioItem, StandardItem, SubMenu};
use ksni::{MenuItem, ToolTip};

use crate::i18n::{self, Choice, Lang};

pub const TRAY_ARG: &str = "--bandeja";
pub const TRAY_ARG_EN: &str = "--tray";

/// Opciones del submenú de idioma, en orden.
const LANGUAGE_CHOICES: [Choice; 4] = [Choice::Auto, Choice::Fixed(Lang::Es), Choice::Fixed(Lang::En), Choice::Fixed(Lang::Pt)];

/// Tiempo para que se cierre el menú antes de capturar, si no saldría en la imagen.
const MENU_CLOSE_DELAY: Duration = Duration::from_millis(400);

struct RecorteTray;

impl ksni::Tray for RecorteTray {
    fn id(&self) -> String {
        crate::APP_ID.into()
    }

    fn title(&self) -> String {
        "Recorte".into()
    }

    fn icon_name(&self) -> String {
        "applets-screenshooter".into()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "Recorte".into(),
            description: i18n::t().tray_tooltip.into(),
            ..Default::default()
        }
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        start_capture();
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let t = i18n::t();
        let current = crate::settings::load_language();
        let language_options = LANGUAGE_CHOICES
            .iter()
            .map(|choice| RadioItem {
                label: match choice {
                    Choice::Auto => t.tray_language_auto.into(),
                    Choice::Fixed(lang) => lang.native_name().into(),
                },
                ..Default::default()
            })
            .collect();

        vec![
            StandardItem {
                label: t.tray_capture.into(),
                icon_name: "applets-screenshooter".into(),
                activate: Box::new(|_| start_capture()),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: t.tray_open_folder.into(),
                icon_name: "folder-pictures".into(),
                activate: Box::new(|_| {
                    let _ = Command::new("xdg-open").arg(crate::save::screenshots_dir()).spawn();
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            SubMenu {
                label: t.tray_settings.into(),
                icon_name: "preferences-system".into(),
                submenu: vec![
                    SubMenu {
                        label: t.tray_language.into(),
                        icon_name: "preferences-desktop-locale".into(),
                        submenu: vec![
                            RadioGroup {
                                selected: LANGUAGE_CHOICES.iter().position(|c| *c == current).unwrap_or(0),
                                select: Box::new(|_, index| {
                                    i18n::set_choice(LANGUAGE_CHOICES[index]);
                                    refresh_autostart_entry();
                                }),
                                options: language_options,
                            }
                            .into(),
                        ],
                        ..Default::default()
                    }
                    .into(),
                    CheckmarkItem {
                        label: t.tray_autostart.into(),
                        checked: autostart_path().is_some_and(|p| p.exists()),
                        activate: Box::new(|_| toggle_autostart()),
                        ..Default::default()
                    }
                    .into(),
                ],
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: t.tray_quit.into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}

/// Ejecuta la bandeja hasta que el usuario elige "Salir".
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let Some(_lock) = lock() else {
        return Ok(()); // Ya hay un icono en la bandeja.
    };
    let _handle = RecorteTray.spawn()?;
    loop {
        std::thread::park();
    }
}

/// Inicia la bandeja en segundo plano si todavía no está en marcha.
pub fn ensure_running() {
    // Si el bloqueo está libre no hay bandeja; se suelta enseguida para el hijo.
    match lock() {
        Some(free) => drop(free),
        None => return,
    }
    if let Ok(exe) = std::env::current_exe() {
        let _ = Command::new(exe)
            .arg(TRAY_ARG)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn();
    }
}

/// Bloqueo que indica que la bandeja está en marcha (se libera al salir).
fn lock() -> Option<File> {
    let file = File::create(crate::runtime_dir().join("recorte-bandeja.lock")).ok()?;
    file.try_lock().ok()?;
    Some(file)
}

fn start_capture() {
    std::thread::spawn(|| {
        std::thread::sleep(MENU_CLOSE_DELAY);
        if let Ok(exe) = std::env::current_exe() {
            let _ = Command::new(exe).spawn().and_then(|mut child| child.wait());
        }
    });
}

fn autostart_path() -> Option<PathBuf> {
    Some(dirs::config_dir()?.join("autostart").join(format!("{}.desktop", crate::APP_ID)))
}

fn toggle_autostart() {
    let Some(path) = autostart_path() else { return };
    if path.exists() {
        let _ = std::fs::remove_file(&path);
    } else {
        write_autostart_entry(&path);
    }
}

/// Reescribe el inicio automático (si está activo) para que su descripción quede
/// en el idioma nuevo.
fn refresh_autostart_entry() {
    if let Some(path) = autostart_path().filter(|p| p.exists()) {
        write_autostart_entry(&path);
    }
}

fn write_autostart_entry(path: &std::path::Path) {
    let Ok(exe) = std::env::current_exe() else { return };
    if let Some(dir) = path.parent() {
        let _ = std::fs::create_dir_all(dir);
    }
    let entry = format!(
        "[Desktop Entry]\nType=Application\nName=Recorte\nComment={}\n\
         Exec={} {TRAY_ARG}\nIcon=applets-screenshooter\nX-GNOME-Autostart-enabled=true\n",
        i18n::t().autostart_comment,
        exe.display()
    );
    let _ = std::fs::write(path, entry);
}
