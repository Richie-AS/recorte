//! Idiomas de la interfaz: español, inglés y portugués.
//!
//! Por defecto se usa el idioma del sistema (`LANGUAGE`, `LC_ALL`, `LC_MESSAGES`,
//! `LANG`): español con `es_*`, portugués con `pt_*` e inglés en cualquier otro caso.
//! El usuario puede fijarlo desde el menú de la bandeja o con `recorte --idioma`.

use std::sync::atomic::{AtomicU8, Ordering};

use crate::settings;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Lang {
    Es,
    En,
    Pt,
}

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Es, Lang::En, Lang::Pt];

    /// Código que se guarda en los ajustes.
    pub fn code(self) -> &'static str {
        match self {
            Lang::Es => "es",
            Lang::En => "en",
            Lang::Pt => "pt",
        }
    }

    /// Nombre del idioma escrito en ese mismo idioma.
    pub fn native_name(self) -> &'static str {
        match self {
            Lang::Es => "Español",
            Lang::En => "English",
            Lang::Pt => "Português",
        }
    }

    pub fn from_code(code: &str) -> Option<Lang> {
        Lang::ALL.into_iter().find(|l| l.code() == code.trim().to_ascii_lowercase())
    }
}

/// Preferencia del usuario: seguir al sistema o un idioma fijo.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Choice {
    Auto,
    Fixed(Lang),
}

impl Choice {
    pub fn code(self) -> &'static str {
        match self {
            Choice::Auto => "auto",
            Choice::Fixed(lang) => lang.code(),
        }
    }

    pub fn from_code(code: &str) -> Option<Choice> {
        if code.trim().eq_ignore_ascii_case("auto") {
            Some(Choice::Auto)
        } else {
            Lang::from_code(code).map(Choice::Fixed)
        }
    }
}

/// Idioma según las variables de entorno de locale, en orden de prioridad.
pub fn detect(vars: &[Option<String>]) -> Lang {
    for value in vars.iter().flatten() {
        // LANGUAGE admite una lista "pt_BR:en"; vale la primera entrada.
        let first = value.split(':').next().unwrap_or("").trim();
        if first.is_empty() || first == "C" || first.starts_with("C.") || first == "POSIX" {
            continue;
        }
        let lower = first.to_ascii_lowercase();
        return if lower.starts_with("es") {
            Lang::Es
        } else if lower.starts_with("pt") {
            Lang::Pt
        } else {
            Lang::En
        };
    }
    Lang::En
}

fn system_lang() -> Lang {
    let vars = ["LANGUAGE", "LC_ALL", "LC_MESSAGES", "LANG"].map(|name| std::env::var(name).ok());
    detect(&vars)
}

pub fn resolve(choice: Choice) -> Lang {
    match choice {
        Choice::Auto => system_lang(),
        Choice::Fixed(lang) => lang,
    }
}

const UNSET: u8 = u8::MAX;
static CURRENT: AtomicU8 = AtomicU8::new(UNSET);

pub fn lang() -> Lang {
    match CURRENT.load(Ordering::Relaxed) {
        0 => Lang::Es,
        1 => Lang::En,
        2 => Lang::Pt,
        _ => {
            let lang = resolve(settings::load_language());
            set_current(lang);
            lang
        }
    }
}

fn set_current(lang: Lang) {
    let value = match lang {
        Lang::Es => 0,
        Lang::En => 1,
        Lang::Pt => 2,
    };
    CURRENT.store(value, Ordering::Relaxed);
}

/// Guarda la preferencia y la aplica de inmediato en este proceso.
pub fn set_choice(choice: Choice) {
    settings::save_language(choice);
    set_current(resolve(choice));
}

/// Textos del idioma actual.
pub fn t() -> &'static Texts {
    texts(lang())
}

pub fn texts(lang: Lang) -> &'static Texts {
    match lang {
        Lang::Es => &ES,
        Lang::En => &EN,
        Lang::Pt => &PT,
    }
}

/// Sustituye `{nombre}` por su valor en una plantilla.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    values
        .iter()
        .fold(template.to_owned(), |text, (key, value)| text.replace(&format!("{{{key}}}"), value))
}

pub struct Texts {
    pub tool_select: &'static str,
    pub tool_arrow: &'static str,
    pub tool_rectangle: &'static str,
    pub tool_marker: &'static str,
    pub tool_text: &'static str,
    pub tool_pixelate: &'static str,
    pub thin: &'static str,
    pub medium: &'static str,
    pub thick: &'static str,
    /// Mismo orden que `annotations::PALETTE`.
    pub colors: [&'static str; 8],
    pub color_and_size: &'static str,

    pub tip_undo: &'static str,
    pub tip_copy: &'static str,
    pub tip_save: &'static str,
    pub tip_about: &'static str,
    pub tip_close: &'static str,

    pub copy_title: &'static str,
    pub copy_body: &'static str,
    pub copy_button: &'static str,
    pub save_title: &'static str,
    /// `{dir}`: carpeta de destino.
    pub save_body: &'static str,
    pub save_button: &'static str,
    pub keep_editing: &'static str,

    /// `{version}`
    pub about_version: &'static str,
    pub about_description: &'static str,
    pub about_author: &'static str,
    pub about_source: &'static str,
    pub about_license: &'static str,
    pub about_license_value: &'static str,
    pub about_disclaimer: &'static str,
    pub about_credits: &'static str,
    pub about_close: &'static str,
    pub credit_ui: &'static str,
    pub credit_windows: &'static str,
    pub credit_graphics: &'static str,
    pub credit_portal: &'static str,
    pub credit_tray: &'static str,
    pub credit_clipboard: &'static str,
    pub credit_image: &'static str,
    pub credit_font: &'static str,

    pub tray_tooltip: &'static str,
    pub tray_capture: &'static str,
    pub tray_open_folder: &'static str,
    pub tray_autostart: &'static str,
    pub tray_settings: &'static str,
    pub tray_language: &'static str,
    pub tray_language_auto: &'static str,
    pub tray_quit: &'static str,
    pub autostart_comment: &'static str,

    /// `{err}`
    pub err_capture: &'static str,
    pub err_copy: &'static str,
    pub err_save: &'static str,
    pub err_tray: &'static str,
    pub err_shortcut: &'static str,
    pub err_clipboard: &'static str,
    pub shortcut_ok: &'static str,
    /// `{path}`
    pub saved_to: &'static str,
    /// `{lang}`
    pub language_set: &'static str,
    pub language_usage: &'static str,

    /// Nombre de los archivos guardados: "{prefijo} AAAA-MM-DD HH-MM-SS.png".
    pub file_prefix: &'static str,
    /// Subcarpeta de capturas dentro de la carpeta de imágenes.
    pub screenshots_folder: &'static str,
}

pub static ES: Texts = Texts {
    tool_select: "Selección",
    tool_arrow: "Flecha",
    tool_rectangle: "Rectángulo",
    tool_marker: "Marcador",
    tool_text: "Texto",
    tool_pixelate: "Pixelar",
    thin: "Fino",
    medium: "Medio",
    thick: "Grueso",
    colors: ["Rojo", "Naranja", "Amarillo", "Verde", "Azul", "Morado", "Blanco", "Negro"],
    color_and_size: "Color y grosor",

    tip_undo: "Deshacer (Ctrl+Z)",
    tip_copy: "Copiar y salir (Enter / Ctrl+C)",
    tip_save: "Guardar y salir (Ctrl+S)",
    tip_about: "Acerca de Recorte (F1)",
    tip_close: "Cerrar (Esc)",

    copy_title: "¿Copiar y salir?",
    copy_body: "La captura con sus anotaciones se copiará al portapapeles.",
    copy_button: "Copiar y salir  (Enter)",
    save_title: "¿Guardar y salir?",
    save_body: "La captura se guardará en {dir}.",
    save_button: "Guardar y salir  (Enter)",
    keep_editing: "Seguir editando  (Esc)",

    about_version: "Versión {version}",
    about_description: "Herramienta de captura y edición rápida de pantalla para GNOME Wayland",
    about_author: "Autor",
    about_source: "Código fuente",
    about_license: "Licencia",
    about_license_value: "GPL-3.0 o posterior",
    about_disclaimer: "Este programa es software libre: puede redistribuirlo y modificarlo bajo los \
                       términos de la Licencia Pública General de GNU. Se distribuye SIN NINGUNA GARANTÍA.",
    about_credits: "Créditos",
    about_close: "Cerrar  (Esc)",
    credit_ui: "interfaz",
    credit_windows: "ventanas",
    credit_graphics: "gráficos",
    credit_portal: "portal de capturas",
    credit_tray: "icono en la bandeja",
    credit_clipboard: "portapapeles",
    credit_image: "imagen y dibujo",
    credit_font: "Fuente Ubuntu",

    tray_tooltip: "Captura de pantalla (Ctrl+Impr Pant)",
    tray_capture: "Capturar pantalla",
    tray_open_folder: "Abrir carpeta de capturas",
    tray_autostart: "Iniciar con la sesión",
    tray_settings: "Configuración",
    tray_language: "Idioma",
    tray_language_auto: "Automático (sistema)",
    tray_quit: "Salir",
    autostart_comment: "Icono de Recorte en la bandeja",

    err_capture: "No se pudo capturar la pantalla ({err}).\nSi es la primera vez, ejecute \
                  «recorte --instalar-atajo» o abra Recorte desde el menú de aplicaciones para dar permiso.",
    err_copy: "No se pudo copiar la captura: {err}",
    err_save: "No se pudo guardar la captura: {err}",
    err_tray: "No se pudo mostrar el icono en la bandeja: {err}",
    err_shortcut: "No se pudo configurar el atajo: {err}",
    err_clipboard: "Error en el portapapeles: {err}",
    shortcut_ok: "Atajo Ctrl+Impr Pant configurado para Recorte.",
    saved_to: "Guardada en {path}",
    language_set: "Idioma establecido: {lang}.",
    language_usage: "Uso: recorte --idioma auto|es|en|pt",

    file_prefix: "Captura",
    screenshots_folder: "Capturas de pantalla",
};

pub static EN: Texts = Texts {
    tool_select: "Select",
    tool_arrow: "Arrow",
    tool_rectangle: "Rectangle",
    tool_marker: "Highlighter",
    tool_text: "Text",
    tool_pixelate: "Pixelate",
    thin: "Thin",
    medium: "Medium",
    thick: "Thick",
    colors: ["Red", "Orange", "Yellow", "Green", "Blue", "Purple", "White", "Black"],
    color_and_size: "Color and size",

    tip_undo: "Undo (Ctrl+Z)",
    tip_copy: "Copy and exit (Enter / Ctrl+C)",
    tip_save: "Save and exit (Ctrl+S)",
    tip_about: "About Recorte (F1)",
    tip_close: "Close (Esc)",

    copy_title: "Copy and exit?",
    copy_body: "The screenshot and its annotations will be copied to the clipboard.",
    copy_button: "Copy and exit  (Enter)",
    save_title: "Save and exit?",
    save_body: "The screenshot will be saved to {dir}.",
    save_button: "Save and exit  (Enter)",
    keep_editing: "Keep editing  (Esc)",

    about_version: "Version {version}",
    about_description: "Fast screenshot and annotation tool for GNOME Wayland",
    about_author: "Author",
    about_source: "Source code",
    about_license: "License",
    about_license_value: "GPL-3.0 or later",
    about_disclaimer: "This program is free software: you can redistribute it and/or modify it under \
                       the terms of the GNU General Public License. It comes with ABSOLUTELY NO WARRANTY.",
    about_credits: "Credits",
    about_close: "Close  (Esc)",
    credit_ui: "interface",
    credit_windows: "windows",
    credit_graphics: "graphics",
    credit_portal: "screenshot portal",
    credit_tray: "tray icon",
    credit_clipboard: "clipboard",
    credit_image: "image and drawing",
    credit_font: "Ubuntu font",

    tray_tooltip: "Screenshot (Ctrl+PrtSc)",
    tray_capture: "Take screenshot",
    tray_open_folder: "Open screenshots folder",
    tray_autostart: "Start at login",
    tray_settings: "Settings",
    tray_language: "Language",
    tray_language_auto: "Automatic (system)",
    tray_quit: "Quit",
    autostart_comment: "Recorte tray icon",

    err_capture: "Could not take the screenshot ({err}).\nIf this is the first time, run \
                  “recorte --install-shortcut” or open Recorte from the applications menu to grant permission.",
    err_copy: "Could not copy the screenshot: {err}",
    err_save: "Could not save the screenshot: {err}",
    err_tray: "Could not show the tray icon: {err}",
    err_shortcut: "Could not set up the shortcut: {err}",
    err_clipboard: "Clipboard error: {err}",
    shortcut_ok: "Ctrl+PrtSc shortcut configured for Recorte.",
    saved_to: "Saved to {path}",
    language_set: "Language set to {lang}.",
    language_usage: "Usage: recorte --language auto|es|en|pt",

    file_prefix: "Screenshot",
    screenshots_folder: "Screenshots",
};

pub static PT: Texts = Texts {
    tool_select: "Seleção",
    tool_arrow: "Seta",
    tool_rectangle: "Retângulo",
    tool_marker: "Marca-texto",
    tool_text: "Texto",
    tool_pixelate: "Pixelizar",
    thin: "Fino",
    medium: "Médio",
    thick: "Grosso",
    colors: ["Vermelho", "Laranja", "Amarelo", "Verde", "Azul", "Roxo", "Branco", "Preto"],
    color_and_size: "Cor e espessura",

    tip_undo: "Desfazer (Ctrl+Z)",
    tip_copy: "Copiar e sair (Enter / Ctrl+C)",
    tip_save: "Salvar e sair (Ctrl+S)",
    tip_about: "Sobre o Recorte (F1)",
    tip_close: "Fechar (Esc)",

    copy_title: "Copiar e sair?",
    copy_body: "A captura com suas anotações será copiada para a área de transferência.",
    copy_button: "Copiar e sair  (Enter)",
    save_title: "Salvar e sair?",
    save_body: "A captura será salva em {dir}.",
    save_button: "Salvar e sair  (Enter)",
    keep_editing: "Continuar editando  (Esc)",

    about_version: "Versão {version}",
    about_description: "Ferramenta rápida de captura e edição de tela para GNOME Wayland",
    about_author: "Autor",
    about_source: "Código-fonte",
    about_license: "Licença",
    about_license_value: "GPL-3.0 ou posterior",
    about_disclaimer: "Este programa é software livre: você pode redistribuí-lo e modificá-lo sob os \
                       termos da Licença Pública Geral GNU. É distribuído SEM NENHUMA GARANTIA.",
    about_credits: "Créditos",
    about_close: "Fechar  (Esc)",
    credit_ui: "interface",
    credit_windows: "janelas",
    credit_graphics: "gráficos",
    credit_portal: "portal de capturas",
    credit_tray: "ícone na bandeja",
    credit_clipboard: "área de transferência",
    credit_image: "imagem e desenho",
    credit_font: "Fonte Ubuntu",

    tray_tooltip: "Captura de tela (Ctrl+PrtSc)",
    tray_capture: "Capturar tela",
    tray_open_folder: "Abrir pasta de capturas",
    tray_autostart: "Iniciar com a sessão",
    tray_settings: "Configurações",
    tray_language: "Idioma",
    tray_language_auto: "Automático (sistema)",
    tray_quit: "Sair",
    autostart_comment: "Ícone do Recorte na bandeja",

    err_capture: "Não foi possível capturar a tela ({err}).\nSe for a primeira vez, execute \
                  “recorte --install-shortcut” ou abra o Recorte pelo menu de aplicativos para conceder permissão.",
    err_copy: "Não foi possível copiar a captura: {err}",
    err_save: "Não foi possível salvar a captura: {err}",
    err_tray: "Não foi possível mostrar o ícone na bandeja: {err}",
    err_shortcut: "Não foi possível configurar o atalho: {err}",
    err_clipboard: "Erro na área de transferência: {err}",
    shortcut_ok: "Atalho Ctrl+PrtSc configurado para o Recorte.",
    saved_to: "Salva em {path}",
    language_set: "Idioma definido: {lang}.",
    language_usage: "Uso: recorte --language auto|es|en|pt",

    file_prefix: "Captura de tela",
    screenshots_folder: "Capturas de tela",
};

#[cfg(test)]
mod tests {
    use super::*;

    fn vars(values: &[Option<&str>]) -> Vec<Option<String>> {
        values.iter().map(|v| v.map(str::to_owned)).collect()
    }

    #[test]
    fn detects_language_from_locale() {
        assert_eq!(detect(&vars(&[None, None, None, Some("es_PY.UTF-8")])), Lang::Es);
        assert_eq!(detect(&vars(&[None, None, None, Some("pt_BR.UTF-8")])), Lang::Pt);
        assert_eq!(detect(&vars(&[None, None, None, Some("de_DE.UTF-8")])), Lang::En);
        assert_eq!(detect(&vars(&[None, None, None, None])), Lang::En);
    }

    #[test]
    fn language_list_and_c_locale() {
        // LANGUAGE tiene prioridad y es una lista; "C" se ignora.
        assert_eq!(detect(&vars(&[Some("pt_BR:en"), None, None, Some("es_PY.UTF-8")])), Lang::Pt);
        assert_eq!(detect(&vars(&[Some(""), Some("C.UTF-8"), None, Some("es_ES.UTF-8")])), Lang::Es);
    }

    #[test]
    fn choice_codes_round_trip() {
        for choice in [Choice::Auto, Choice::Fixed(Lang::Es), Choice::Fixed(Lang::En), Choice::Fixed(Lang::Pt)] {
            assert_eq!(Choice::from_code(choice.code()), Some(choice));
        }
        assert_eq!(Choice::from_code("fr"), None);
    }

    #[test]
    fn templates_have_their_placeholders() {
        for lang in Lang::ALL {
            let t = texts(lang);
            assert!(t.save_body.contains("{dir}"), "{lang:?}");
            assert!(t.about_version.contains("{version}"), "{lang:?}");
            assert!(t.saved_to.contains("{path}"), "{lang:?}");
            assert!(t.language_set.contains("{lang}"), "{lang:?}");
            for err in [t.err_capture, t.err_copy, t.err_save, t.err_tray, t.err_shortcut, t.err_clipboard] {
                assert!(err.contains("{err}"), "{lang:?}: {err}");
            }
        }
    }

    #[test]
    fn fills_templates() {
        assert_eq!(fill("Hola {name}, {name}", &[("name", "Ana")]), "Hola Ana, Ana");
    }
}
