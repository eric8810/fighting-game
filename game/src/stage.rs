use serde::Deserialize;

/// A single parallax background layer.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct ParallaxLayer {
    /// Identifier / texture path for this layer.
    pub texture: String,
    /// Scroll speed multiplier relative to camera (0.0 = static, 1.0 = moves with camera).
    pub scroll_speed: f32,
    /// Depth ordering (lower = further back, rendered first).
    pub depth: u32,
    /// RGBA tint color for placeholder rendering.
    #[serde(default = "default_color")]
    pub color: [f32; 4],
    /// Vertical offset in screen pixels from the ground line.
    #[serde(default)]
    pub y_offset: f32,
    /// Height of this layer in screen pixels.
    #[serde(default = "default_layer_height")]
    pub height: f32,
}

fn default_color() -> [f32; 4] {
    [0.2, 0.2, 0.3, 1.0]
}

fn default_layer_height() -> f32 {
    600.0
}

/// Stage boundary limits in logic coordinates (1/100 pixel).
#[derive(Clone, Copy, Debug, Deserialize)]
pub struct StageBoundaries {
    /// Left edge in logic coords.
    pub left: i32,
    /// Right edge in logic coords.
    pub right: i32,
}

/// Camera limits derived from stage width.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub struct CameraLimits {
    pub min_x: f32,
    pub max_x: f32,
}

/// Complete stage definition, loadable from RON.
#[derive(Clone, Debug, Deserialize)]
#[allow(dead_code)]
pub struct StageData {
    pub name: String,
    /// Total stage width in logic coordinates.
    pub width: i32,
    /// Path to background music.
    #[serde(default)]
    pub bgm_path: String,
    /// Parallax layers (rendered back-to-front by depth).
    pub layers: Vec<ParallaxLayer>,
    /// Fighter movement boundaries.
    pub boundaries: StageBoundaries,
}

/// Runtime stage state with computed camera info.
pub struct Stage {
    pub data: StageData,
    pub camera_x: f32,
    pub camera_limits: CameraLimits,
}

impl Stage {
    /// Load a stage from a RON file path.
    pub fn load_from_file(path: &str) -> Result<Self, StageLoadError> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| StageLoadError::Io(path.to_string(), e))?;
        let data: StageData =
            ron::from_str(&contents).map_err(|e| StageLoadError::Parse(path.to_string(), e))?;
        Ok(Self::from_data(data))
    }

    /// Create a stage from already-parsed data.
    pub fn from_data(data: StageData) -> Self {
        let stage_width_px = data.width as f32 / 100.0;
        // Camera can scroll from 0 to (stage_width - screen_width).
        // We use a default screen width of 800 for limits; actual clamping
        // happens in update_camera which takes the real screen width.
        let camera_limits = CameraLimits {
            min_x: 0.0,
            max_x: (stage_width_px - 800.0).max(0.0),
        };
        Self {
            data,
            camera_x: 0.0,
            camera_limits,
        }
    }

    /// Create a default training stage without loading from disk.
    pub fn default_dojo() -> Self {
        let data = StageData {
            name: "Dojo".to_string(),
            width: 120_000, // 1200 px
            bgm_path: String::new(),
            layers: vec![
                ParallaxLayer {
                    texture: "sky".to_string(),
                    scroll_speed: 0.0,
                    depth: 0,
                    color: [0.05, 0.05, 0.15, 1.0],
                    y_offset: 0.0,
                    height: 600.0,
                },
                ParallaxLayer {
                    texture: "mountains".to_string(),
                    scroll_speed: 0.2,
                    depth: 1,
                    color: [0.1, 0.1, 0.2, 1.0],
                    y_offset: 200.0,
                    height: 250.0,
                },
                ParallaxLayer {
                    texture: "dojo_floor".to_string(),
                    scroll_speed: 0.5,
                    depth: 2,
                    color: [0.25, 0.15, 0.1, 1.0],
                    y_offset: 450.0,
                    height: 150.0,
                },
            ],
            boundaries: StageBoundaries {
                left: 0,
                right: 120_000,
            },
        };
        Self::from_data(data)
    }

    /// Update camera to track the midpoint between two fighters.
    pub fn update_camera(&mut self, p1_x: f32, p2_x: f32, screen_width: f32) {
        let midpoint = (p1_x + p2_x) / 2.0;
        let target = midpoint - screen_width / 2.0;
        let stage_width_px = self.data.width as f32 / 100.0;
        let max_x = (stage_width_px - screen_width).max(0.0);
        self.camera_x = target.clamp(0.0, max_x);
        self.camera_limits.max_x = max_x;
    }

    /// Clamp a logic-coordinate X position to stage boundaries.
    pub fn clamp_x(&self, x: i32) -> i32 {
        x.clamp(self.data.boundaries.left, self.data.boundaries.right)
    }

    /// Compute the screen-space X offset for a parallax layer given the current camera.
    pub fn layer_offset_x(&self, layer: &ParallaxLayer) -> f32 {
        -self.camera_x * layer.scroll_speed
    }

    /// Build quad instances for all parallax layers (placeholder colored rects).
    /// `screen_w` and `screen_h` are the viewport dimensions in pixels.
    /// `ground_screen_y` is the Y pixel position of the ground line on screen.
    pub fn render_layers(
        &self,
        screen_w: f32,
        _screen_h: f32,
        _ground_screen_y: f32,
    ) -> Vec<crate::quad_renderer::QuadInstance> {
        let mut instances = Vec::new();
        let mut sorted_layers: Vec<&ParallaxLayer> = self.data.layers.iter().collect();
        sorted_layers.sort_by_key(|l| l.depth);

        for layer in sorted_layers {
            let offset_x = self.layer_offset_x(layer);
            // Layer y position: measured from top of screen.
            // y_offset=0 means top of screen, larger values move down.
            let y = if layer.y_offset > 0.0 {
                layer.y_offset
            } else {
                0.0
            };
            // Tile the layer across the visible area.
            // For simplicity, draw one wide rect that covers the screen.
            let layer_w = screen_w + self.camera_limits.max_x.abs() + 200.0;

            instances.push(crate::quad_renderer::QuadInstance {
                rect: [offset_x - 100.0, y, layer_w, layer.height],
                color: layer.color,
                ..Default::default()
            });
        }
        instances
    }
}

#[derive(Debug)]
pub enum StageLoadError {
    Io(String, std::io::Error),
    Parse(String, ron::error::SpannedError),
}

impl std::fmt::Display for StageLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(path, e) => write!(f, "failed to read stage '{}': {}", path, e),
            Self::Parse(path, e) => write!(f, "failed to parse stage '{}': {}", path, e),
        }
    }
}

impl std::error::Error for StageLoadError {}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_dojo_creation() {
        let stage = Stage::default_dojo();
        assert_eq!(stage.data.name, "Dojo");
        assert_eq!(stage.data.width, 120_000);
        assert_eq!(stage.data.boundaries.left, 0);
        assert_eq!(stage.data.boundaries.right, 120_000);
        assert!(!stage.data.layers.is_empty());
    }

    #[test]
    fn test_clamp_x_within_bounds() {
        let stage = Stage::default_dojo();
        assert_eq!(stage.clamp_x(50_000), 50_000);
    }

    #[test]
    fn test_clamp_x_below_left() {
        let stage = Stage::default_dojo();
        assert_eq!(stage.clamp_x(-5000), 0);
    }

    #[test]
    fn test_clamp_x_above_right() {
        let stage = Stage::default_dojo();
        assert_eq!(stage.clamp_x(200_000), 120_000);
    }

    #[test]
    fn test_camera_update_centers_on_midpoint() {
        let mut stage = Stage::default_dojo();
        // Two fighters at 400px and 600px (logic: 40000, 60000).
        stage.update_camera(400.0, 600.0, 800.0);
        // Midpoint = 500, target = 500 - 400 = 100.
        assert!((stage.camera_x - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_camera_clamps_to_left() {
        let mut stage = Stage::default_dojo();
        stage.update_camera(100.0, 200.0, 800.0);
        // Midpoint = 150, target = 150 - 400 = -250 -> clamped to 0.
        assert_eq!(stage.camera_x, 0.0);
    }

    #[test]
    fn test_camera_clamps_to_right() {
        let mut stage = Stage::default_dojo();
        // Stage is 1200px wide, screen 800px, so max_x = 400.
        stage.update_camera(1100.0, 1200.0, 800.0);
        // Midpoint = 1150, target = 1150 - 400 = 750 -> clamped to 400.
        assert_eq!(stage.camera_x, 400.0);
    }

    #[test]
    fn test_parallax_layer_offset() {
        let mut stage = Stage::default_dojo();
        stage.camera_x = 200.0;
        let layer_slow = ParallaxLayer {
            texture: "bg".to_string(),
            scroll_speed: 0.5,
            depth: 0,
            color: [0.0; 4],
            y_offset: 0.0,
            height: 100.0,
        };
        let layer_fast = ParallaxLayer {
            texture: "fg".to_string(),
            scroll_speed: 1.0,
            depth: 1,
            color: [0.0; 4],
            y_offset: 0.0,
            height: 100.0,
        };
        assert!((stage.layer_offset_x(&layer_slow) - (-100.0)).abs() < 0.01);
        assert!((stage.layer_offset_x(&layer_fast) - (-200.0)).abs() < 0.01);
    }

    #[test]
    fn test_render_layers_sorted_by_depth() {
        let stage = Stage::default_dojo();
        let instances = stage.render_layers(800.0, 600.0, 500.0);
        // Should have one instance per layer.
        assert_eq!(instances.len(), stage.data.layers.len());
    }

    #[test]
    fn test_load_from_ron_string() {
        let ron_str = r#"(
            name: "Test Stage",
            width: 100000,
            bgm_path: "",
            layers: [
                (texture: "bg", scroll_speed: 0.0, depth: 0),
            ],
            boundaries: (left: 0, right: 100000),
        )"#;
        let data: StageData = ron::from_str(ron_str).unwrap();
        let stage = Stage::from_data(data);
        assert_eq!(stage.data.name, "Test Stage");
        assert_eq!(stage.data.width, 100_000);
    }
}
