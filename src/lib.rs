// Copyright 2024 Shun Takebayashi
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! The `imagehash` crate provides image hashing algorithms.
//!
//! ## Supported Algorithms
//!
//! - Average Hash (aHash)
//! - Difference Hash (dHash)
//!
//! ## Usage
//!
//! ```rust
//! use image;
//! use imagehash::AverageHash;
//!
//! let img_filename = "tests/1.jpg";
//! let img = image::open(img_filename).unwrap();
//!
//! let hasher = AverageHash::default();
//! let hash = hasher.hash(&img);
//! println!("{}", hash); // hex-encoded hash string
//! ```

pub use image::imageops::FilterType;

/// Contains the image pre-processing parameters.
pub struct ImageOp {
    pub width: u8,
    pub height: u8,
    pub filter: FilterType,
}

/// Represents a grayscale image.
struct GrayscaleImage {
    pixels: Vec<u8>,
    width: usize,
}

impl GrayscaleImage {
    /// Creates a new `GrayscaleImage` from the flattened pixels.
    fn new(pixels: Vec<u8>, width: usize, height: usize) -> Self {
        assert_eq!(pixels.len(), width * height);
        GrayscaleImage { pixels, width }
    }

    /// Returns an iterator over the pixels as the specified type.
    fn iter_pixels_as<'a, T>(&'a self) -> impl Iterator<Item = T> + 'a
    where
        T: From<u8> + 'a,
    {
        self.pixels.iter().map(|&v| T::from(v))
    }

    /// Returns an iterator over the rows as the specified type.
    fn iter_rows_as<'a, T>(&'a self) -> impl Iterator<Item = impl Iterator<Item = T> + 'a> + 'a
    where
        T: From<u8> + 'a,
    {
        self.pixels
            .chunks(self.width)
            .map(|row| row.iter().map(|&v| T::from(v)))
    }
}

impl From<image::DynamicImage> for GrayscaleImage {
    fn from(image: image::DynamicImage) -> Self {
        let width = image.width() as usize;
        let height = image.height() as usize;
        GrayscaleImage::new(image.into_luma8().into_raw(), width, height)
    }
}

fn resize(image: &image::DynamicImage, op: &ImageOp) -> image::DynamicImage {
    image.resize_exact(op.width as u32, op.height as u32, op.filter)
}

/// Provides average hash (aHash) calculation.
pub struct AverageHash<'a> {
    op: &'a ImageOp,
}

impl<'a> AverageHash<'a> {
    /// Creates a new `AverageHasher` with default parameters.
    pub fn new() -> Self {
        AverageHash::default()
    }

    /// Creates a new `AverageHasher` with the specified parameters.
    pub fn with_op(op: &'a ImageOp) -> Self {
        AverageHash { op }
    }

    /// Calculates average hash (aHash) of the image and returns as a hex string.
    pub fn hash(&self, image: &image::DynamicImage) -> String {
        let bits = average_hash(image, self.op);
        let bytes = bits_to_bytes(&bits);
        bytes_to_hex(&bytes)
    }
}

impl Default for AverageHash<'_> {
    /// Creates a new `AverageHasher` with default parameters.
    fn default() -> Self {
        AverageHash {
            op: &ImageOp {
                width: 8,
                height: 8,
                filter: FilterType::Lanczos3,
            },
        }
    }
}

/// Calculates average hash (aHash) of the image.
pub fn average_hash(image: &image::DynamicImage, op: &ImageOp) -> Vec<bool> {
    let image: GrayscaleImage = resize(&image.grayscale(), op).into();
    average_hash_core(&image, 8, 8)
}

fn average_hash_core(image: &GrayscaleImage, hash_width: usize, hash_height: usize) -> Vec<bool> {
    let total: f64 = image
        .iter_rows_as::<f64>()
        .take(hash_height)
        .flat_map(|row| row.take(hash_width))
        .sum();
    let mean = total / (hash_width * hash_height) as f64;
    image.iter_pixels_as::<f64>().map(|v| v > mean).collect()
}

/// Provides difference hash (dHash) calculation.
pub struct DifferenceHash<'a> {
    op: &'a ImageOp,
}

impl<'a> DifferenceHash<'a> {
    /// Creates a new `DifferenceHasher` with default parameters.
    pub fn new() -> Self {
        DifferenceHash::default()
    }

    /// Creates a new `DifferenceHasher` with the specified parameters.
    pub fn with_op(op: &'a ImageOp) -> Self {
        DifferenceHash { op }
    }

    /// Calculates difference hash (dHash) of the image and returns as a hex string.
    pub fn hash(&self, image: &image::DynamicImage) -> String {
        let bits = difference_hash(image, self.op);
        let bytes = bits_to_bytes(&bits);
        bytes_to_hex(&bytes)
    }
}

impl Default for DifferenceHash<'_> {
    /// Creates a new `DifferenceHasher` with default parameters.
    fn default() -> Self {
        DifferenceHash {
            op: &ImageOp {
                width: 9,
                height: 8,
                filter: FilterType::Lanczos3,
            },
        }
    }
}

/// Calculates difference hash (dHash) of the image.
pub fn difference_hash(image: &image::DynamicImage, op: &ImageOp) -> Vec<bool> {
    let image: GrayscaleImage = resize(&image.grayscale(), op).into();
    difference_hash_core(&image, 8, 8)
}

fn difference_hash_core(
    image: &GrayscaleImage,
    hash_width: usize,
    hash_height: usize,
) -> Vec<bool> {
    image
        .iter_rows_as::<u8>()
        .take(hash_height)
        .flat_map(|row| {
            row.collect::<Vec<u8>>()
                .windows(2)
                .take(hash_width)
                .map(|w| w[1] > w[0])
                .collect::<Vec<bool>>()
        })
        .collect()
}

fn bits_to_bytes(bits: &[bool]) -> Vec<u8> {
    let mut bytes = vec![0; (bits.len() + 7) / 8];
    for (i, bit) in bits.iter().enumerate() {
        if *bit {
            bytes[i / 8] |= 1 << (7 - (i % 8));
        }
    }
    bytes
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push_str(&format!("{:02x}", byte));
    }
    result
}
