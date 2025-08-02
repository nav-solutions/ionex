IONEX
=====

[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/rust.yml)
[![Rust](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml/badge.svg)](https://github.com/nav-solutions/ionex/actions/workflows/daily.yml)
[![crates.io](https://docs.rs/ionex/badge.svg)](https://docs.rs/ionex/)
[![crates.io](https://img.shields.io/crates/d/ionex.svg)](https://crates.io/crates/ionex)

[![MRSV](https://img.shields.io/badge/MSRV-1.82.0-orange?style=for-the-badge)](https://github.com/rust-lang/rust/releases/tag/1.82.0)
[![License](https://img.shields.io/badge/license-MPL_2.0-orange?style=for-the-badge&logo=mozilla)](https://github.com/nav-solutions/ionex/blob/main/LICENSE)


`ionex` is small library to parse IONEX files. IONEX (Ionosphere Maps) are RINEX-like 
ASCII files that describe an estimate of the Total Electron Density (TEC) in the 
Ionosphere layer. 

To contribute to either of our project or join our community, you way
- open an [Issue on Github.com](https://github.com/nav-solutions/ionex/issues) 
- follow our [Discussions on Github.com](https://github.com/nav-solutions/discussions)
- join our [Discord channel](https://discord.gg/EqhEBXBmJh)

## Advantages :rocket: 

- Fast
- Open sources: read and access all the code
- Seamless Gzip decompression (on `flate2` feature)
- Convenient `geo` bridging functions
- Full 2D support
- Partial 3D support
- File formatting is work in progress

## Citation and referencing

If you need to reference this work, please use the following model:

`Nav-solutions (2025), RINEX: analysis and processing (MPLv2), https://github.com/nav-solutions`

## Contributions

Contributions are welcomed, do not hesitate to open new issues
and submit Pull Requests through Github. Join us on Discord for more detailed
discussions.

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
