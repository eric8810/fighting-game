use anyhow::Result;
use std::collections::HashMap;
use wgpu::{Device, Queue};

/// GPU texture wrapper with bind group for shader access.
pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    pub size: (u32, u32),
}

impl Texture {
    /// Load a texture from raw PNG/image bytes.
    pub fn load_from_bytes(
        device: &Device,
        queue: &Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<Self> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let (width, height) = rgba.dimensions();

        let texture_size = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &rgba,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            texture_size,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok(Self {
            texture,
            view,
            sampler,
            size: (width, height),
        })
    }

    /// Create a 1x1 white texture (useful as a default / solid-color fallback).
    pub fn white_1x1(device: &Device, queue: &Queue) -> Self {
        let texture_size = wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("white_1x1"),
            size: texture_size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &[255u8, 255, 255, 255],
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4),
                rows_per_image: Some(1),
            },
            texture_size,
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            size: (1, 1),
        }
    }
}

// --- Texture Atlas ---

/// A region within a texture atlas.
#[derive(Clone, Copy, Debug)]
pub struct AtlasRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Sprite sheet / texture atlas that maps named regions to UV coordinates.
pub struct TextureAtlas {
    pub texture: Texture,
    pub regions: HashMap<String, AtlasRegion>,
}

impl TextureAtlas {
    pub fn new(texture: Texture) -> Self {
        Self {
            texture,
            regions: HashMap::new(),
        }
    }

    /// Add a named region to the atlas.
    pub fn add_region(&mut self, name: impl Into<String>, region: AtlasRegion) {
        self.regions.insert(name.into(), region);
    }

    /// Generate a uniform grid of regions (e.g. for a sprite sheet with equal-sized cells).
    pub fn generate_grid(&mut self, prefix: &str, cols: u32, rows: u32, cell_w: u32, cell_h: u32) {
        for row in 0..rows {
            for col in 0..cols {
                let index = row * cols + col;
                self.regions.insert(
                    format!("{prefix}_{index}"),
                    AtlasRegion {
                        x: col * cell_w,
                        y: row * cell_h,
                        width: cell_w,
                        height: cell_h,
                    },
                );
            }
        }
    }

    /// Get normalized UV offset and size for a named region.
    pub fn get_uv(&self, name: &str) -> Option<([f32; 2], [f32; 2])> {
        let region = self.regions.get(name)?;
        let (tw, th) = (self.texture.size.0 as f32, self.texture.size.1 as f32);
        Some((
            [region.x as f32 / tw, region.y as f32 / th],
            [region.width as f32 / tw, region.height as f32 / th],
        ))
    }
}
