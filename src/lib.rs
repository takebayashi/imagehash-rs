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
//! let img_filename = "tests/1.jpg";
//! let img = image::open(img_filename).unwrap();
//!
//! // Simple usage
//! let hash = imagehash::average_hash(&img);
//! println!("{}", hash); // hex-encoded hash string
//!
//! // Advanced usage
//! let hasher = imagehash::AverageHash::new()
//!     .with_image_size(8, 8)
//!     .with_hash_size(8, 8)
//!     .with_resizer(|img, w, h| {
//!        // Your custom resizer function
//!        img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3)
//!    });
//! let hash = hasher.hash(&img);
//! println!("{}", hash); // hex-encoded hash string
//! ```

/// Represents a hash value.
#[derive(Debug)]
pub struct Hash {
    /// The bit vector representation of the hash.
    pub bits: Vec<bool>,
}

impl Hash {
    /// Returns the byte vector representation of the hash.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = vec![0; (self.bits.len() + 7) / 8];
        for (i, bit) in self.bits.iter().enumerate() {
            if *bit {
                bytes[i / 8] |= 1 << (7 - (i % 8));
            }
        }
        bytes
    }
}

impl From<Vec<bool>> for Hash {
    fn from(bits: Vec<bool>) -> Self {
        Hash { bits }
    }
}

impl std::fmt::Display for Hash {
    /// Returns the hex-encoded string representation of the hash.
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let bytes = self.to_bytes();
        let mut result = String::with_capacity(bytes.len() * 2);
        for byte in bytes {
            result.push_str(&format!("{:02x}", byte));
        }
        write!(f, "{}", result)
    }
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

fn resize(image: &image::DynamicImage, width: usize, height: usize) -> image::DynamicImage {
    image.resize_exact(
        width as u32,
        height as u32,
        image::imageops::FilterType::Lanczos3,
    )
}

/// Provides average hash (aHash) calculation.
pub struct AverageHash {
    image_size: (usize, usize),
    hash_size: (usize, usize),
    resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
}

impl AverageHash {
    /// Creates a new `AverageHasher` with default parameters.
    pub fn new() -> Self {
        AverageHash::default()
    }

    /// Constructs a hasher with the image size.
    pub fn with_image_size(self, width: usize, height: usize) -> Self {
        AverageHash {
            image_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the hash size.
    pub fn with_hash_size(self, width: usize, height: usize) -> Self {
        AverageHash {
            hash_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the resizer function.
    pub fn with_resizer(
        self,
        resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
    ) -> Self {
        AverageHash { resizer, ..self }
    }

    /// Calculates average hash (aHash) of the image and returns as a hex string.
    pub fn hash(&self, image: &image::DynamicImage) -> Hash {
        let image: GrayscaleImage =
            (self.resizer)(&image.grayscale(), self.image_size.0, self.image_size.0).into();
        average_hash_core(&image, self.hash_size.0, self.hash_size.1)
    }
}

impl Default for AverageHash {
    /// Creates a new `AverageHasher` with default parameters.
    fn default() -> Self {
        AverageHash {
            image_size: (8, 8),
            hash_size: (8, 8),
            resizer: resize,
        }
    }
}

/// Calculates average hash (aHash) of the image.
pub fn average_hash(image: &image::DynamicImage) -> Hash {
    let image: GrayscaleImage = resize(&image.grayscale(), 8, 8).into();
    average_hash_core(&image, 8, 8)
}

fn average_hash_core(image: &GrayscaleImage, hash_width: usize, hash_height: usize) -> Hash {
    let total: f64 = image
        .iter_rows_as::<f64>()
        .take(hash_height)
        .flat_map(|row| row.take(hash_width))
        .sum();
    let mean = total / (hash_width * hash_height) as f64;
    image
        .iter_pixels_as::<f64>()
        .map(|v| v > mean)
        .collect::<Vec<bool>>()
        .into()
}

/// Provides difference hash (dHash) calculation.
pub struct DifferenceHash {
    image_size: (usize, usize),
    hash_size: (usize, usize),
    resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
}

impl DifferenceHash {
    /// Creates a new `DifferenceHasher` with default parameters.
    pub fn new() -> Self {
        DifferenceHash::default()
    }

    /// Constructs a hasher with the image size.
    pub fn with_image_size(self, width: usize, height: usize) -> Self {
        DifferenceHash {
            image_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the hash size.
    pub fn with_hash_size(self, width: usize, height: usize) -> Self {
        DifferenceHash {
            hash_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the resizer function.
    pub fn with_resizer(
        self,
        resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
    ) -> Self {
        DifferenceHash { resizer, ..self }
    }

    /// Calculates difference hash (dHash) of the image and returns as a hex string.
    pub fn hash(&self, image: &image::DynamicImage) -> Hash {
        let image: GrayscaleImage =
            (self.resizer)(&image.grayscale(), self.image_size.0, self.image_size.1).into();
        difference_hash_core(&image, self.hash_size.0, self.hash_size.1)
    }
}

impl Default for DifferenceHash {
    /// Creates a new `DifferenceHasher` with default parameters.
    fn default() -> Self {
        DifferenceHash {
            image_size: (9, 8),
            hash_size: (8, 8),
            resizer: resize,
        }
    }
}

/// Calculates difference hash (dHash) of the image.
pub fn difference_hash(image: &image::DynamicImage) -> Hash {
    let image: GrayscaleImage = resize(&image.grayscale(), 9, 8).into();
    difference_hash_core(&image, 8, 8)
}

fn difference_hash_core(image: &GrayscaleImage, hash_width: usize, hash_height: usize) -> Hash {
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
        .collect::<Vec<bool>>()
        .into()
}
