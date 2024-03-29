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

use imagehash::*;

#[test]
fn test_average_hash_1() {
    let dynimg = image::open("tests/1.jpg").unwrap();
    let result = AverageHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "00007cf0e0eafefe");
}

#[test]
fn test_average_hash_2() {
    let dynimg = image::open("tests/2.jpg").unwrap();
    let result = AverageHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "fff7e7e3c3000000");
}

#[test]
fn test_difference_hash_1() {
    let dynimg = image::open("tests/1.jpg").unwrap();
    let result = DifferenceHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "e0e0f0c4c6d290c0");
}

#[test]
fn test_difference_hash_2() {
    let dynimg = image::open("tests/2.jpg").unwrap();
    let result = DifferenceHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "ededcc860b0c19b6");
}

#[test]
fn test_perceptual_hash_1() {
    let dynimg = image::open("tests/1.jpg").unwrap();
    let result = PerceptualHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "2f2fafafafafafaf");
}

#[test]
fn test_perceptual_hash_2() {
    let dynimg = image::open("tests/2.jpg").unwrap();
    let result = PerceptualHash::new().hash(&dynimg);
    assert_eq!(result.to_string(), "3f3f3f4c4c4c4c4c");
}
