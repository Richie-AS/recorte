//! Diálogo "Acerca de". Los datos salen de `Cargo.toml`.

const NAME: &str = "Recorte";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
const LICENSE: &str = env!("CARGO_PKG_LICENSE");

const CREDITS: &[(&str, &str)] = &[
    ("egui", "interfaz"),
    ("winit", "ventanas"),
    ("wgpu", "gráficos"),
    ("ashpd", "portal de capturas"),
    ("arboard", "portapapeles"),
    ("image · tiny-skia · ab_glyph", "imagen y dibujo"),
    ("Fuente Ubuntu", "Ubuntu Font Licence 1.0"),
];

/// Dibuja el diálogo. Devuelve `true` si el usuario lo cerró con el botón.
pub fn show(ctx: &egui::Context) -> bool {
    let mut close = false;
    egui::Modal::new(egui::Id::new("acerca_de")).show(ctx, |ui| {
        ui.set_width(380.0);
        ui.vertical_centered(|ui| {
            ui.heading(NAME);
            ui.label(format!("Versión {VERSION}"));
            ui.add_space(4.0);
            ui.label(DESCRIPTION);
        });
        ui.add_space(12.0);

        egui::Grid::new("datos").num_columns(2).spacing([12.0, 6.0]).show(ui, |ui| {
            // Cargo separa varios autores con ':'.
            if !AUTHORS.is_empty() {
                ui.strong("Autor");
                ui.label(AUTHORS.replace(':', ", "));
                ui.end_row();
            }
            if !REPOSITORY.is_empty() {
                ui.strong("Código fuente");
                ui.hyperlink(REPOSITORY);
                ui.end_row();
            }
            ui.strong("Licencia");
            ui.label(LICENSE.replace("-or-later", " o posterior"));
            ui.end_row();
        });

        ui.add_space(8.0);
        ui.small(
            "Este programa es software libre: puede redistribuirlo y modificarlo bajo los \
             términos de la Licencia Pública General de GNU. Se distribuye SIN NINGUNA GARANTÍA.",
        );

        ui.add_space(8.0);
        ui.collapsing("Créditos", |ui| {
            for (name, role) in CREDITS {
                ui.label(format!("{name} — {role}"));
            }
        });

        ui.add_space(12.0);
        ui.vertical_centered(|ui| {
            if ui.button("Cerrar  (Esc)").clicked() {
                close = true;
            }
        });
    });
    close
}
