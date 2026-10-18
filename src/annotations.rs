//! Herramientas de dibujo: modelo de anotaciones, vista previa con egui y
//! renderizado final sobre la imagen recortada.
//!
//! Las anotaciones se guardan en coordenadas lógicas globales, igual que la selección.

use ab_glyph::{Font, FontRef, PxScale, ScaleFont, point};
use egui::{Align2, Color32, FontId, Pos2, Rect, Shape, Stroke, StrokeKind, TextureId, Vec2};
use image::RgbaImage;
use tiny_skia::{FillRule, LineCap, LineJoin, Paint, PathBuilder, Pixmap, Transform};

/// Paleta de colores de GNOME disponible en la barra. Los nombres están en
/// `i18n::Texts::colors`, en el mismo orden.
pub const PALETTE: [Color32; 8] = [
    Color32::from_rgb(0xe0, 0x1b, 0x24), // rojo
    Color32::from_rgb(0xff, 0x78, 0x00), // naranja
    Color32::from_rgb(0xf6, 0xd3, 0x2d), // amarillo
    Color32::from_rgb(0x33, 0xd1, 0x7a), // verde
    Color32::from_rgb(0x35, 0x84, 0xe4), // azul
    Color32::from_rgb(0x91, 0x41, 0xac), // morado
    Color32::from_rgb(0xff, 0xff, 0xff), // blanco
    Color32::from_rgb(0x00, 0x00, 0x00), // negro
];
/// Opacidad del marcador translúcido.
const MARKER_ALPHA: u8 = 110;
/// Lado del bloque de pixelado, en píxeles de la captura.
pub const PIXEL_BLOCK: u32 = 8;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Thickness {
    Thin,
    Medium,
    Thick,
}

impl Thickness {
    pub const ALL: [Thickness; 3] = [Thickness::Thin, Thickness::Medium, Thickness::Thick];

    /// Nombre visible, en el idioma de la interfaz.
    pub fn name(self) -> &'static str {
        let t = crate::i18n::t();
        match self {
            Thickness::Thin => t.thin,
            Thickness::Medium => t.medium,
            Thickness::Thick => t.thick,
        }
    }

    /// Identificador fijo para los ajustes (no depende del idioma).
    pub fn id(self) -> &'static str {
        match self {
            Thickness::Thin => "fino",
            Thickness::Medium => "medio",
            Thickness::Thick => "grueso",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|t| t.id().eq_ignore_ascii_case(id.trim()))
    }
}

/// Color y grosor con que se dibuja una anotación.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Style {
    pub color: Color32,
    pub thickness: Thickness,
}

impl Default for Style {
    fn default() -> Self {
        Self { color: PALETTE[0], thickness: Thickness::Medium }
    }
}

impl Style {
    pub fn stroke_width(self) -> f32 {
        match self.thickness {
            Thickness::Thin => 2.0,
            Thickness::Medium => 3.0,
            Thickness::Thick => 6.0,
        }
    }

    pub fn marker_width(self) -> f32 {
        match self.thickness {
            Thickness::Thin => 12.0,
            Thickness::Medium => 18.0,
            Thickness::Thick => 28.0,
        }
    }

    pub fn text_size(self) -> f32 {
        match self.thickness {
            Thickness::Thin => 16.0,
            Thickness::Medium => 22.0,
            Thickness::Thick => 32.0,
        }
    }

    /// El color activo en versión translúcida (premultiplicada) para el marcador.
    pub fn marker_color(self) -> Color32 {
        let [r, g, b, _] = self.color.to_srgba_unmultiplied();
        Color32::from_rgba_unmultiplied(r, g, b, MARKER_ALPHA)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Select,
    Arrow,
    Rectangle,
    Marker,
    Text,
    Pixelate,
}

impl Tool {
    pub const ALL: [Tool; 6] = [
        Tool::Select,
        Tool::Arrow,
        Tool::Rectangle,
        Tool::Marker,
        Tool::Text,
        Tool::Pixelate,
    ];

    pub fn name(self) -> &'static str {
        let t = crate::i18n::t();
        match self {
            Tool::Select => t.tool_select,
            Tool::Arrow => t.tool_arrow,
            Tool::Rectangle => t.tool_rectangle,
            Tool::Marker => t.tool_marker,
            Tool::Text => t.tool_text,
            Tool::Pixelate => t.tool_pixelate,
        }
    }

    /// Atajo de teclado provisional (hasta tener la barra flotante).
    pub fn key(self) -> egui::Key {
        match self {
            Tool::Select => egui::Key::S,
            Tool::Arrow => egui::Key::A,
            Tool::Rectangle => egui::Key::R,
            Tool::Marker => egui::Key::M,
            Tool::Text => egui::Key::T,
            Tool::Pixelate => egui::Key::P,
        }
    }
}

#[derive(Clone)]
pub enum Annotation {
    Arrow { from: Pos2, to: Pos2, style: Style },
    Rectangle { rect: Rect, style: Style },
    Marker { points: Vec<Pos2>, style: Style },
    Text { pos: Pos2, text: String, style: Style },
    Pixelate(Rect),
}

impl Annotation {
    /// Crea la anotación que corresponde a la herramienta al empezar a arrastrar.
    pub fn start(tool: Tool, pos: Pos2, style: Style) -> Option<Self> {
        match tool {
            Tool::Arrow => Some(Self::Arrow { from: pos, to: pos, style }),
            Tool::Rectangle => Some(Self::Rectangle { rect: Rect::from_two_pos(pos, pos), style }),
            Tool::Marker => Some(Self::Marker { points: vec![pos], style }),
            Tool::Pixelate => Some(Self::Pixelate(Rect::from_two_pos(pos, pos))),
            Tool::Select | Tool::Text => None,
        }
    }

    /// Actualiza la anotación mientras se arrastra desde `start` hasta `pos`.
    pub fn drag_to(&mut self, start: Pos2, pos: Pos2) {
        match self {
            Self::Arrow { to, .. } => *to = pos,
            Self::Rectangle { rect, .. } | Self::Pixelate(rect) => *rect = Rect::from_two_pos(start, pos),
            Self::Marker { points, .. } => {
                if points.last().is_none_or(|last| last.distance(pos) >= 1.0) {
                    points.push(pos);
                }
            }
            Self::Text { .. } => {}
        }
    }

    /// `true` si la anotación es demasiado pequeña para conservarla.
    pub fn is_degenerate(&self) -> bool {
        match self {
            Self::Arrow { from, to, .. } => from.distance(*to) < 3.0,
            Self::Rectangle { rect, .. } | Self::Pixelate(rect) => rect.width() < 2.0 || rect.height() < 2.0,
            Self::Marker { points, .. } => points.len() < 2,
            Self::Text { text, .. } => text.trim().is_empty(),
        }
    }

    /// Dibuja la vista previa en una ventana.
    ///
    /// `view_rect` es el área lógica global del monitor y `pixel_texture` la versión
    /// pixelada de su captura.
    pub fn paint(&self, painter: &egui::Painter, view_rect: Rect, pixel_texture: TextureId) {
        let offset = view_rect.min.to_vec2();
        let local = |p: Pos2| p - offset;
        match self {
            Self::Arrow { from, to, style } => {
                let width = style.stroke_width();
                if let Some((shaft_end, head)) = arrow_geometry(*from, *to, width) {
                    painter.line_segment([local(*from), local(shaft_end)], Stroke::new(width, style.color));
                    painter.add(Shape::convex_polygon(head.map(local).to_vec(), style.color, Stroke::NONE));
                }
            }
            Self::Rectangle { rect, style } => {
                painter.rect_stroke(
                    rect.translate(-offset),
                    0.0,
                    Stroke::new(style.stroke_width(), style.color),
                    StrokeKind::Middle,
                );
            }
            Self::Marker { points, style } => {
                let points = points.iter().map(|p| local(*p)).collect();
                painter.add(Shape::line(points, Stroke::new(style.marker_width(), style.marker_color())));
            }
            Self::Text { pos, text, style } => {
                let font = FontId::proportional(style.text_size());
                painter.text(local(*pos), Align2::LEFT_TOP, text, font, style.color);
            }
            Self::Pixelate(rect) => {
                let uv = Rect::from_min_max(
                    ((rect.min - view_rect.min) / view_rect.size()).to_pos2(),
                    ((rect.max - view_rect.min) / view_rect.size()).to_pos2(),
                );
                painter.image(pixel_texture, rect.translate(-offset), uv, Color32::WHITE);
            }
        }
    }
}

/// Punta de flecha: devuelve dónde termina el trazo y el triángulo de la punta.
fn arrow_geometry(from: Pos2, to: Pos2, width: f32) -> Option<(Pos2, [Pos2; 3])> {
    let delta = to - from;
    let length = delta.length();
    if length < 1.0 {
        return None;
    }
    let dir = delta / length;
    let head = (width * 4.0 + 8.0).min(length);
    let perp = dir.rot90();
    let base = to - dir * head;
    let triangle = [to, base + perp * head * 0.5, base - perp * head * 0.5];
    // El trazo termina dentro de la punta para que no asome por el vértice.
    Some((to - dir * head * 0.5, triangle))
}

/// Agrupa la imagen en bloques de `PIXEL_BLOCK`×`PIXEL_BLOCK` con su color medio.
pub fn pixelate(image: &RgbaImage) -> RgbaImage {
    let (w, h) = image.dimensions();
    let mut out = RgbaImage::new(w, h);
    for by in (0..h).step_by(PIXEL_BLOCK as usize) {
        for bx in (0..w).step_by(PIXEL_BLOCK as usize) {
            let bw = PIXEL_BLOCK.min(w - bx);
            let bh = PIXEL_BLOCK.min(h - by);
            let mut sum = [0u32; 4];
            for y in by..by + bh {
                for x in bx..bx + bw {
                    let p = image.get_pixel(x, y).0;
                    for c in 0..4 {
                        sum[c] += p[c] as u32;
                    }
                }
            }
            let n = bw * bh;
            let avg = image::Rgba(sum.map(|s| (s / n) as u8));
            for y in by..by + bh {
                for x in bx..bx + bw {
                    out.put_pixel(x, y, avg);
                }
            }
        }
    }
    out
}

/// Conversión de coordenadas lógicas globales a píxeles de la imagen recortada.
pub struct PixelMap {
    /// Origen lógico de la captura completa.
    pub bounds_min: Pos2,
    /// Píxeles de captura por unidad lógica.
    pub scale: Vec2,
    /// Esquina del recorte dentro de la captura completa, en píxeles.
    pub crop_origin: (u32, u32),
}

impl PixelMap {
    fn to_px(&self, p: Pos2) -> (f32, f32) {
        let v = (p - self.bounds_min) * self.scale;
        (v.x - self.crop_origin.0 as f32, v.y - self.crop_origin.1 as f32)
    }
}

/// Aplica las anotaciones, en orden, sobre la imagen recortada.
pub fn render(
    image: RgbaImage,
    annotations: &[Annotation],
    map: &PixelMap,
    pixelated: &RgbaImage,
) -> RgbaImage {
    let (w, h) = image.dimensions();
    // La captura es opaca, así que RGBA directo y premultiplicado coinciden.
    let size = tiny_skia::IntSize::from_wh(w, h).expect("imagen no vacía");
    let mut pixmap = Pixmap::from_vec(image.into_raw(), size).expect("tamaño válido");
    let scale = map.scale.x;

    for annotation in annotations {
        match annotation {
            Annotation::Arrow { from, to, style } => {
                let width = style.stroke_width() * scale;
                let Some((shaft_end, head)) = arrow_geometry(*from, *to, style.stroke_width()) else {
                    continue;
                };
                let mut shaft = PathBuilder::new();
                let (x, y) = map.to_px(*from);
                shaft.move_to(x, y);
                let (x, y) = map.to_px(shaft_end);
                shaft.line_to(x, y);
                stroke(&mut pixmap, shaft.finish(), width, style.color);

                let mut triangle = PathBuilder::new();
                for (i, p) in head.iter().enumerate() {
                    let (x, y) = map.to_px(*p);
                    if i == 0 { triangle.move_to(x, y) } else { triangle.line_to(x, y) }
                }
                triangle.close();
                if let Some(path) = triangle.finish() {
                    pixmap.fill_path(&path, &paint(style.color), FillRule::Winding, Transform::identity(), None);
                }
            }
            Annotation::Rectangle { rect, style } => {
                let (x0, y0) = map.to_px(rect.min);
                let (x1, y1) = map.to_px(rect.max);
                if let Some(r) = tiny_skia::Rect::from_ltrb(x0, y0, x1, y1) {
                    stroke(&mut pixmap, Some(PathBuilder::from_rect(r)), style.stroke_width() * scale, style.color);
                }
            }
            Annotation::Marker { points, style } => {
                let mut path = PathBuilder::new();
                for (i, p) in points.iter().enumerate() {
                    let (x, y) = map.to_px(*p);
                    if i == 0 { path.move_to(x, y) } else { path.line_to(x, y) }
                }
                stroke(&mut pixmap, path.finish(), style.marker_width() * scale, style.marker_color());
            }
            Annotation::Text { pos, text, style } => {
                draw_text(&mut pixmap, text, map.to_px(*pos), style.text_size() * scale, style.color);
            }
            Annotation::Pixelate(rect) => {
                copy_pixelated(&mut pixmap, map, *rect, pixelated);
            }
        }
    }

    RgbaImage::from_raw(w, h, pixmap.take()).expect("tamaño válido")
}

fn paint(color: Color32) -> Paint<'static> {
    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let mut paint = Paint::default();
    paint.set_color_rgba8(r, g, b, a);
    paint.anti_alias = true;
    paint
}

fn stroke(pixmap: &mut Pixmap, path: Option<tiny_skia::Path>, width: f32, color: Color32) {
    let Some(path) = path else { return };
    let stroke = tiny_skia::Stroke {
        width,
        line_cap: LineCap::Round,
        line_join: LineJoin::Round,
        ..Default::default()
    };
    pixmap.stroke_path(&path, &paint(color), &stroke, Transform::identity(), None);
}

/// Copia la región pixelada (alineada a la captura completa) sobre el recorte.
fn copy_pixelated(pixmap: &mut Pixmap, map: &PixelMap, rect: Rect, pixelated: &RgbaImage) {
    let (w, h) = (pixmap.width() as i64, pixmap.height() as i64);
    let (x0, y0) = map.to_px(rect.min);
    let (x1, y1) = map.to_px(rect.max);
    let x0 = (x0.round() as i64).clamp(0, w);
    let y0 = (y0.round() as i64).clamp(0, h);
    let x1 = (x1.round() as i64).clamp(0, w);
    let y1 = (y1.round() as i64).clamp(0, h);
    let (ox, oy) = (map.crop_origin.0 as i64, map.crop_origin.1 as i64);

    let data = pixmap.data_mut();
    for y in y0..y1 {
        for x in x0..x1 {
            let (sx, sy) = ((x + ox) as u32, (y + oy) as u32);
            if sx < pixelated.width() && sy < pixelated.height() {
                let i = ((y * w + x) * 4) as usize;
                data[i..i + 4].copy_from_slice(&pixelated.get_pixel(sx, sy).0);
            }
        }
    }
}

/// Texto en una línea con la misma fuente que egui (Ubuntu Light).
fn draw_text(pixmap: &mut Pixmap, text: &str, (x, y): (f32, f32), size_px: f32, color: Color32) {
    let font = FontRef::try_from_slice(epaint_default_fonts::UBUNTU_LIGHT).expect("fuente válida");
    // egui escala por em; ab_glyph por altura (ascenso - descenso).
    let units_per_em = font.units_per_em().unwrap_or(1000.0);
    let px_scale = PxScale::from(size_px * font.height_unscaled() / units_per_em);
    let scaled = font.as_scaled(px_scale);

    let [r, g, b, a] = color.to_srgba_unmultiplied();
    let (w, h) = (pixmap.width() as i32, pixmap.height() as i32);
    let data = pixmap.data_mut();

    let baseline = y + scaled.ascent();
    let mut caret = x;
    let mut previous = None;
    for ch in text.chars() {
        let id = scaled.glyph_id(ch);
        if let Some(prev) = previous {
            caret += scaled.kern(prev, id);
        }
        let glyph = id.with_scale_and_position(px_scale, point(caret, baseline));
        caret += scaled.h_advance(id);
        previous = Some(id);

        let Some(outlined) = font.outline_glyph(glyph) else { continue };
        let bounds = outlined.px_bounds();
        outlined.draw(|gx, gy, coverage| {
            let px = bounds.min.x as i32 + gx as i32;
            let py = bounds.min.y as i32 + gy as i32;
            if px < 0 || py < 0 || px >= w || py >= h {
                return;
            }
            let alpha = coverage.clamp(0.0, 1.0) * a as f32 / 255.0;
            let i = ((py * w + px) * 4) as usize;
            for (c, src) in [r, g, b].into_iter().enumerate() {
                let dst = data[i + c] as f32;
                data[i + c] = (src as f32 * alpha + dst * (1.0 - alpha)).round() as u8;
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gradient(w: u32, h: u32) -> RgbaImage {
        RgbaImage::from_fn(w, h, |x, y| image::Rgba([(x % 256) as u8, (y % 256) as u8, 128, 255]))
    }

    #[test]
    fn pixelate_uses_block_average() {
        let image = gradient(16, 16);
        let out = pixelate(&image);
        // Bloque 0..8: media de x = 3.5 → 3; todos los píxeles del bloque iguales.
        assert_eq!(out.get_pixel(0, 0), out.get_pixel(7, 7));
        assert_eq!(out.get_pixel(0, 0).0[0], 3);
        assert_ne!(out.get_pixel(0, 0), out.get_pixel(8, 0));
    }

    #[test]
    fn render_draws_all_annotations() {
        let image = gradient(400, 300);
        let pixelated = pixelate(&image);
        let map = PixelMap { bounds_min: Pos2::ZERO, scale: Vec2::splat(2.0), crop_origin: (0, 0) };
        let blue = Style { color: PALETTE[4], thickness: Thickness::Thin };
        let annotations = vec![
            Annotation::Pixelate(Rect::from_min_max(Pos2::new(110.0, 10.0), Pos2::new(190.0, 60.0))),
            Annotation::Arrow { from: Pos2::new(10.0, 140.0), to: Pos2::new(90.0, 80.0), style: blue },
            Annotation::Rectangle {
                rect: Rect::from_min_max(Pos2::new(10.0, 10.0), Pos2::new(90.0, 60.0)),
                style: Style::default(),
            },
            Annotation::Marker {
                points: vec![Pos2::new(110.0, 100.0), Pos2::new(190.0, 110.0)],
                style: Style { color: PALETTE[2], thickness: Thickness::Thick },
            },
            Annotation::Text { pos: Pos2::new(110.0, 120.0), text: "Hola áé".into(), style: blue },
        ];
        let out = render(image.clone(), &annotations, &map, &pixelated);

        // Borde del rectángulo (x = 10 lógico → 20 px) pintado de rojo.
        assert_eq!(&out.get_pixel(20, 60).0[..3], &[0xe0, 0x1b, 0x24]);
        // Zona pixelada copiada desde la imagen pixelada.
        assert_eq!(out.get_pixel(300, 50), pixelated.get_pixel(300, 50));
        assert_ne!(out, image);

        if let Ok(path) = std::env::var("RENDER_TEST_OUT") {
            out.save(path).unwrap();
        }
    }
}
