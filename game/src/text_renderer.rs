use glyphon::{
    Attrs, Buffer, Cache, Color as GlyphonColor, Family, FontSystem, Metrics, Resolution,
    Shaping, SwashCache, TextArea as GlyphonTextArea, TextAtlas, TextBounds,
    TextRenderer as GlyphonRenderer, Viewport,
};
use wgpu::MultisampleState;

const FONT_DATA: &[u8] = include_bytes!("../../assets/fonts/PressStart2P-Regular.ttf");
const FONT_NAME: &str = "Press Start 2P";

/// A text item to render this frame.
pub struct TextArea {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub color: [f32; 4],
    /// Optional (max_width, max_height) bounds for the text box.
    pub bounds: Option<(f32, f32)>,
}

pub struct TextRenderer {
    font_system: FontSystem,
    swash_cache: SwashCache,
    cache: Cache,
    viewport: Viewport,
    atlas: TextAtlas,
    renderer: GlyphonRenderer,
}

impl TextRenderer {
    pub fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_format: wgpu::TextureFormat,
    ) -> Self {
        let mut font_system = FontSystem::new();
        // Load Press Start 2P pixel font (OFL license)
        font_system.db_mut().load_font_data(FONT_DATA.to_vec());
        let swash_cache = SwashCache::new();
        let cache = Cache::new(device);
        let viewport = Viewport::new(device, &cache);
        let mut atlas = TextAtlas::new(device, queue, &cache, surface_format);
        let renderer =
            GlyphonRenderer::new(&mut atlas, device, MultisampleState::default(), None);
        Self {
            font_system,
            swash_cache,
            cache,
            viewport,
            atlas,
            renderer,
        }
    }

    /// Prepare all text areas for rendering this frame.
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen_w: f32,
        screen_h: f32,
        text_areas: &[TextArea],
    ) {
        self.viewport.update(
            queue,
            Resolution {
                width: screen_w as u32,
                height: screen_h as u32,
            },
        );

        // Build glyphon TextArea list, keeping Buffers alive for the duration.
        let mut buffers: Vec<Buffer> = Vec::with_capacity(text_areas.len());
        for ta in text_areas {
            let line_height = ta.size * 1.2;
            let mut buf = Buffer::new(&mut self.font_system, Metrics::new(ta.size, line_height));
            let (bw, bh) = ta.bounds.unwrap_or((screen_w - ta.x, screen_h - ta.y));
            buf.set_size(&mut self.font_system, Some(bw), Some(bh));
            buf.set_text(
                &mut self.font_system,
                &ta.text,
                &Attrs::new().family(Family::Name(FONT_NAME)),
                Shaping::Advanced,
                None,
            );
            buf.shape_until_scroll(&mut self.font_system, false);
            buffers.push(buf);
        }

        let glyphon_areas: Vec<GlyphonTextArea<'_>> = buffers
            .iter()
            .zip(text_areas.iter())
            .map(|(buf, ta)| {
                let r = (ta.color[0] * 255.0) as u8;
                let g = (ta.color[1] * 255.0) as u8;
                let b = (ta.color[2] * 255.0) as u8;
                let a = (ta.color[3] * 255.0) as u8;
                GlyphonTextArea {
                    buffer: buf,
                    left: ta.x,
                    top: ta.y,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: ta.x as i32,
                        top: ta.y as i32,
                        right: (ta.x + ta.bounds.map(|(w, _)| w).unwrap_or(screen_w - ta.x))
                            as i32,
                        bottom: (ta.y + ta.bounds.map(|(_, h)| h).unwrap_or(screen_h - ta.y))
                            as i32,
                    },
                    default_color: GlyphonColor::rgba(r, g, b, a),
                    custom_glyphs: &[],
                }
            })
            .collect();

        let _ = self.renderer.prepare(
            device,
            queue,
            &mut self.font_system,
            &mut self.atlas,
            &self.viewport,
            glyphon_areas,
            &mut self.swash_cache,
        );
    }

    /// Render text into an active render pass.
    pub fn render<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        let _ = self.renderer.render(&self.atlas, &self.viewport, pass);
    }

    /// Trim the glyph atlas (call after present).
    pub fn trim_atlas(&mut self) {
        self.atlas.trim();
    }
}
