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
//! - Perceptual Hash (pHash)
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
    height: usize,
}

impl GrayscaleImage {
    /// Creates a new `GrayscaleImage` from the flattened pixels.
    fn new(pixels: Vec<u8>, width: usize, height: usize) -> Self {
        assert_eq!(pixels.len(), width * height);
        GrayscaleImage {
            pixels,
            width,
            height,
        }
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

/// Provides perceptual hash (pHash) calculation.
pub struct PerceptualHash {
    image_size: (usize, usize),
    hash_size: (usize, usize),
    resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
}

impl PerceptualHash {
    /// Creates a new `PerceptualHasher` with default parameters.
    pub fn new() -> Self {
        PerceptualHash::default()
    }

    /// Constructs a hasher with the image size.
    pub fn with_image_size(self, width: usize, height: usize) -> Self {
        PerceptualHash {
            image_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the hash size.
    pub fn with_hash_size(self, width: usize, height: usize) -> Self {
        PerceptualHash {
            hash_size: (width, height),
            ..self
        }
    }

    /// Constructs a hasher with the resizer function.
    pub fn with_resizer(
        self,
        resizer: fn(&image::DynamicImage, usize, usize) -> image::DynamicImage,
    ) -> Self {
        PerceptualHash { resizer, ..self }
    }

    /// Calculates perceptual hash (pHash) of the image and returns as a hex string.
    pub fn hash(&self, image: &image::DynamicImage) -> Hash {
        let image: GrayscaleImage =
            (self.resizer)(&image.grayscale(), self.image_size.0, self.image_size.1).into();
        perceptual_hash_core(&image, self.hash_size.0, self.hash_size.1)
    }
}

impl Default for PerceptualHash {
    /// Creates a new `PerceptualHasher` with default parameters.
    fn default() -> Self {
        PerceptualHash {
            image_size: (32, 32),
            hash_size: (8, 8),
            resizer: resize,
        }
    }
}

/// Calculates perceptual hash (pHash) of the image.
pub fn perceptual_hash(image: &image::DynamicImage) -> Hash {
    let image: GrayscaleImage = resize(&image.grayscale(), 32, 32).into();
    perceptual_hash_core(&image, 8, 8)
}

fn perceptual_hash_core(image: &GrayscaleImage, hash_width: usize, hash_height: usize) -> Hash {
    let mut dct_rows = vec![0.0; image.width * image.height];
    for (y, row) in image.iter_rows_as::<f64>().enumerate() {
        let dct = dct2(&row.collect::<Vec<_>>());
        for (x, v) in dct.iter().enumerate() {
            dct_rows[y * image.width + x] = *v;
        }
    }
    let low_freqs: Vec<f64> = dct_rows
        .chunks(image.width)
        .take(hash_height)
        .flat_map(|row| {
            row.iter()
                .skip(1)
                .take(hash_width)
                .copied()
                .collect::<Vec<_>>()
        })
        .collect();
    let mean = low_freqs.iter().sum::<f64>() / (hash_width * hash_height) as f64;
    low_freqs
        .iter()
        .map(|v| *v > mean)
        .collect::<Vec<bool>>()
        .into()
}

fn dct2(input: &[f64]) -> Vec<f64> {
    // scipy-style dct-ii
    let n = input.len();
    (0..n)
        .map(|k| {
            input
                .iter()
                .enumerate()
                .map(|(i, xi)| {
                    2.0_f64
                        * xi
                        * (std::f64::consts::PI * k as f64 * (2 * i + 1) as f64 / (2 * n) as f64)
                            .cos()
                })
                .sum::<f64>()
        })
        .collect()
}

#[test]
fn test_dct2() {
    let input = vec![0., 1., 2.];
    let actual = dct2(&input);
    let expected = [6.00000000e+00, -3.46410162e+00, -4.44089210e-16];
    assert_eq!(actual.len(), expected.len());
    for (a, e) in actual.iter().zip(expected.iter()) {
        assert!((a - e).abs() < 1e-8);
    }
}
