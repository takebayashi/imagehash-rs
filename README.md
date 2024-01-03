# imagehash

The `imagehash` crate provides image hashing algorithms.

## Supported Algorithms

- Average Hash (aHash)
- Difference Hash (dHash)

## Usage

```rust
use image;
use imagehash::AverageHash;

let img_filename = "tests/1.jpg";
let img = image::open(img_filename).unwrap();

let hasher = AverageHash::default();
let hash = hasher.hash(&img);
println!("{}", hash); // hex-encoded hash string
```
