//! In-memory asset store + decoded-texture cache.
//!
//! v014 keeps raw image bytes in memory keyed by [`AssetId`], and a
//! parallel cache of [`egui::TextureHandle`] for elements that have
//! already been rendered at least once. A future milestone will move
//! the bytes to disk via `bse-storage` and add an LRU eviction policy.

use std::collections::HashMap;

use bse_types::AssetId;
use eframe::egui;
use image::ImageReader;
use tracing::warn;

/// Errors produced when loading an asset blob.
#[derive(Debug, thiserror::Error)]
pub enum AssetError {
    /// I/O failure reading the file.
    #[error("I/O : {0}")]
    Io(String),
    /// The bytes are not a supported image format.
    #[error("decode failure : {0}")]
    Decode(String),
}

impl From<std::io::Error> for AssetError {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err.to_string())
    }
}

impl From<image::ImageError> for AssetError {
    fn from(err: image::ImageError) -> Self {
        Self::Decode(err.to_string())
    }
}

/// One asset entry : the raw encoded bytes + decoded pixel dimensions
/// (in source pixels, before any world-space scaling).
#[derive(Clone, Debug)]
pub struct Asset {
    /// Raw encoded bytes (PNG, JPEG, etc.) — what the user supplied.
    pub bytes: Vec<u8>,
    /// Decoded image width in source pixels.
    pub pixel_width: u32,
    /// Decoded image height in source pixels.
    pub pixel_height: u32,
}

/// In-memory store of asset blobs + GPU texture cache.
#[derive(Default)]
pub struct AssetStore {
    blobs: HashMap<AssetId, Asset>,
    textures: HashMap<AssetId, egui::TextureHandle>,
}

impl AssetStore {
    /// Empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of stored assets.
    #[must_use]
    pub fn len(&self) -> usize {
        self.blobs.len()
    }

    /// `true` if no assets are stored.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blobs.is_empty()
    }

    /// Load image bytes from a path, store them and return the new id
    /// along with the decoded pixel dimensions.
    pub fn ingest_file(
        &mut self,
        path: &std::path::Path,
    ) -> Result<(AssetId, u32, u32), AssetError> {
        let bytes = std::fs::read(path)?;
        self.ingest_bytes(bytes)
    }

    /// Same as [`Self::ingest_file`] but from in-memory bytes (clipboard paste, etc.).
    pub fn ingest_bytes(&mut self, bytes: Vec<u8>) -> Result<(AssetId, u32, u32), AssetError> {
        let reader = ImageReader::new(std::io::Cursor::new(&bytes)).with_guessed_format()?;
        let img = reader.decode()?;
        let (w, h) = (img.width(), img.height());
        let id = AssetId::new_v7();
        self.blobs.insert(
            id,
            Asset {
                bytes,
                pixel_width: w,
                pixel_height: h,
            },
        );
        Ok((id, w, h))
    }

    /// Borrow a previously-ingested asset.
    #[must_use]
    pub fn get(&self, id: AssetId) -> Option<&Asset> {
        self.blobs.get(&id)
    }

    /// Return the `egui::TextureHandle` for `id`, decoding and uploading
    /// it on first call. Returns `None` if the asset is unknown.
    pub fn texture(&mut self, id: AssetId, ctx: &egui::Context) -> Option<&egui::TextureHandle> {
        if !self.textures.contains_key(&id) {
            let asset = self.blobs.get(&id)?;
            match decode_to_color_image(asset) {
                Ok(image) => {
                    let handle = ctx.load_texture(
                        format!("asset-{}", id.as_uuid()),
                        image,
                        egui::TextureOptions::LINEAR,
                    );
                    self.textures.insert(id, handle);
                }
                Err(err) => {
                    warn!(target: "bse::assets", asset = %id, error = %err, "decode failed");
                    return None;
                }
            }
        }
        self.textures.get(&id)
    }
}

fn decode_to_color_image(asset: &Asset) -> Result<egui::ColorImage, AssetError> {
    let reader = ImageReader::new(std::io::Cursor::new(&asset.bytes)).with_guessed_format()?;
    let img = reader.decode()?.to_rgba8();
    let size = [img.width() as usize, img.height() as usize];
    Ok(egui::ColorImage::from_rgba_unmultiplied(size, img.as_raw()))
}
