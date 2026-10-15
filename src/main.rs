mod about;
mod annotations;
mod capture;
mod clipboard;
mod save;
mod settings;
mod shortcut;
mod toolbar;
mod tray;

/// Identificador de la aplicación: coincide con el nombre del archivo `.desktop`.
/// El portal guarda el permiso de captura con este nombre.
pub const APP_ID: &str = "io.github.Richie_AS.Recorte";

use std::num::NonZeroU32;
use std::sync::Arc;

use annotations::{Annotation, PixelMap, Style, Tool};
use toolbar::ToolbarAction;
use egui::{
    Color32, CursorIcon, Event, FontId, Key, Pos2, Rect, Stroke, StrokeKind,
    TextureHandle, TextureOptions, Vec2, ViewportId,
};
use egui_wgpu::winit::Painter;
use egui_wgpu::{RendererOptions, WgpuConfiguration, WgpuSetup};
use image::RgbaImage;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::monitor::MonitorHandle;
use winit::platform::wayland::WindowAttributesExtWayland;
use winit::window::{Fullscreen, Window, WindowId};

/// Opacidad del velo oscuro sobre la captura congelada.
const DIM_ALPHA: u8 = 110;
/// Azul de acento de GNOME.
const ACCENT: Color32 = Color32::from_rgb(0x35, 0x84, 0xe4);

/// Texto en edición, antes de confirmarlo con `Enter`.
struct TextEdit {
    pos: Pos2,
    text: String,
}

/// Cómo termina la aplicación con la imagen final.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Finish {
    Copy,
    Save,
}

/// Estado compartido por todas las ventanas, en coordenadas lógicas globales.
struct Shared {
    capture: RgbaImage,
    /// Captura completa pixelada en bloques alineados a su origen.
    pixelated: RgbaImage,
    /// Caja que engloba todos los monitores (equivale al PNG completo del portal).
    bounds: Rect,
    /// Área lógica global de cada monitor.
    monitors: Vec<Rect>,
    selection: Option<Rect>,
    drag_start: Option<Pos2>,
    tool: Tool,
    /// Color y grosor para las anotaciones nuevas.
    style: Style,
    annotations: Vec<Annotation>,
    /// Anotación que se está arrastrando.
    drawing: Option<Annotation>,
    text_edit: Option<TextEdit>,
    /// Diálogo de confirmación abierto ("¿Copiar y salir?" / "¿Guardar y salir?").
    confirm: Option<Finish>,
    /// Diálogo "Acerca de" abierto.
    show_about: bool,
    /// Tamaño de la barra medido en el último frame.
    toolbar_size: Vec2,
}

impl Shared {
    /// Selección recortada a los límites del escritorio, si tiene área.
    fn valid_selection(&self) -> Option<Rect> {
        let rect = self.selection?.intersect(self.bounds);
        (rect.width() >= 1.0 && rect.height() >= 1.0).then_some(rect)
    }

    /// Píxeles de la captura por unidad lógica.
    fn capture_scale(&self) -> Vec2 {
        let (w, h) = self.capture.dimensions();
        Vec2::new(w as f32 / self.bounds.width(), h as f32 / self.bounds.height())
    }

    /// Región en píxeles de la captura (x, y, ancho, alto) para un área lógica.
    fn capture_region(&self, region: Rect) -> (u32, u32, u32, u32) {
        let (img_w, img_h) = self.capture.dimensions();
        let scale = self.capture_scale();
        let min = (region.min - self.bounds.min) * scale;
        let max = (region.max - self.bounds.min) * scale;

        let x = (min.x.round().max(0.0) as u32).min(img_w - 1);
        let y = (min.y.round().max(0.0) as u32).min(img_h - 1);
        let w = ((max.x.round() as u32).saturating_sub(x)).clamp(1, img_w - x);
        let h = ((max.y.round() as u32).saturating_sub(y)).clamp(1, img_h - y);
        (x, y, w, h)
    }

    /// Posición de la barra flotante, oculta mientras se traza la selección.
    fn toolbar_rect(&self) -> Option<Rect> {
        let selecting = self.drag_start.is_some() && self.drawing.is_none();
        if selecting {
            return None;
        }
        let selection = self.valid_selection()?;
        Some(toolbar::place(selection, &self.monitors, self.toolbar_size))
    }

    /// Hay un diálogo abierto que bloquea la edición.
    fn dialog_open(&self) -> bool {
        self.confirm.is_some() || self.show_about
    }

    /// Pide confirmación para terminar, si hay algo seleccionado.
    fn ask_finish(&mut self, finish: Finish) {
        self.commit_text();
        if self.valid_selection().is_some() {
            self.confirm = Some(finish);
        }
    }

    /// Guarda el texto en edición como anotación.
    fn commit_text(&mut self) {
        if let Some(TextEdit { pos, text }) = self.text_edit.take() {
            let annotation = Annotation::Text { pos, text, style: self.style };
            if !annotation.is_degenerate() {
                self.annotations.push(annotation);
            }
        }
    }

    /// Imagen final: la selección con todas las anotaciones aplicadas.
    fn export(&self) -> Option<RgbaImage> {
        let selection = self.valid_selection()?;
        let (x, y, w, h) = self.capture_region(selection);
        let crop = image::imageops::crop_imm(&self.capture, x, y, w, h).to_image();
        let map = PixelMap {
            bounds_min: self.bounds.min,
            scale: self.capture_scale(),
            crop_origin: (x, y),
        };
        Some(annotations::render(crop, &self.annotations, &map, &self.pixelated))
    }
}

/// Qué debe hacer la aplicación tras un frame.
#[derive(PartialEq)]
enum Action {
    None,
    /// El estado compartido cambió: hay que repintar todas las ventanas.
    RedrawAll,
    Exit,
    Finish(Finish),
}

/// Una ventana a pantalla completa sobre un monitor concreto.
struct MonitorView {
    window: Arc<Window>,
    ctx: egui::Context,
    state: egui_winit::State,
    painter: Painter,
    texture: TextureHandle,
    pixel_texture: TextureHandle,
    /// Área del monitor en coordenadas lógicas globales.
    logical_rect: Rect,
}

struct App {
    shared: Shared,
    views: Vec<MonitorView>,
    /// Imagen final y qué hacer con ella cuando se cierre el event loop.
    result: Option<(Finish, RgbaImage)>,
}

impl App {
    fn new(capture: RgbaImage) -> Self {
        let pixelated = annotations::pixelate(&capture);
        Self {
            shared: Shared {
                capture,
                pixelated,
                bounds: Rect::NOTHING,
                monitors: Vec::new(),
                selection: None,
                drag_start: None,
                tool: Tool::Select,
                style: settings::load_style(),
                annotations: Vec::new(),
                drawing: None,
                text_edit: None,
                confirm: None,
                show_about: false,
                toolbar_size: toolbar::ESTIMATED_SIZE,
            },
            views: Vec::new(),
            result: None,
        }
    }

    fn create_views(&mut self, event_loop: &ActiveEventLoop) {
        let monitors: Vec<MonitorHandle> = event_loop.available_monitors().collect();
        let layouts: Vec<(Option<MonitorHandle>, Rect)> = if monitors.is_empty() {
            let (w, h) = self.shared.capture.dimensions();
            vec![(None, Rect::from_min_size(Pos2::ZERO, Vec2::new(w as f32, h as f32)))]
        } else {
            monitors
                .into_iter()
                .map(|m| {
                    let rect = monitor_logical_rect(&m);
                    (Some(m), rect)
                })
                .collect()
        };

        self.shared.monitors = layouts.iter().map(|(_, r)| *r).collect();
        self.shared.bounds = layouts
            .iter()
            .map(|(_, r)| *r)
            .reduce(|a, b| a.union(b))
            .expect("al menos un monitor");

        for (monitor, logical_rect) in layouts {
            let attributes = Window::default_attributes()
                .with_title("Recorte")
                .with_name(APP_ID, APP_ID)
                .with_decorations(false)
                .with_fullscreen(Some(Fullscreen::Borderless(monitor)));
            let window = Arc::new(
                event_loop
                    .create_window(attributes)
                    .expect("no se pudo crear la ventana"),
            );

            let ctx = egui::Context::default();
            let mut painter = pollster::block_on(Painter::new(
                ctx.clone(),
                wgpu_configuration(),
                false,
                RendererOptions::default(),
            ));
            pollster::block_on(painter.set_window(ViewportId::ROOT, Some(window.clone())))
                .expect("no se pudo inicializar wgpu");

            let state = egui_winit::State::new(
                ctx.clone(),
                ViewportId::ROOT,
                event_loop,
                Some(window.scale_factor() as f32),
                None,
                painter.max_texture_side(),
            );

            let region = self.shared.capture_region(logical_rect);
            let texture = load_region(&ctx, "captura", &self.shared.capture, region, TextureOptions::LINEAR);
            // NEAREST para que los bloques del pixelado se vean nítidos.
            let pixel_texture =
                load_region(&ctx, "pixelado", &self.shared.pixelated, region, TextureOptions::NEAREST);

            window.request_redraw();
            self.views.push(MonitorView {
                window,
                ctx,
                state,
                painter,
                texture,
                pixel_texture,
                logical_rect,
            });
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.views.is_empty() {
            self.create_views(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(view) = self.views.iter_mut().find(|v| v.window.id() == id) else {
            return;
        };

        let response = view.state.on_window_event(&view.window, &event);
        if response.repaint {
            view.window.request_redraw();
        }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => {
                if let (Some(w), Some(h)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height)) {
                    view.painter.on_window_resized(ViewportId::ROOT, w, h);
                }
            }
            WindowEvent::RedrawRequested => match redraw(view, &mut self.shared) {
                Action::None => {}
                Action::RedrawAll => {
                    for other in &self.views {
                        if other.window.id() != id {
                            other.window.request_redraw();
                        }
                    }
                }
                Action::Exit => event_loop.exit(),
                Action::Finish(finish) => {
                    if let Some(image) = self.shared.export() {
                        self.result = Some((finish, image));
                        event_loop.exit();
                    }
                }
            },
            _ => {}
        }
    }
}

/// Procesa la entrada y dibuja un frame de una ventana.
fn redraw(view: &mut MonitorView, shared: &mut Shared) -> Action {
    let raw_input = view.state.take_egui_input(&view.window);
    // Cualquier entrada puede cambiar el estado compartido que ven las demás ventanas.
    let mut action = if raw_input.events.is_empty() { Action::None } else { Action::RedrawAll };
    let texture_id = view.texture.id();
    let pixel_texture_id = view.pixel_texture.id();
    let view_rect = view.logical_rect;
    let offset = view_rect.min.to_vec2();

    let mut output = view.ctx.run_ui(raw_input, |ui| {
        // Con el diálogo abierto el ratón solo actúa sobre sus botones.
        if !shared.dialog_open() {
            handle_pointer(ui, shared, offset);
        }
        if let Some(keyboard_action) = handle_keyboard(ui, shared) {
            action = keyboard_action;
        }

        let screen = ui.ctx().content_rect();
        let painter = ui.painter();
        let full_uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));
        painter.image(texture_id, screen, full_uv, Color32::WHITE);

        let dim = Color32::from_black_alpha(DIM_ALPHA);
        match shared.valid_selection() {
            Some(global) => {
                let local = global.translate(-offset);
                for rect in surrounding_rects(screen, local) {
                    painter.rect_filled(rect, 0.0, dim);
                }

                let clipped = painter.with_clip_rect(local);
                for annotation in shared.annotations.iter().chain(&shared.drawing) {
                    annotation.paint(&clipped, view_rect, pixel_texture_id);
                }
                if let Some(edit) = &shared.text_edit {
                    let caret = format!("{}|", edit.text);
                    Annotation::Text { pos: edit.pos, text: caret, style: shared.style }
                        .paint(&clipped, view_rect, pixel_texture_id);
                }

                painter.rect_stroke(local, 0.0, Stroke::new(2.0, ACCENT), StrokeKind::Outside);
                draw_size_label(painter, screen, local, global.size() * shared.capture_scale());
            }
            None => {
                painter.rect_filled(screen, 0.0, dim);
            }
        }
        if let Some(bar) = shared.toolbar_rect()
            && bar.intersects(view_rect)
        {
            let can_undo = !shared.annotations.is_empty();
            let previous_style = shared.style;
            let (clicked, size) =
                toolbar::show(ui.ctx(), bar.min - offset, shared.tool, can_undo, &mut shared.style);
            shared.toolbar_size = size;
            if shared.style != previous_style {
                settings::save_style(shared.style);
                action = Action::RedrawAll;
            }
            if let Some(clicked) = clicked {
                action = apply_toolbar_action(shared, clicked);
            }
        }

        if let Some(finish) = shared.confirm
            && let Some(dialog_action) = show_confirm_dialog(ui.ctx(), shared, finish)
        {
            action = dialog_action;
        }
        if shared.show_about && about::show(ui.ctx()) {
            shared.show_about = false;
            action = Action::RedrawAll;
        }

        let over_ui = ui.input(|i| i.pointer.latest_pos()).is_some_and(|p| {
            shared.dialog_open() || ui.ctx().is_pointer_over_egui() || shared.toolbar_rect().is_some_and(|bar| bar.contains(p + offset))
        });
        // Sin selección, cualquier herramienta actúa como Selección.
        let tool = if shared.valid_selection().is_some() { shared.tool } else { Tool::Select };
        let cursor = match tool {
            _ if over_ui => CursorIcon::Default,
            // Cruz para las herramientas que marcan un área con precisión.
            Tool::Rectangle | Tool::Marker | Tool::Pixelate => CursorIcon::Crosshair,
            Tool::Text => CursorIcon::Text,
            Tool::Select | Tool::Arrow => CursorIcon::Default,
        };
        ui.ctx().set_cursor_icon(cursor);
    });

    view.state
        .handle_platform_output(&view.window, std::mem::take(&mut output.platform_output));
    let primitives = view.ctx.tessellate(output.shapes, output.pixels_per_point);
    view.painter.paint_and_update_textures(
        ViewportId::ROOT,
        output.pixels_per_point,
        [0.0, 0.0, 0.0, 1.0],
        &primitives,
        &mut output.textures_delta,
        Vec::new(),
        &view.window,
    );

    action
}

/// Selección o dibujo con el ratón, según la herramienta activa.
fn handle_pointer(ui: &egui::Ui, shared: &mut Shared, offset: Vec2) {
    let (pressed, down, released, pos) = ui.input(|i| {
        (
            i.pointer.primary_pressed(),
            i.pointer.primary_down(),
            i.pointer.primary_released(),
            i.pointer.latest_pos(),
        )
    });

    // Los clics sobre la barra son para sus botones.
    let on_toolbar = pos.is_some_and(|p| shared.toolbar_rect().is_some_and(|bar| bar.contains(p + offset)));
    if pressed && (on_toolbar || ui.ctx().is_pointer_over_egui()) {
        return;
    }

    // Sin selección, cualquier herramienta empieza seleccionando.
    let selection = shared.valid_selection();
    let tool = if selection.is_some() { shared.tool } else { Tool::Select };

    if let Some(global) = pos.map(|p| p + offset) {
        match (tool, selection) {
            (Tool::Select, _) | (_, None) => {
                if pressed {
                    shared.commit_text();
                    shared.drag_start = Some(global);
                    shared.selection = Some(Rect::from_two_pos(global, global));
                } else if down && let Some(start) = shared.drag_start {
                    shared.selection = Some(Rect::from_two_pos(start, global));
                }
            }
            (Tool::Text, Some(selection)) => {
                // Un clic mientras se escribe solo confirma el texto; el siguiente abre uno nuevo.
                if pressed && shared.text_edit.is_some() {
                    shared.commit_text();
                } else if pressed {
                    shared.text_edit = Some(TextEdit { pos: selection.clamp(global), text: String::new() });
                }
            }
            (_, Some(selection)) => {
                let global = selection.clamp(global);
                if pressed {
                    shared.commit_text();
                    shared.drag_start = Some(global);
                    shared.drawing = Annotation::start(tool, global, shared.style);
                } else if down
                    && let (Some(start), Some(drawing)) = (shared.drag_start, &mut shared.drawing)
                {
                    drawing.drag_to(start, global);
                }
            }
        }
    }

    if released {
        shared.drag_start = None;
        if let Some(drawing) = shared.drawing.take()
            && !drawing.is_degenerate()
        {
            shared.annotations.push(drawing);
        }
    }
}

/// Atajos de teclado y escritura de texto. Devuelve la acción a ejecutar, si hay.
fn handle_keyboard(ui: &egui::Ui, shared: &mut Shared) -> Option<Action> {
    let events = ui.input(|i| i.events.clone());
    let mut action = None;

    for event in events {
        if shared.show_about {
            if let Event::Key { key: Key::Escape, pressed: true, .. } = event {
                shared.show_about = false;
            }
            continue;
        }

        if let Some(finish) = shared.confirm {
            match event {
                Event::Key { key: Key::Enter, pressed: true, .. } => action = Some(Action::Finish(finish)),
                Event::Key { key: Key::Escape, pressed: true, .. } => shared.confirm = None,
                _ => {}
            }
            continue;
        }

        if let Some(edit) = &mut shared.text_edit {
            match event {
                Event::Text(text) => edit.text.push_str(&text),
                Event::Key { key: Key::Backspace, pressed: true, .. } => {
                    edit.text.pop();
                }
                Event::Key { key: Key::Enter, pressed: true, .. } => shared.commit_text(),
                Event::Key { key: Key::Escape, pressed: true, .. } => shared.text_edit = None,
                Event::Copy => {
                    shared.commit_text();
                    action = Some(Action::Finish(Finish::Copy));
                }
                _ => {}
            }
            continue;
        }

        match event {
            Event::Key { key: Key::Escape, pressed: true, .. } => action = Some(Action::Exit),
            Event::Key { key: Key::F1, pressed: true, .. } => shared.show_about = true,
            // Enter pide confirmación; Ctrl+C copia directamente.
            Event::Key { key: Key::Enter, pressed: true, .. } => shared.ask_finish(Finish::Copy),
            Event::Copy => action = Some(Action::Finish(Finish::Copy)),
            Event::Key { key, pressed: true, modifiers, .. } if modifiers.command => match key {
                Key::C => action = Some(Action::Finish(Finish::Copy)),
                Key::S => shared.ask_finish(Finish::Save),
                Key::Z => {
                    shared.annotations.pop();
                }
                _ => {}
            },
            Event::Key { key, pressed: true, modifiers, .. } if modifiers.is_none() => {
                if let Some(tool) = Tool::ALL.into_iter().find(|t| t.key() == key) {
                    shared.tool = tool;
                }
            }
            _ => {}
        }
    }
    action
}

/// Acción de un botón de la barra flotante.
fn apply_toolbar_action(shared: &mut Shared, clicked: ToolbarAction) -> Action {
    match clicked {
        ToolbarAction::SetTool(tool) => {
            shared.commit_text();
            shared.tool = tool;
        }
        ToolbarAction::Undo => {
            shared.annotations.pop();
        }
        ToolbarAction::Copy => shared.ask_finish(Finish::Copy),
        ToolbarAction::Save => shared.ask_finish(Finish::Save),
        ToolbarAction::About => {
            shared.commit_text();
            shared.show_about = true;
        }
        ToolbarAction::Close => return Action::Exit,
    }
    Action::RedrawAll
}

/// Diálogo de confirmación antes de copiar o guardar y salir.
fn show_confirm_dialog(ctx: &egui::Context, shared: &mut Shared, finish: Finish) -> Option<Action> {
    let (title, body, confirm) = match finish {
        Finish::Copy => (
            "¿Copiar y salir?",
            "La captura con sus anotaciones se copiará al portapapeles.".to_owned(),
            "Copiar y salir  (Enter)",
        ),
        Finish::Save => (
            "¿Guardar y salir?",
            format!("La captura se guardará en {}.", save::screenshots_dir().display()),
            "Guardar y salir  (Enter)",
        ),
    };

    let mut action = None;
    egui::Modal::new(egui::Id::new("confirmar")).show(ctx, |ui| {
        ui.set_width(340.0);
        ui.heading(title);
        ui.label(body);
        ui.add_space(12.0);
        ui.horizontal(|ui| {
            if ui.button(confirm).clicked() {
                action = Some(Action::Finish(finish));
            }
            if ui.button("Seguir editando  (Esc)").clicked() {
                shared.confirm = None;
                action = Some(Action::RedrawAll);
            }
        });
    });
    action
}

/// Los cuatro rectángulos de `screen` que quedan fuera de `hole`.
fn surrounding_rects(screen: Rect, hole: Rect) -> [Rect; 4] {
    let hole = hole.intersect(screen);
    if hole.is_negative() {
        return [screen, Rect::NOTHING, Rect::NOTHING, Rect::NOTHING];
    }
    [
        Rect::from_min_max(screen.min, Pos2::new(screen.max.x, hole.min.y)),
        Rect::from_min_max(Pos2::new(screen.min.x, hole.max.y), screen.max),
        Rect::from_min_max(Pos2::new(screen.min.x, hole.min.y), Pos2::new(hole.min.x, hole.max.y)),
        Rect::from_min_max(Pos2::new(hole.max.x, hole.min.y), Pos2::new(screen.max.x, hole.max.y)),
    ]
}

/// Etiqueta con el tamaño real en píxeles, encima de la selección (o dentro si no cabe).
fn draw_size_label(painter: &egui::Painter, screen: Rect, selection: Rect, pixels: Vec2) {
    let text = format!("{} × {}", pixels.x.round(), pixels.y.round());
    let galley = painter.layout_no_wrap(text, FontId::proportional(13.0), Color32::WHITE);
    let padding = Vec2::new(6.0, 3.0);
    let size = galley.size() + padding * 2.0;

    let mut pos = selection.left_top() - Vec2::new(0.0, size.y + 4.0);
    if pos.y < screen.min.y {
        pos = selection.left_top() + Vec2::splat(4.0);
    }
    let rect = Rect::from_min_size(pos, size);
    painter.rect_filled(rect, 4.0, Color32::from_black_alpha(180));
    painter.galley(rect.min + padding, galley, Color32::WHITE);
}

/// Sube a la GPU una región de una imagen como textura de egui.
fn load_region(
    ctx: &egui::Context,
    name: &str,
    image: &RgbaImage,
    (x, y, w, h): (u32, u32, u32, u32),
    options: TextureOptions,
) -> TextureHandle {
    let crop = image::imageops::crop_imm(image, x, y, w, h).to_image();
    let size = [crop.width() as usize, crop.height() as usize];
    let color_image = egui::ColorImage::from_rgba_unmultiplied(size, crop.as_raw());
    ctx.load_texture(name, color_image, options)
}

/// Rectángulo del monitor en coordenadas lógicas globales.
fn monitor_logical_rect(monitor: &MonitorHandle) -> Rect {
    let scale = monitor.scale_factor();
    let pos = monitor.position().to_logical::<f32>(scale);
    let size = monitor.size().to_logical::<f32>(scale);
    Rect::from_min_size(Pos2::new(pos.x, pos.y), Vec2::new(size.width, size.height))
}

/// Usa la GPU integrada salvo que `WGPU_POWER_PREF` diga otra cosa: alcanza de
/// sobra para dibujar la captura y evita GPUs dedicadas antiguas que se cuelgan.
fn wgpu_configuration() -> WgpuConfiguration {
    let mut config = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(setup) = &mut config.wgpu_setup
        && egui_wgpu::wgpu::PowerPreference::from_env().is_none()
    {
        setup.power_preference = egui_wgpu::wgpu::PowerPreference::LowPower;
    }
    config
}

/// Muestra el error como notificación del escritorio (al lanzarse con el atajo no
/// hay terminal) y también en la salida de errores.
fn report_error(message: &str) {
    eprintln!("{message}");
    let _ = std::process::Command::new("notify-send")
        .args(["--app-name=Recorte", "--icon=applets-screenshooter", "Recorte", message])
        .status();
}

/// Carpeta de la sesión para los archivos de bloqueo.
fn runtime_dir() -> std::path::PathBuf {
    std::env::var_os("XDG_RUNTIME_DIR").map(std::path::PathBuf::from).unwrap_or_else(std::env::temp_dir)
}

/// Evita abrir una segunda captura si ya hay una en curso (por ejemplo, al pulsar
/// el atajo varias veces). El bloqueo se libera solo al terminar el proceso.
fn acquire_single_instance() -> Option<std::fs::File> {
    let file = std::fs::File::create(runtime_dir().join("recorte.lock")).ok()?;
    match file.try_lock() {
        Ok(()) => Some(file),
        Err(std::fs::TryLockError::WouldBlock) => None,
        // Si no se puede bloquear por otro motivo, mejor seguir que no capturar.
        Err(std::fs::TryLockError::Error(_)) => Some(file),
    }
}

fn main() {
    match std::env::args().nth(1).as_deref() {
        Some(clipboard::DAEMON_ARG) => {
            if let Err(err) = clipboard::run_daemon() {
                eprintln!("Error en el portapapeles: {err}");
                std::process::exit(1);
            }
            return;
        }
        Some(shortcut::INSTALL_ARG) => {
            match shortcut::install() {
                Ok(()) => println!("Atajo Ctrl+Impr Pant configurado para Recorte."),
                Err(err) => {
                    eprintln!("No se pudo configurar el atajo: {err}");
                    std::process::exit(1);
                }
            }
            return;
        }
        Some(tray::TRAY_ARG) => {
            if let Err(err) = tray::run() {
                report_error(&format!("No se pudo mostrar el icono en la bandeja: {err}"));
                std::process::exit(1);
            }
            return;
        }
        _ => {}
    }

    // Al abrir Recorte el icono queda en la bandeja hasta elegir "Salir".
    tray::ensure_running();

    let Some(_lock) = acquire_single_instance() else {
        return;
    };

    let capture = match capture::capture_screen() {
        Ok(image) => image,
        Err(err) => {
            report_error(&format!(
                "No se pudo capturar la pantalla ({err}).\n\
                 Si es la primera vez, ejecute «recorte --instalar-atajo» \
                 o abra Recorte desde el menú de aplicaciones para dar permiso."
            ));
            std::process::exit(1);
        }
    };

    let event_loop = EventLoop::new().expect("no se pudo crear el event loop");
    let mut app = App::new(capture);
    event_loop.run_app(&mut app).expect("error en el event loop");

    if let Some((finish, image)) = app.result.take() {
        // Cerrar las ventanas antes de copiar o guardar.
        drop(app);
        let result = match finish {
            Finish::Copy => clipboard::copy_in_background(&image),
            Finish::Save => save::save_png(&image).map(|path| println!("Guardada en {}", path.display())),
        };
        if let Err(err) = result {
            let what = match finish {
                Finish::Copy => "copiar la captura",
                Finish::Save => "guardar la captura",
            };
            report_error(&format!("No se pudo {what}: {err}"));
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(key: Key) -> Event {
        Event::Key { key, physical_key: None, pressed: true, repeat: false, modifiers: Default::default() }
    }

    fn run_keys(app: &mut App, events: Vec<Event>) -> Option<Action> {
        let ctx = egui::Context::default();
        let input = egui::RawInput { events, ..Default::default() };
        let mut action = None;
        ctx.run_ui(input, |ui| action = handle_keyboard(ui, &mut app.shared))
            .drop_without_applying_deltas();
        action
    }

    #[test]
    fn app_id_is_valid_for_the_portal() {
        assert!(ashpd::AppID::try_from(APP_ID).is_ok());
    }

    #[test]
    fn escape_cancels_text_without_exiting() {
        let mut app = App::new(RgbaImage::new(8, 8));
        app.shared.text_edit = Some(TextEdit { pos: Pos2::ZERO, text: String::new() });

        let action = run_keys(&mut app, vec![Event::Text("hola".into()), key(Key::Escape)]);
        assert!(app.shared.text_edit.is_none());
        assert!(app.shared.annotations.is_empty());
        assert!(action.is_none());
    }

    #[test]
    fn enter_asks_for_confirmation_before_copying() {
        let mut app = App::new(RgbaImage::new(8, 8));
        app.shared.bounds = Rect::from_min_size(Pos2::ZERO, Vec2::splat(8.0));
        app.shared.selection = Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(4.0)));

        assert!(run_keys(&mut app, vec![key(Key::Enter)]).is_none());
        assert_eq!(app.shared.confirm, Some(Finish::Copy));

        // Esc cancela el diálogo sin salir.
        assert!(run_keys(&mut app, vec![key(Key::Escape)]).is_none());
        assert!(app.shared.confirm.is_none());

        run_keys(&mut app, vec![key(Key::Enter)]);
        assert!(run_keys(&mut app, vec![key(Key::Enter)]) == Some(Action::Finish(Finish::Copy)));
    }
}
