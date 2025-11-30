IONEX
=====

[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml)
[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml)  

[![crates.io](https://img.shields.io/crates/v/ionex.svg)](https://crates.io/crates/ionex)
[![crates.io](https://docs.rs/ionex/badge.svg)](https://docs.rs/ionex/)
[![crates.io](https://img.shields.io/crates/d/ionex.svg)](https://crates.io/crates/ionex)
[![discord server](https://img.shields.io/discord/1342922474110586910?logo=discord)](https://discord.gg/EqhEBXBmJh)

[![MRSV](https://img.shields.io/badge/MSRV-1.82.0-orange?style=for-the-badge)](https://github.com/rust-lang/rust/releases/tag/1.82.0)
[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/nav-solutions/ionex/blob/main/LICENSE)

`ionex` is Rust library to parse IONEX and process files. 

IONEX files are RINEX-like ASCII file that describe the Total Electron Density (TEC)
in the Ionosphere, using TEC maps. They can be Global / Worldwide or limited to a region
(so called regionnal IONEX). The map is quantized in specific coordinates given a TEC estimate
for each position. In case of 3D (volumic) IONEX, the volume is also quantized, with an altitude
quantization spec that is constant over entire fileset and is described in the header.

## Advantages

- Fast and powerful parser
- Open source
- Seamless Gzip decompression (on `flate2` feature)
- Full 2D support
- TEC Root Mean Square is supported
- File formatting is now supported for 2D IONEX, including RMS maps.
- Spatial and Temporal interpolation now supported

## Limitations

- Height map not supported yet

## Citation and referencing

If you need to reference this work, please use the following model:

`Nav-solutions (2025), IONEX (MPLv2), https://github.com/nav-solutions`

## Contributions

Contributions are welcomed:

- open an [Issue on Github.com](https://github.com/nav-solutions/ionex/issues) 
- follow our [Discussions on Github.com](https://github.com/nav-solutions/discussions)
- join our [Discord channel](https://discord.gg/EqhEBXBmJh)

## Getting started

```rust
use std::fs::File;
use std::io::BufWriter;

use ionex::prelude::*;

// Parse Global/worldwide map
let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    .unwrap();

// header contains high level information
// like file standard revision:
assert_eq!(ionex.header.version.major, 1);
assert_eq!(ionex.header.version.minor, 0);

// mean altitude above mean-sea-level of the ionosphere
assert_eq!(ionex.header.grid.altitude.start, 350.0);
assert_eq!(ionex.header.grid.altitude.end, 350.0);

// radius of the mean-sea-level
assert_eq!(ionex.header.base_radius_km, 6371.0);

// most file are 2D maps
// meaning they "only" give the evolution of an isosurface
// at previous altitude, above mean sea level
assert!(ionex.is_2d());

// this file is named according to IGS standards
let descriptor = ionex.production.clone().unwrap();

// to obtain TEC values at any coordinates, you
// should use the [MapCell] local region (rectangle quanta)
// that offers many functions based off the Geo crate.

// Convenient helper to follow standard conventions
let filename = ionex.standardized_filename();

// Dump to file
let fd = File::create("custom.txt").unwrap();
let mut writer = BufWriter::new(fd);

ionex.format(&mut writer)
    .unwrap_or_else(|e| {
        panic!("failed to format IONEX: {}", e);
    });

// parse back
let _ = IONEX::from_file("custom.txt")
    .unwrap_or_else(|e| {
        panic!("failed to parse IONEX: {}", e);
    });
```
