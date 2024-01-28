# imagehash

The `imagehash` crate provides image hashing algorithms.

## Supported Algorithms

- Average Hash (aHash)
- Difference Hash (dHash)
- Perceptual Hash (pHash)

## Usage

```rust
let img_filename = "tests/1.jpg";
let img = image::open(img_filename).unwrap();

// Simple usage
let hash = imagehash::average_hash(&img);
println!("{}", hash); // hex-encoded hash string

// Advanced usage
let hasher = imagehash::AverageHash::new()
    .with_image_size(8, 8)
    .with_hash_size(8, 8)
    .with_resizer(|img, w, h| {
       // Your custom resizer function
       img.resize_exact(w as u32, h as u32, image::imageops::FilterType::Lanczos3)
   });
let hash = hasher.hash(&img);
println!("{}", hash); // hex-encoded hash string
```
