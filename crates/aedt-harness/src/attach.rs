//! The files a message carries: an image's bytes as the model takes them, and
//! the path that every file keeps.

use std::{fs, path::Path};

use base64::{Engine, engine::general_purpose::STANDARD};
use rig_harness::prelude::{UserContent, message::ImageMediaType};

/// The most an image may take. A larger file goes as its path alone, so one
/// attachment cannot hold a whole turn's request in memory.
const MAX_IMAGE_BYTES: u64 = 20 * 1024 * 1024;

/// `path`'s bytes as an image the model reads: a PNG, JPEG, GIF or WebP by its
/// first bytes, no larger than [`MAX_IMAGE_BYTES`]. `None` for every other
/// file, which goes as its path alone.
pub(crate) fn image(path: &Path) -> Option<UserContent> {
    let meta = fs::metadata(path).ok()?;
    if !meta.is_file() || meta.len() > MAX_IMAGE_BYTES {
        return None;
    }
    let bytes = fs::read(path).ok()?;
    let media = ImageMediaType::sniff(&bytes)?;

    Some(UserContent::image_base64(
        STANDARD.encode(&bytes),
        Some(media),
        None,
    ))
}
