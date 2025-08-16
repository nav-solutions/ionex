IONEX
=====

[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml)
[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/ionex/badge.svg)](https://docs.rs/ionex/)
[![crates.io](https://img.shields.io/crates/d/ionex.svg)](https://crates.io/crates/ionex)

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
- Convenient `geo` bridging functions
- Full 2D support
- TEC Root Mean Square is supported
- File formatting is validated for 2D IONEX, including TEC RMS

## Limitations

- Height map not supported yet
- This parser/formatter will not work well if coordinates grid 
is is not the same across regions or between maps. 
Next version should support that as well.

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
use ionex::prelude::{IONEX};

let ionex = IONEX::from_gzip_file("../data/IONEX/V1/CKMG0020.22I.gz")
    .unwrap();

// Most IONEX files provide 2D maps
assert!(ionex.is_2d());


// File header gives meaningful information

// TEC maps in chronlogical order, 
// standard format is 1hour between TEC evolution,
// starting at midnight, last map at midnight-1: 25 maps per day.
assert_eq!(ionex.header.number_of_maps, 25);

// chronology is expressed in UTC 
// 
assert_eq!(ionex.header.epoch_of_first_map.to_string(), "2020-06-25T00:00:00 UTC");
assert_eq!(ionex.header.epoch_of_last_map.to_string(), "2020-06-25T00:00:00 UTC");

// Map borders, this is a worldwide file

assert!(ionex.is_worldwide_ionex()); // only for files that use standard naming

// Ground stations that served during evaluation
assert_eq!(ionex.header.nb_stations, 0);

// Satellites that served during evaluation
assert_eq!(ionex.header.nb_satellites, 0);

```
