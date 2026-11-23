#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nav-solutions/.github/master/logos/logo2.jpg"
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/*
 * IONEX is part of the nav-solutions framework.
 *
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/nav-solutions/ionex/graphs/contributors),
 * licensed under Mozilla Public license V2.
 *
 * Documentation: https://github.com/nav-solutions/ionex
 */

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate gnss_rs as gnss;

pub mod bias;
pub mod error;
pub mod file_attributes;
pub mod grid;
pub mod header;
pub mod interpolation;
pub mod ipp;
pub mod key;
pub mod linspace;
pub mod mapf;
pub mod system;
pub mod tec;
pub mod version;

mod cell;
mod coordinates;
mod epoch;
mod ionosphere;
mod quantized;
mod record;

#[cfg(test)]
mod tests;

use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};

use itertools::Itertools;

use geo::{BoundingRect, Geometry, Point, Polygon, Rect, coord};

#[cfg(feature = "flate2")]
use flate2::{Compression as GzCompression, read::GzDecoder, write::GzEncoder};

use hifitime::prelude::{Duration, Epoch, TimeSeries};

use crate::{
    cell::{MapCell, TecPoint},
    coordinates::QuantizedCoordinates,
    error::{Error, FormattingError, ParsingError},
    file_attributes::{FileAttributes, Region},
    grid::Grid,
    header::Header,
    interpolation::TemporalInterpolation,
    ipp::IPP,
    key::Key,
    linspace::Linspace,
    quantized::Quantized,
    record::Record,
    tec::TEC,
};

pub mod prelude {
    // export
    pub use crate::{
        Comments, IONEX,
        bias::BiasSource,
        cell::{Cell3x3, MapCell, TecPoint},
        error::{Error, FormattingError, ParsingError},
        file_attributes::*,
        grid::{Axis, Grid},
        header::Header,
        interpolation::TemporalInterpolation,
        ionosphere::IonosphereParameters,
        ipp::IPP,
        key::Key,
        linspace::Linspace,
        mapf::MappingFunction,
        record::Record,
        system::ReferenceSystem,
        tec::TEC,
        version::Version,
    };

    // pub re-export
    pub use geo::{
        BoundingRect, GeodesicArea, Geometry, LineString, Point, Polygon, Rect,
        algorithm::contains::Contains, coord,
    };
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries, Unit};
}

/// IONEX comments are readable descriptions.
pub type Comments = Vec<String>;

/// Divides provided usize, returns ceil rounded value.
fn div_ceil(value: usize, divider: usize) -> usize {
    let q = value.div_euclid(divider);
    let r = value.rem_euclid(divider);
    if r == 0 { q } else { q + 1 }
}

/// Converts a geo [Rect]angle (x: longitude, y: latitude) to its
/// NE, SE, SW, NW corners as (latitude, longitude) pairs in degrees
pub(crate) fn rectangle_quadrant_decomposition(
    rect: Rect,
) -> ((f64, f64), (f64, f64), (f64, f64), (f64, f64)) {
    let (min, max) = (rect.min(), rect.max());
    (
        (max.y, max.x),
        (min.y, max.x),
        (min.y, min.x),
        (max.y, min.x),
    )
}

/// Converts the NE, SE, SW, NW corners, as (latitude, longitude) pairs in degrees,
/// to a [Rect]angle (x: longitude, y: latitude)
#[allow(dead_code)]
pub(crate) fn quadrant_to_rectangle(
    quadrant: ((f64, f64), (f64, f64), (f64, f64), (f64, f64)),
) -> Rect {
    let (ne_lat, ne_long) = quadrant.0;
    let (sw_lat, sw_long) = quadrant.2;
    Rect::new(coord!(x: sw_long, y: sw_lat), coord!(x: ne_long, y: ne_lat))
}

/// macro to format one header line or a comment
pub(crate) fn fmt_ionex(content: &str, marker: &str) -> String {
    if content.len() < 60 {
        format!("{:<padding$}{}", content, marker, padding = 60)
    } else {
        let mut string = String::new();

        let nb_lines = div_ceil(content.len(), 60);

        for i in 0..nb_lines {
            let start_off = i * 60;
            let end_off = std::cmp::min(start_off + 60, content.len());
            let chunk = &content[start_off..end_off];
            string.push_str(&format!("{:<padding$}{}", chunk, marker, padding = 60));
            if i < nb_lines - 1 {
                string.push('\n');
            }
        }
        string
    }
}

/// macro to generate comments with standardized formatting
pub(crate) fn fmt_comment(content: &str) -> String {
    fmt_ionex(content, "COMMENT")
}

/// [IONEX] is composed of a [Header] section and a [Record] section.
/// It is the discrete estimation of the Total Electron Content (TEC)
/// over a plane layer or volume of the ionosphere.
///
/// ```
/// use std::fs::File;
/// use std::io::BufWriter;
///
/// use ionex::prelude::*;
///
/// // Worldwide (so called 'global') IONEX map
/// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
///     .unwrap();
///
/// // header contains high level information
/// // like file standard revision:
/// assert_eq!(ionex.header.version.major, 1);
/// assert_eq!(ionex.header.version.minor, 0);
///
/// // mean altitude above mean-sea-level of the ionosphere
/// assert_eq!(ionex.header.grid.altitude.start, 350.0);
/// assert_eq!(ionex.header.grid.altitude.end, 350.0);
///
/// // radius of the mean-sea-level
/// assert_eq!(ionex.header.base_radius_km, 6371.0);
///
/// // most file are 2D maps
/// // meaning they "only" give the evolution of an isosurface
/// // at previous altitude, above mean sea level
/// assert!(ionex.is_2d());
///
/// // this file is named according to IGS standards
/// let descriptor = ionex.attributes.clone().unwrap();
///
/// // to obtain TEC values at any coordinates and any point in time,
/// // use [IONEX::tec_at] (2D maps) or [IONEX::volumetric_tec_at] (3D maps).
/// // The [MapCell] local region (rectangle quanta)
/// // offers many functions based off the Geo crate.
///
/// // Convenient helper to follow standard conventions
/// let filename = ionex.generate_standardized_filename();
///
/// // Dump to file
/// let fd = File::create("custom.txt").unwrap();
/// let mut writer = BufWriter::new(fd);
///
/// ionex.format(&mut writer)
///     .unwrap_or_else(|e| {
///         panic!("failed to format IONEX: {}", e);
///     });
///
/// // parse back
/// let _ = IONEX::from_file("custom.txt")
///     .unwrap_or_else(|e| {
///         panic!("failed to parse region of interest");
///     });
/// ```
#[derive(Clone, Debug, Default, PartialEq)]
pub struct IONEX {
    /// [Header] gives general information and describes following content.
    pub header: Header,

    /// IONEX [Record].
    pub record: Record,

    /// [Comments] found in file record
    pub comments: Comments,

    /// [FileAttributes] resolved for file names that follow the IGS conventions.
    pub attributes: Option<FileAttributes>,
}

impl IONEX {
    /// Builds a new [IONEX] struct from given header & body sections.
    pub fn new(header: Header, record: Record) -> Self {
        Self {
            header,
            record,
            attributes: None,
            comments: Default::default(),
        }
    }

    /// Copy and return this [IONEX] with updated [Header].
    pub fn with_header(&self, header: Header) -> Self {
        Self {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Replace [Header] with mutable access.
    pub fn replace_header(&mut self, header: Header) {
        self.header = header.clone();
    }

    /// Copy and return this [IONEX] with updated [Record]
    pub fn with_record(&self, record: Record) -> Self {
        IONEX {
            record,
            header: self.header.clone(),
            comments: self.comments.clone(),
            attributes: self.attributes.clone(),
        }
    }

    /// Replace [Record] with mutable access.
    pub fn replace_record(&mut self, record: Record) {
        self.record = record.clone();
    }

    /// Returns true if this [IONEX] is 2D (planar TEC map, not 3D volume).
    pub fn is_2d(&self) -> bool {
        self.header.map_dimension == 2
    }

    /// Returns true if this [IONEX] is 3D
    pub fn is_3d(&self) -> bool {
        !self.is_2d()
    }

    /// Returns total altitude range covered, in kilometers.
    pub fn altitude_width_km(&self) -> f64 {
        self.header.grid.altitude.width()
    }

    /// Returns a file name that would describe [Self] according to the
    /// standards.
    pub fn generate_standardized_filename(&self) -> String {
        let (agency, region, year, doy) = if let Some(attributes) = &self.attributes {
            (
                attributes.agency.clone(),
                attributes.region,
                attributes.year - 2000,
                attributes.doy,
            )
        } else {
            ("XXX".to_string(), Region::default(), 0, 0)
        };

        let extension = match &self.attributes {
            #[cfg(feature = "flate2")]
            Some(attributes) if attributes.gzip_compressed => ".gz",
            _ => "",
        };

        format!("{}{}{:03}0.{:02}I{}", agency, region, doy, year, extension)
    }

    /// Guesses [FileAttributes] from actual dataset. This is particularly useful
    /// to generate a standardized file name, especially when arriving from data that
    /// did not follow the conventions.
    /// The name of the production agency (data provider) is only determined by the [FileAttributes]
    /// itself, so we have no means to generate correct one, so we need you to define it right here.
    /// The production agency should be at least a 3 letter code, for example: "IGS".
    pub fn guess_file_attributes(&self, agency: &str) -> Option<FileAttributes> {
        if agency.len() < 3 {
            return None;
        }

        let first_epoch = self.record.first_epoch()?;
        let year = first_epoch.year();
        let doy = first_epoch.day_of_year().round() as u32;

        let region = if self.header.grid.is_worldwide() {
            Region::Worldwide
        } else {
            Region::Regional
        };

        Some(FileAttributes {
            doy,
            region,
            year: year as u32,
            agency: agency.to_string(),

            #[cfg(feature = "flate2")]
            gzip_compressed: if let Some(attributes) = &self.attributes {
                attributes.gzip_compressed
            } else {
                false
            },
        })
    }

    /// Parse [IONEX] content by consuming [BufReader] (efficient buffered reader).
    /// Attributes potentially described by a file name need to be provided either
    /// manually / externally, or guessed when parsing has been completed.
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        // Parses Header section (=consumes header until this point)
        let mut header = Header::parse(reader)?;

        // Parse record (=consumes rest of this resource)
        // Comments are preserved and store "as is"
        let (record, comments) = Record::parse(&mut header, reader)?;

        Ok(Self {
            header,
            record,
            comments,
            attributes: Default::default(),
        })
    }

    /// Format [RINEX] into writable I/O using efficient buffered writer
    /// and following standard specifications. The revision to be followed is defined
    /// in [Header] section. This is the mirror operation of [Self::parse].
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        self.header.format(writer)?;

        // format all comments at beginning of file
        for comment in self.comments.iter() {
            writeln!(writer, "{}", fmt_comment(comment))?;
        }

        self.record.format(&self.header, writer)?;

        writer.flush()?;
        Ok(())
    }

    /// Parses [IONEX] from local readable file.
    ///
    /// Will panic if provided file does not exist or is not readable.
    /// See [Self::from_gzip_file] for seamless Gzip support.
    ///
    /// If file name follows standard naming conventions, then internal definitions
    /// will truly be complete. Otherwise [FileAttributes] cannot be fully determined.
    /// If you want or need to you can either
    ///  1. define it yourself with further customization
    ///  2. use the smart guesser (after parsing): [Self::guess_attributes_attributes]
    ///
    /// This is typically needed in data attributes contexts.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<IONEX, ParsingError> {
        let path = path.as_ref();

        // deduce all we can from file name
        let file_attributes = match path.file_name() {
            Some(filename) => {
                let filename = filename.to_string_lossy().to_string();
                if let Ok(prod) = FileAttributes::from_str(&filename) {
                    Some(prod)
                } else {
                    None
                }
            },
            _ => None,
        };

        let fd = File::open(path)?;
        let mut reader = BufReader::new(fd);

        let mut ionex = Self::parse(&mut reader)?;
        ionex.attributes = file_attributes;

        Ok(ionex)
    }

    /// Dumps [IONEX] into writable local file (as readable ASCII UTF-8)
    /// using efficient buffered formatting.
    /// This is the mirror operation of [Self::from_file].
    /// Returns total amount of bytes that was generated.
    /// ```
    /// // Read a IONEX and dump it without any modifications
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///   .unwrap();
    ///
    /// assert!(ionex.to_file("test.txt").is_ok());
    /// ```
    ///
    /// Other useful links are in data attributes contexts:
    ///   * [Self::standard_filename] to generate a standardized filename
    ///   * [Self::guess_attributes_attributes] helps generate standardized filenames for
    ///     files that do not follow naming conventions
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let mut writer = BufWriter::new(fd);
        self.format(&mut writer)?;
        Ok(())
    }

    /// Parses [IONEX] from local gzip compressed file.
    ///
    /// Will panic if provided file does not exist or is not readable.
    /// Refer to [Self::from_file] for more information.
    ///
    /// ```
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// assert!(ionex.is_2d());
    ///
    /// // fixed altitude IONEX (=single isosurface)
    /// assert_eq!(ionex.header.grid.altitude.start, 350.0);
    /// assert_eq!(ionex.header.grid.altitude.end, 350.0);
    ///     
    /// // latitude grid
    /// assert_eq!(ionex.header.grid.latitude.start, 87.5);
    /// assert_eq!(ionex.header.grid.latitude.end, -87.5);
    /// assert_eq!(ionex.header.grid.latitude.spacing, -2.5);
    ///
    /// // longitude grid
    /// assert_eq!(ionex.header.grid.longitude.start, -180.0);
    /// assert_eq!(ionex.header.grid.longitude.end, 180.0);
    /// assert_eq!(ionex.header.grid.longitude.spacing, 5.0);

    /// assert_eq!(ionex.header.elevation_cutoff, 0.0);
    /// assert_eq!(ionex.header.mapf, MappingFunction::None);
    /// ```
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<IONEX, ParsingError> {
        let path = path.as_ref();

        // deduce all we can from file name
        let file_attributes = match path.file_name() {
            Some(filename) => {
                let filename = filename.to_string_lossy().to_string();
                if let Ok(prod) = FileAttributes::from_str(&filename) {
                    Some(prod)
                } else {
                    None
                }
            },
            _ => None,
        };

        let fd = File::open(path)?;

        let reader = GzDecoder::new(fd);
        let mut reader = BufReader::new(reader);

        let mut ionex = Self::parse(&mut reader)?;
        ionex.attributes = file_attributes;

        Ok(ionex)
    }

    /// Dumps and gzip encodes [IONEX] into writable local file,
    /// using efficient buffered formatting.
    ///
    /// This operation is [Self::from_gzip_file] mirror operation.
    /// ```
    /// // Read a IONEX and dump it without any modifications
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///   .unwrap();
    ///
    /// assert!(ionex.to_gzip_file("test.txt.gz").is_ok());
    /// ```
    ///
    /// Other useful methods are:
    ///   * [Self::generate_standardized_filename]: to generate a standardize file name
    ///   * [Self::guess_file_attributes] in close relation: helps generate standardized
    /// filenames for files that did now follow the convention
    ///   * [gnss_qc_traits::Merge]: to combine two files to one another
    ///   * [Self::to_regional_roi]: to reduce a larger (possibly [Region::GlobalWorldwide] map)
    /// to a given ROI
    ///   * [Self::to_worldwide_map]: to enlarge a possibly [Region::Regional] map to a [Region::GlobalWorldwide] IONEX.
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn to_gzip_file<P: AsRef<Path>>(&self, path: P) -> Result<(), FormattingError> {
        let fd = File::create(path)?;
        let compression = GzCompression::new(5);
        let mut writer = BufWriter::new(GzEncoder::new(fd, compression));
        self.format(&mut writer)?;
        Ok(())
    }

    /// Determines whether this [IONEX] is the result of a previous merge operation.
    /// That is, the combination of two files merged together.  
    /// This is determined by the presence of custom yet somewhat standardized [Comments].
    pub fn is_merged(&self) -> bool {
        for comment in self.header.comments.iter() {
            if comment.contains("FILE MERGE") {
                return true;
            }
        }
        false
    }

    /// Returns map borders as a [Rect]angle, with coordinates in decimal degrees.
    /// This uses the [Header] description and assumes all maps are within these borders.
    pub fn bounding_rect_degrees(&self) -> Rect {
        Rect::new(
            coord!( x: self.header.grid.longitude.start, y: self.header.grid.latitude.start ),
            coord!( x: self.header.grid.longitude.end, y: self.header.grid.latitude.end),
        )
    }

    /// Returns true if this [IONEX] is a Worldwide map
    /// (as opposed to a local/regional ROI).
    pub fn is_worldwide_map(&self) -> bool {
        if let Some(attributes) = &self.attributes {
            attributes.region == Region::Worldwide
        } else {
            let bounding_rect = self.bounding_rect_degrees();
            bounding_rect.width() == 360.0 && bounding_rect.height() == 175.0
        }
    }

    /// Returns true if this [IONEX] is not a Worldwide map but a regional/local ROI.
    pub fn is_regional_map(&self) -> bool {
        !self.is_worldwide_map()
    }

    /// Stretch this [IONEX] definition so it becomes compatible
    /// with the description of a Global/Worldwide [IONEX].
    pub fn to_worldwide_ionex(&self) -> IONEX {
        let mut ionex = self.clone();

        if let Some(attributes) = &mut ionex.attributes {
            attributes.region = Region::Worldwide;
        }

        // update grid specs, preserve accuracy
        ionex.header.grid.latitude.start = -87.5;
        ionex.header.grid.latitude.end = 87.5;
        ionex.header.grid.longitude.start = -180.0;
        ionex.header.grid.longitude.end = 180.0;

        // insert appriopriate values

        ionex
    }

    /// Reduce this [IONEX] definition so it is reduced to a regional ROI,
    /// described by a complex [Polygon] in decimal degrees.
    /// The quantization (both spatial and temporal) is preserved, only the
    /// spatial dimensions are modified by this operation.
    /// [Polygon::bounding_rect] must be defined for this operation to work correctly.
    ///
    /// Assuming the original map and '*' marking the complex ROI you want to draw,
    /// the resuling IONEX is the smallest rectangle of grid nodes the ROI fits in, because this library
    /// (and correct/valid IONEX) is limited to rectangles. The ROI is saturated to the map
    /// boundaries, and returns [Error::OutsideSpatialBoundaries] if it does not intersect this map.
    ///
    /// origin
    /// ----------------------------------
    /// |       returned roi___           |
    /// |      |       * * *  |           |
    /// |      |    *      *  |           |
    /// |      | * specs   *  |           |
    /// |      | *        *   |           |
    /// |      |  ****** *    |           |
    /// |       ______________|           |
    /// |---------------------------------|
    pub fn to_regional_ionex(&self, roi: Polygon) -> Result<IONEX, Error> {
        let boundaries = roi.bounding_rect().ok_or(Error::UndefinedBoundaries)?;

        /// Snaps value to the closest grid node, below (or above) this value,
        /// saturated to the axis boundaries.
        fn snap(linspace: &Linspace, value: f64, below: bool) -> f64 {
            let (min, max) = linspace.minmax();
            let value = value.clamp(min, max);
            let nodes = linspace.quantize().map(|q| q.real_value());

            if below {
                nodes
                    .filter(|node| *node <= value + 1.0E-9)
                    .fold(min, f64::max)
            } else {
                nodes
                    .filter(|node| *node >= value - 1.0E-9)
                    .fold(max, f64::min)
            }
        }

        let grid = &self.header.grid;

        let (min_lat, max_lat) = (
            snap(&grid.latitude, boundaries.min().y, true),
            snap(&grid.latitude, boundaries.max().y, false),
        );

        let (min_long, max_long) = (
            snap(&grid.longitude, boundaries.min().x, true),
            snap(&grid.longitude, boundaries.max().x, false),
        );

        if min_lat >= max_lat || min_long >= max_long {
            return Err(Error::OutsideSpatialBoundaries);
        }

        let mut ionex = self.clone();

        ionex.record.map.retain(|k, _| {
            let (lat, long) = (k.latitude_ddeg(), k.longitude_ddeg());
            lat >= min_lat - 1.0E-9
                && lat <= max_lat + 1.0E-9
                && long >= min_long - 1.0E-9
                && long <= max_long + 1.0E-9
        });

        // update attributes
        ionex.attributes = Some(match &self.attributes {
            Some(attributes) => attributes.clone().regionalized(),
            None => FileAttributes::default().regionalized(),
        });

        // update header, preserving the axes orientation
        let latitude = &mut ionex.header.grid.latitude;
        (latitude.start, latitude.end) = if latitude.spacing < 0.0 {
            (max_lat, min_lat)
        } else {
            (min_lat, max_lat)
        };

        let longitude = &mut ionex.header.grid.longitude;
        (longitude.start, longitude.end) = if longitude.spacing < 0.0 {
            (max_long, min_long)
        } else {
            (min_long, max_long)
        };

        Ok(ionex)
    }

    // /// Modify the grid dimensions by a positive, possibly fractional number,
    // /// and interpolates the TEC values.
    // ///
    // /// This does not modify the grid quantization, only the dimensions.
    // ///
    // /// ## Input
    // /// - axis: [Axis] to be stretched
    // /// - factor:
    // ///    - > 1.0: upscaling
    // ///    - < 1.0: downscaling
    // ///    - 0.0: invalid
    // ///
    // /// Example 1: spatial upscaling
    // /// ```
    // /// use ionex::prelude::{
    // ///     IONEX,
    // ///     Axis,
    // /// };
    // ///
    // /// // grab a local dataset, upscaling hardly applies to worldwide maps..
    // /// // this example is a data/IONEX/V1/CKMG0020.22I.gz that went through a x0.5 downscale.
    // /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMR0020.22I.gz").unwrap_or_else(|e| {
    // ///     panic!("Failed to parse CKMR0020: {}", e);
    // /// });
    // ///
    // /// // spatial dimensions upscale including interpolation: upscales to worldwide dimensions
    // /// let upscaled = ionex.spatially_stretched(Axis::Planar, 2.0)
    // ///     .unwrap();
    // ///
    // /// // testbench: compares these results to the original data
    // /// ```
    // pub fn spatially_stretched(&self, axis: Axis, factor: f64) -> Result<IONEX, Error> {
    //     let mut s = self.clone();
    //     s.spatial_stretching_mut(axis, factor)?;
    //     Ok(s)
    // }

    // /// Modify the grid dimensions by a positive, possibly fractional number,
    // /// and interpolates the TEC values.
    // ///
    // /// This does not modify the grid quantization, only the dimensions.
    // ///
    // /// ## Input
    // /// - axis: [Axis] to be stretched
    // /// - factor:
    // ///    - > 1.0: upscaling
    // ///    - < 1.0: downscaling
    // ///    - 0.0: invalid
    // pub fn spatial_stretching_mut(&mut self, axis: Axis, factor: f64) -> Result<(), Error> {
    //     if factor > 2.0 {
    //         // the maximal factor supported per iter is 2
    //         // let mut fact = factor.max(2.0);
    //         // while fact > 1.0 {
    //         //     self.spatial_stretching_mut(fact);
    //         //     fact = factor /2.0;
    //         // }
    //         Err(Error::TemporalMismatch) // TODO
    //     } else {

    //         // Update header specs
    //         match axis {
    //             Axis::Latitude => {
    //                 self.header.grid.latitude.stretch_mut(factor)?;
    //             },
    //             Axis::Longitude => {
    //                 self.header.grid.longitude.stretch_mut(factor)?;
    //             },
    //             Axis::Altitude => {
    //                 self.header.grid.altitude.stretch_mut(factor)?;
    //             },
    //             Axis::Planar => {
    //                 self.header.grid.latitude.stretch_mut(factor)?;
    //                 self.header.grid.longitude.stretch_mut(factor)?;
    //             },
    //             Axis::All => {
    //                 self.header.grid.latitude.stretch_mut(factor)?;
    //                 self.header.grid.longitude.stretch_mut(factor)?;
    //                 self.header.grid.altitude.stretch_mut(factor)?;
    //             },
    //         }

    //         // update the attributes
    //         match self.attributes {
    //             Some(ref mut attributes) => {
    //                 if self.header.grid.is_worldwide() {
    //                     attributes.region = Region::Worldwide;
    //                 } else {
    //                     attributes.region = Region::Regional;
    //                 }
    //             },
    //             None => {
    //                 let mut attributes = FileAttributes::default();
    //                 if self.header.grid.is_worldwide() {
    //                     attributes.region = Region::Worldwide;
    //                 } else {
    //                     attributes.region = Region::Regional;
    //                 }

    //                 self.attributes = Some(attributes);
    //             },
    //         }

    //         // synchronous spatial interpolation
    //         for epoch in self.epoch_iter() {
    //             for cell3x3 in self
    //                 .map_cell3x3_iter()
    //                 .filter(|cell| cell.center.epoch == epoch)
    //             {
    //                 // if factor 1.0 {
    //                 // } else {
    //                 // }
    //             }
    //         }

    //         Ok(())
    //     }
    // }

    /// Returns an iterator over [Epoch]s in chronological order.
    pub fn epoch_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.record.map.keys().map(|k| k.epoch).unique().sorted())
    }

    /// Resamples this [IONEX] in time: a new map is produced every sampling period,
    /// from the first map to the last map (included, if it lines up), each node
    /// being interpolated with the [TemporalInterpolation] strategy of your choice.
    /// The spatial grid is preserved. Omitted values remain omitted.
    ///
    /// This is both an upsampling (shorter period) and a downsampling (longer period)
    /// operation. Downsampling simply selects the maps when they line up with the new period.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// // reduce to a regional map (faster)
    /// let roi = Rect::new(coord!(x: -10.0, y: 35.0), coord!(x: 30.0, y: 60.0));
    /// let ionex = ionex.to_regional_ionex(roi.into()).unwrap();
    ///
    /// // 1 hour maps to 15 minutes maps
    /// let resampled = ionex
    ///     .temporally_resampled(Duration::from_seconds(900.0), TemporalInterpolation::Linear)
    ///     .unwrap();
    ///
    /// assert_eq!(resampled.header.number_of_maps, 97);
    /// assert_eq!(resampled.epoch_iter().count(), 97);
    /// assert_eq!(resampled.header.sampling_period, Duration::from_seconds(900.0));
    /// ```
    pub fn temporally_resampled(
        &self,
        sampling_period: Duration,
        method: TemporalInterpolation,
    ) -> Result<IONEX, Error> {
        if sampling_period <= Duration::ZERO {
            return Err(Error::InvalidStretchFactor);
        }

        let (first, last) = (
            self.record
                .first_epoch()
                .ok_or(Error::OutsideTemporalBoundaries)?,
            self.record
                .last_epoch()
                .ok_or(Error::OutsideTemporalBoundaries)?,
        );

        let mut record = Record::default();

        // grid nodes, as described by the first map
        let nodes = self
            .record
            .map
            .keys()
            .take_while(|k| k.epoch == first)
            .map(|k| k.coordinates)
            .collect::<Vec<_>>();

        let mut last_epoch = first;

        for epoch in TimeSeries::inclusive(first, last, sampling_period) {
            for coordinates in nodes.iter() {
                let point = Point::new(coordinates.longitude_ddeg(), coordinates.latitude_ddeg());

                match self.interpolate(epoch, point, coordinates.altitude_km(), method) {
                    Ok(tec) => {
                        record.insert(
                            Key {
                                epoch,
                                coordinates: *coordinates,
                            },
                            tec,
                        );
                    },
                    Err(Error::MissingData) => {},
                    Err(e) => return Err(e),
                }
            }

            last_epoch = epoch;
        }

        let mut ionex = self.with_record(record);
        ionex.header.sampling_period = sampling_period;
        ionex.header.epoch_of_first_map = first;
        ionex.header.epoch_of_last_map = last_epoch;
        ionex.header.number_of_maps = ionex.epoch_iter().count() as u32;

        Ok(ionex)
    }

    /// Resamples this [IONEX] in space: the grid spacing is modified (both
    /// latitude and longitude, in degrees) while the map boundaries are preserved,
    /// and the TEC (and RMS) is interpolated at every new grid node, for every map
    /// (and every altitude layer of 3D maps). The temporal axis is preserved.
    /// Omitted values remain omitted (the 4 surrounding nodes must be defined).
    ///
    /// The spacings are absolute values, the axes orientations are preserved, and
    /// they must divide the latitude and longitude ranges.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// // reduce to a regional map (faster)
    /// let roi = Rect::new(coord!(x: -10.0, y: 35.0), coord!(x: 30.0, y: 60.0));
    /// let regional = ionex.to_regional_ionex(roi.into()).unwrap();
    ///
    /// // 2.5°x5° grid to 0.5°x0.5° grid
    /// let resampled = regional.spatially_resampled(0.5, 0.5).unwrap();
    /// assert_eq!(resampled.header.grid.latitude.spacing, -0.5);
    /// assert_eq!(resampled.header.grid.longitude.spacing, 0.5);
    ///
    /// let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
    /// let toulouse = Point::new(1.5, 43.5);
    ///
    /// // new grid node
    /// let key = Key::from_decimal_degrees_km(noon, 43.5, 1.5, 350.0);
    /// let tec = resampled.record.get(&key).unwrap();
    ///
    /// assert!((tec.tecu() - ionex.tec_at(noon, toulouse).unwrap().tecu()).abs() < 1E-6);
    /// ```
    pub fn spatially_resampled(
        &self,
        latitude_spacing_deg: f64,
        longitude_spacing_deg: f64,
    ) -> Result<IONEX, Error> {
        let grid = &self.header.grid;

        let resampled = |linspace: &Linspace, spacing: f64| {
            let spacing = if linspace.spacing < 0.0 {
                -spacing.abs()
            } else {
                spacing.abs()
            };

            Linspace::new(linspace.start, linspace.end, spacing)
                .map_err(|_| Error::InvalidGridSpacing)
        };

        let new_grid = Grid {
            latitude: resampled(&grid.latitude, latitude_spacing_deg)?,
            longitude: resampled(&grid.longitude, longitude_spacing_deg)?,
            altitude: grid.altitude,
        };

        let mut record = Record::default();

        for epoch in self.epoch_iter() {
            for altitude in new_grid.altitude.quantize() {
                for latitude in new_grid.latitude.quantize() {
                    for longitude in new_grid.longitude.quantize() {
                        let point = Point::new(longitude.real_value(), latitude.real_value());

                        match self.spatial_tec(epoch, point, altitude.real_value()) {
                            Ok(tec) => {
                                record.insert(
                                    Key {
                                        epoch,
                                        coordinates: QuantizedCoordinates::from_quantized(
                                            latitude, longitude, altitude,
                                        ),
                                    },
                                    tec,
                                );
                            },
                            Err(Error::MissingData) => {},
                            Err(e) => return Err(e),
                        }
                    }
                }
            }
        }

        let mut ionex = self.with_record(record);
        ionex.header.grid = new_grid;
        Ok(ionex)
    }

    /// Upscale (upsample) or Downscale (downsample) this mutable [IONEX] on the temporal axis,
    /// using linear temporal interpolation. See [Self::temporally_resampled] for more options.
    ///
    /// ## Input
    /// - factor: a positive finite number
    ///    - 0.0: is not valid
    ///    - >1.0: upscaling case. For example, 2.0 means x2 sample rate increase (+100%).
    ///    - and 1.5 means +50% sample rate increase.
    ///    - <1.0: downscaling case. For example, 0.5 means /2 sample rate decrease (-50%).
    pub fn temporal_stretching_mut(&mut self, factor: f64) -> Result<(), Error> {
        if !factor.is_normal() || factor < 0.0 {
            return Err(Error::InvalidStretchFactor);
        }

        let sampling_period = self.header.sampling_period / factor;
        *self = self.temporally_resampled(sampling_period, TemporalInterpolation::Linear)?;
        Ok(())
    }

    // /// Stretchs this mutable [IONEX] both in temporal and spatial dimensions,
    // /// modifying both dimensions.
    // /// When factor > 1.0 this is an upscaling operation, when factor < 1.0,
    // /// this is a downscaling operation.
    // /// This uses both spatial and temporal interpolation to form valid data points.
    // pub fn temporal_spatial_stretching_mut(
    //     &mut self,
    //     temporal_factor: f64,
    //     axis: Axis,
    //     spatial_factor: f64,
    // ) -> Result<(), Error> {
    //     self.spatial_stretching_mut(axis, spatial_factor)?;
    //     self.temporal_stretching_mut(temporal_factor)?;
    //     Ok(())
    // }

    // /// Stretchs this [IONEX] into a new [IONEX], both temporally and spatially stretched,
    // /// modifying both dimensions.
    // /// When factor > 1.0 this is an upscaling operation, when factor < 1.0,
    // /// this is a downscaling operation.
    // /// This uses both spatial and temporal interpolation to form valid data points.
    // pub fn temporally_spatially_stretched(
    //     &self,
    //     temporal_factor: f64,
    //     axis: Axis,
    //     spatial_factor: f64,
    // ) -> Result<Self, Error> {
    //     let mut s = self.clone();
    //     s.temporal_spatial_stretching_mut(temporal_factor, axis, spatial_factor)?;
    //     Ok(s)
    // }

    /// Produces a [TimeSeries] from this [IONEX], describing the temporal axis.
    pub fn timeseries(&self) -> TimeSeries {
        self.header.timeseries()
    }

    /// Returns these coordinates (x: longitude, y: latitude, in decimal degrees)
    /// with longitude wrapped into the longitude range of this map, when this map
    /// covers the 360° longitude range. For example, 190° becomes -170°
    /// on usual worldwide maps (that range from -180° to 180°).
    /// Coordinates are returned unchanged otherwise.
    pub fn wrapped_coordinates(&self, coordinates: Point<f64>) -> Point<f64> {
        let longitudes = &self.header.grid.longitude;
        let (min, max) = longitudes.minmax();

        if (max - min - 360.0).abs() > 1.0E-9 {
            return coordinates;
        }

        let mut longitude = coordinates.x();

        if longitude < min || longitude > max {
            longitude = (longitude - min).rem_euclid(360.0) + min;
        }

        Point::new(longitude, coordinates.y())
    }

    /// Builds the [MapCell] delimited by these grid nodes, which are reordered
    /// whatever the axes orientation. Returns None if one of the 4 corners is not
    /// defined (omitted data, or coordinates that do not exist).
    fn quantized_map_cell(
        &self,
        epoch: Epoch,
        latitudes: (Quantized, Quantized),
        longitudes: (Quantized, Quantized),
        altitude: Quantized,
    ) -> Option<MapCell> {
        let (north, south) = if latitudes.0 >= latitudes.1 {
            (latitudes.0, latitudes.1)
        } else {
            (latitudes.1, latitudes.0)
        };

        let (west, east) = if longitudes.0 <= longitudes.1 {
            (longitudes.0, longitudes.1)
        } else {
            (longitudes.1, longitudes.0)
        };

        let tec_point = |latitude: Quantized, longitude: Quantized| {
            let key = Key {
                epoch,
                coordinates: QuantizedCoordinates::from_quantized(latitude, longitude, altitude),
            };

            let tec = self.record.get(&key)?;

            Some(TecPoint {
                tec: *tec,
                point: Point::new(longitude.real_value(), latitude.real_value()),
            })
        };

        Some(MapCell {
            epoch,
            north_east: tec_point(north, east)?,
            north_west: tec_point(north, west)?,
            south_east: tec_point(south, east)?,
            south_west: tec_point(south, west)?,
        })
    }

    /// Quantized altitude of the fixed altitude layer
    /// (the only layer of 2D maps, the lowest layer of 3D maps).
    fn fixed_altitude(&self) -> Quantized {
        self.header
            .grid
            .altitude
            .quantize()
            .next()
            .unwrap_or(Quantized::auto_scaled(self.header.grid.altitude.start))
    }

    /// Designs a [MapCell] iterator (smallest ROI following the grid quantization)
    /// that allows local interpolation. [MapCell]s are returned in chronological order,
    /// following the grid description (usually North to South and West to East).
    /// In case of 3D maps, this iterates the fixed (first) altitude layer.
    pub fn map_cell_iter(&self) -> Box<dyn Iterator<Item = MapCell> + '_> {
        Box::new(
            self.epoch_iter()
                .flat_map(move |epoch| self.synchronous_map_cell_iter(epoch)),
        )
    }

    /// Obtain a synchronous [MapCell] iterator at specific point in time,
    /// that must exist in this [IONEX]. See [Self::map_cell_iter].
    pub fn synchronous_map_cell_iter(
        &self,
        epoch: Epoch,
    ) -> Box<dyn Iterator<Item = MapCell> + '_> {
        let altitude = self.fixed_altitude();

        let lat_pairs = self.header.grid.latitude.quantize().tuple_windows();
        let long_pairs = self
            .header
            .grid
            .longitude
            .quantize()
            .tuple_windows()
            .collect::<Vec<_>>();

        Box::new(lat_pairs.cartesian_product(long_pairs).filter_map(
            move |(latitudes, longitudes)| {
                self.quantized_map_cell(epoch, latitudes, longitudes, altitude)
            },
        ))
    }

    /// Returns the [MapCell] (grid quantum) that contains these coordinates
    /// (x: longitude, y: latitude in decimal degrees), at this [Epoch] that must exist
    /// in this [IONEX], and for the fixed altitude layer.
    /// This is a direct lookup (no iteration). Longitudes are wrapped on 360° maps
    /// (see [Self::wrapped_coordinates]).
    ///
    /// Returns None when the coordinates are outside this map, when this [Epoch]
    /// does not exist, or if one of the corners was omitted.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
    ///
    /// // Toulouse (France): x = longitude, y = latitude
    /// let toulouse = Point::new(1.44, 43.6);
    ///
    /// let cell = ionex.map_cell_at(noon, toulouse).unwrap();
    ///
    /// assert_eq!(cell.south_west.point, Point::new(0.0, 42.5));
    /// assert_eq!(cell.north_east.point, Point::new(5.0, 45.0));
    /// assert!(cell.contains(&Geometry::Point(toulouse)));
    /// ```
    pub fn map_cell_at(&self, epoch: Epoch, coordinates: Point<f64>) -> Option<MapCell> {
        self.altitude_map_cell_at(epoch, coordinates, self.fixed_altitude())
    }

    /// [MapCell] that contains these coordinates, at this [Epoch]
    /// that must exist, for this altitude layer.
    fn altitude_map_cell_at(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        altitude: Quantized,
    ) -> Option<MapCell> {
        let coordinates = self.wrapped_coordinates(coordinates);

        let latitudes = self
            .header
            .grid
            .latitude
            .quantized_bracket(coordinates.y())?;

        let longitudes = self
            .header
            .grid
            .longitude
            .quantized_bracket(coordinates.x())?;

        self.quantized_map_cell(epoch, latitudes, longitudes, altitude)
    }

    /// Spatial interpolation at an [Epoch] that must exist in this [IONEX],
    /// at any coordinates and altitude within this map.
    /// For 3D maps, the two surrounding altitude layers are linearly interpolated.
    fn spatial_tec(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        altitude_km: f64,
    ) -> Result<TEC, Error> {
        let grid = &self.header.grid;
        let coordinates = self.wrapped_coordinates(coordinates);

        if !grid.latitude.contains(coordinates.y()) || !grid.longitude.contains(coordinates.x()) {
            return Err(Error::OutsideSpatialBoundaries);
        }

        let (alt0, alt1) = grid
            .altitude
            .quantized_bracket(altitude_km)
            .ok_or(Error::OutsideSpatialBoundaries)?;

        let layer_tec = |altitude: Quantized| {
            self.altitude_map_cell_at(epoch, coordinates, altitude)
                .ok_or(Error::MissingData)?
                .spatial_tec_interp(coordinates)
        };

        let tec0 = layer_tec(alt0)?;

        if alt0 == alt1 {
            return Ok(tec0);
        }

        let tec1 = layer_tec(alt1)?;

        let (z0, z1) = (alt0.real_value(), alt1.real_value());
        Ok(tec0.lerp(&tec1, (altitude_km - z0) / (z1 - z0)))
    }

    /// Interpolates the [TEC] at any instant, coordinates and altitude.
    fn interpolate(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        altitude_km: f64,
        method: TemporalInterpolation,
    ) -> Result<TEC, Error> {
        let (t0, t1) = self
            .record
            .epoch_bracket(epoch)
            .ok_or(Error::OutsideTemporalBoundaries)?;

        if t0 == t1 {
            return self.spatial_tec(t0, coordinates, altitude_km);
        }

        let dt = (t1 - t0).to_seconds();
        let w = (epoch - t0).to_seconds() / dt;

        match method {
            TemporalInterpolation::NearestMap => {
                let nearest = if w <= 0.5 { t0 } else { t1 };
                self.spatial_tec(nearest, coordinates, altitude_km)
            },
            TemporalInterpolation::Linear => {
                let tec0 = self.spatial_tec(t0, coordinates, altitude_km)?;
                let tec1 = self.spatial_tec(t1, coordinates, altitude_km)?;
                Ok(tec0.lerp(&tec1, w))
            },
            TemporalInterpolation::RotatedMaps => {
                let rotated = |t: Epoch| {
                    let shift_deg =
                        (epoch - t).to_seconds() * TemporalInterpolation::SUN_FIXED_ROTATION_DEG_S;
                    Point::new(coordinates.x() + shift_deg, coordinates.y())
                };

                let tec0 = self.spatial_tec(t0, rotated(t0), altitude_km)?;
                let tec1 = self.spatial_tec(t1, rotated(t1), altitude_km)?;
                Ok(tec0.lerp(&tec1, w))
            },
        }
    }

    /// Interpolates the [TEC] at any instant within the temporal axis,
    /// and any coordinates (x: longitude, y: latitude in decimal degrees) within this map,
    /// using the [TemporalInterpolation] strategy recommended by the IONEX specifications
    /// ([TemporalInterpolation::RotatedMaps]). See [Self::tec_at_with] to select another strategy.
    ///
    /// Spatial interpolation uses the 4 surrounding grid nodes (see [MapCell::spatial_tec_interp]),
    /// grid nodes are returned as is. The TEC RMS is interpolated as well, when defined.
    ///
    /// This applies to 2D maps (or the fixed altitude layer). For 3D maps, you must
    /// specify the altitude with [Self::volumetric_tec_at], otherwise this returns
    /// [Error::AltitudeRequired].
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// // x: longitude, y: latitude
    /// let toulouse = Point::new(1.44, 43.6);
    ///
    /// // anywhere in time
    /// let t = Epoch::from_str("2022-01-02T12:34:56 UTC").unwrap();
    /// let tec = ionex.tec_at(t, toulouse).unwrap();
    /// assert!(tec.tecu() > 0.0);
    ///
    /// // on the grid: no interpolation
    /// let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
    /// let node = Point::new(0.0, 42.5);
    ///
    /// let key = Key::from_decimal_degrees_km(noon, 42.5, 0.0, 350.0);
    /// let expected = ionex.record.get(&key).unwrap();
    ///
    /// assert_eq!(ionex.tec_at(noon, node).unwrap().tecu(), expected.tecu());
    ///
    /// // outside of the temporal axis
    /// let t = Epoch::from_str("2022-01-04T00:00:00 UTC").unwrap();
    /// assert!(ionex.tec_at(t, toulouse).is_err());
    /// ```
    pub fn tec_at(&self, epoch: Epoch, coordinates: Point<f64>) -> Result<TEC, Error> {
        self.tec_at_with(epoch, coordinates, TemporalInterpolation::default())
    }

    /// Interpolates the [TEC] at any instant and coordinates, like [Self::tec_at],
    /// using the [TemporalInterpolation] strategy of your choice.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// let toulouse = Point::new(1.44, 43.6);
    /// let t = Epoch::from_str("2022-01-02T12:30:00 UTC").unwrap();
    ///
    /// let linear = ionex.tec_at_with(t, toulouse, TemporalInterpolation::Linear).unwrap();
    /// let rotated = ionex.tec_at_with(t, toulouse, TemporalInterpolation::RotatedMaps).unwrap();
    /// let nearest = ionex.tec_at_with(t, toulouse, TemporalInterpolation::NearestMap).unwrap();
    /// ```
    pub fn tec_at_with(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        method: TemporalInterpolation,
    ) -> Result<TEC, Error> {
        if self.is_3d() {
            return Err(Error::AltitudeRequired);
        }

        self.interpolate(
            epoch,
            coordinates,
            self.fixed_altitude().real_value(),
            method,
        )
    }

    /// Interpolates the [TEC] at any instant, coordinates (x: longitude, y: latitude in decimal
    /// degrees) and altitude (in kilometers) within the volume described by a 3D IONEX.
    /// The 4 surrounding nodes of the two surrounding altitude layers are used
    /// (trilinear interpolation), and the [TemporalInterpolation] strategy of your choice.
    ///
    /// This also applies to 2D maps, but the altitude must then match the fixed altitude
    /// of the map, otherwise this returns [Error::OutsideSpatialBoundaries].
    pub fn volumetric_tec_at(
        &self,
        epoch: Epoch,
        coordinates: Point<f64>,
        altitude_km: f64,
        method: TemporalInterpolation,
    ) -> Result<TEC, Error> {
        self.interpolate(epoch, coordinates, altitude_km, method)
    }

    /// Returns the Ionospheric Pierce Point ([IPP]) of the line of sight from this
    /// receiver (x: longitude, y: latitude in decimal degrees), with given elevation
    /// and azimuth (in degrees), using the single layer model described by this [IONEX]:
    /// the shell altitude is the fixed altitude of the map,
    /// and the Earth radius is [Header::base_radius_km].
    pub fn ionospheric_pierce_point(
        &self,
        receiver: Point<f64>,
        elevation_deg: f64,
        azimuth_deg: f64,
    ) -> Result<IPP, Error> {
        IPP::new(
            receiver,
            elevation_deg,
            azimuth_deg,
            self.header.base_radius_km as f64,
            self.fixed_altitude().real_value(),
        )
    }

    /// Returns the Slant [TEC] along the line of sight from this receiver
    /// (x: longitude, y: latitude in decimal degrees) with given elevation and azimuth (in degrees),
    /// at any instant within the temporal axis.
    ///
    /// The vertical TEC is interpolated at the Ionospheric Pierce Point ([Self::ionospheric_pierce_point])
    /// with [Self::tec_at] and converted to slant TEC with the single layer obliquity factor
    /// (1/cos(z'), see [IPP::obliquity_factor]). The RMS is scaled as well.
    /// This only applies to 2D maps.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// let t = Epoch::from_str("2022-01-02T12:34:56 UTC").unwrap();
    /// let toulouse = Point::new(1.44, 43.6);
    ///
    /// // satellite at 30° elevation, South-East
    /// let stec = ionex.slant_tec_at(t, toulouse, 30.0, 135.0).unwrap();
    ///
    /// // satellite at zenith: STEC is VTEC
    /// let stec = ionex.slant_tec_at(t, toulouse, 90.0, 0.0).unwrap();
    /// let vtec = ionex.tec_at(t, toulouse).unwrap();
    /// assert!((stec.tecu() - vtec.tecu()).abs() < 1E-6);
    ///
    /// // ionospheric delay on GPS L1 (meters)
    /// let delay_m = stec.ionospheric_delay_m(1575.42E6);
    /// ```
    pub fn slant_tec_at(
        &self,
        epoch: Epoch,
        receiver: Point<f64>,
        elevation_deg: f64,
        azimuth_deg: f64,
    ) -> Result<TEC, Error> {
        let ipp = self.ionospheric_pierce_point(receiver, elevation_deg, azimuth_deg)?;
        let vtec = self.tec_at(epoch, ipp.coordinates)?;
        Ok(vtec.scaled(ipp.obliquity_factor()))
    }

    /// Returns the ionospheric delay (in meters) experienced by a signal of this
    /// frequency (in Hz), along the line of sight from this receiver
    /// (x: longitude, y: latitude in decimal degrees) with given elevation and azimuth
    /// (in degrees), at any instant within the temporal axis.
    /// See [Self::slant_tec_at] and [TEC::ionospheric_delay_m].
    pub fn ionospheric_delay_m(
        &self,
        epoch: Epoch,
        receiver: Point<f64>,
        elevation_deg: f64,
        azimuth_deg: f64,
        frequency_hz: f64,
    ) -> Result<f64, Error> {
        let stec = self.slant_tec_at(epoch, receiver, elevation_deg, azimuth_deg)?;
        Ok(stec.ionospheric_delay_m(frequency_hz))
    }

    /// Form a possibly interpolated [MapCell] at specified point in time, wrapping this ROI.
    /// The ROI is described by a [Geometry], which can be complex: the returned [MapCell]
    /// is its bounding rectangle, and the TEC is interpolated at each corner.
    /// Unlike [Self::unitary_roi_at], the returned [MapCell] may not be a grid quantum,
    /// it may be smaller or wider.
    ///
    /// ## Input
    /// - epoch: the returned cell will be synchronous to this [Epoch], that must be within
    /// the temporal axis. When it does not line up with a map, the maps are linearly
    /// interpolated in time ([TemporalInterpolation::Linear]).
    /// - roi: [Geometry] defining the local region we want to wrap (in decimal degrees),
    /// that must be within this map.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// let t = Epoch::from_str("2022-01-02T12:30:00 UTC").unwrap();
    ///
    /// // France (roughly)
    /// let roi = Rect::new(coord!(x: -5.0, y: 42.0), coord!(x: 8.0, y: 51.0));
    ///
    /// let cell = ionex.roi_at(t, Geometry::Rect(roi)).unwrap();
    /// assert_eq!(cell.bounding_rect_degrees(), roi);
    /// ```
    pub fn roi_at(&self, epoch: Epoch, roi: Geometry) -> Result<MapCell, Error> {
        let roi = roi.bounding_rect().ok_or(Error::UndefinedBoundaries)?;

        let ((ne_lat, ne_long), (se_lat, se_long), (sw_lat, sw_long), (nw_lat, nw_long)) =
            rectangle_quadrant_decomposition(roi);

        let altitude_km = self.fixed_altitude().real_value();

        let tec_point = |latitude: f64, longitude: f64| -> Result<TecPoint, Error> {
            let point = Point::new(longitude, latitude);

            Ok(TecPoint {
                point,
                tec: self.interpolate(epoch, point, altitude_km, TemporalInterpolation::Linear)?,
            })
        };

        Ok(MapCell {
            epoch,
            north_east: tec_point(ne_lat, ne_long)?,
            north_west: tec_point(nw_lat, nw_long)?,
            south_east: tec_point(se_lat, se_long)?,
            south_west: tec_point(sw_lat, sw_long)?,
        })
    }

    /// Obtain the [MapCell] (smallest map ROI) at provided point in time and containing provided coordinates.
    ///
    /// ## Input
    /// - epoch: [Epoch] that must fit within the temporal axis.
    /// If this instant exists in this [IONEX], this process does not require temporal interpolation.
    /// Otherwise, we return the [MapCell] linearly interpolated in time
    /// (see [MapCell::temporally_interpolated]), between the two surrounding maps.
    /// - coordinates: [Point] (x: longitude, y: latitude in decimal degrees) that must be
    /// within this map.
    ///
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::*;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// let t = Epoch::from_str("2022-01-02T12:30:00 UTC").unwrap();
    /// let cell = ionex.unitary_roi_at(t, Point::new(1.44, 43.6)).unwrap();
    /// assert_eq!(cell.epoch, t);
    /// ```
    pub fn unitary_roi_at(&self, epoch: Epoch, coordinates: Point<f64>) -> Option<MapCell> {
        let (t0, t1) = self.record.epoch_bracket(epoch)?;

        let cell0 = self.map_cell_at(t0, coordinates)?;

        if t0 == t1 {
            return Some(cell0);
        }

        let cell1 = self.map_cell_at(t1, coordinates)?;
        cell0.temporally_interpolated(epoch, &cell1).ok()
    }

    /// Obtain the best suited [MapCell] spatially wrapping this Geometry that contains following [Geometry].
    ///
    /// ## Input
    /// - epoch: [Epoch] that must exist in this [IONEX]
    /// - geometry: possibly complex [Geometry] to contain (completely).
    pub fn wrapping_map_cell(&self, epoch: Epoch, geometry: &Geometry<f64>) -> Option<MapCell> {
        for cell in self.synchronous_map_cell_iter(epoch) {
            if cell.contains(&geometry) {
                return Some(cell);
            }
        }

        None
    }
}

/// Merge two [IONEX] structures into one.
/// This requires a few mandatory steps:
/// - reference systems must match
/// - maps dimension must match
/// - both must use the same mapping function
///
/// Different sampling rate are supported, because the IONEX
/// description allows to describe that, but you will windup with
/// non constant sample rates.
#[cfg(feature = "qc")]
impl gnss_qc_traits::Merge for IONEX {
    fn merge(&self, rhs: &Self) -> Result<Self, gnss_qc_traits::MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }

    fn merge_mut(&mut self, rhs: &Self) -> Result<(), gnss_qc_traits::MergeError> {
        self.header.merge_mut(&rhs.header)?;
        self.record.merge_mut(&rhs.record)?;

        match self.attributes {
            Some(ref mut prods) => {
                if let Some(rhs) = &rhs.attributes {
                    prods.merge_mut(rhs)?;
                }
            },
            None => {
                if let Some(rhs) = &rhs.attributes {
                    self.attributes = Some(rhs.clone());
                }
            },
        }

        // add new comments
        for comment in rhs.comments.iter() {
            if !self.comments.contains(&comment) {
                self.comments.push(comment.clone());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::{
        div_ceil, fmt_comment, prelude::*, quadrant_to_rectangle, rectangle_quadrant_decomposition,
    };

    #[test]
    fn fmt_comments_singleline() {
        for desc in [
            "test",
            "just a basic comment",
            "just another lengthy comment blahblabblah",
        ] {
            let comment = fmt_comment(desc);

            assert!(
                comment.len() >= 60,
                "comments should be at least 60 byte long"
            );

            assert_eq!(
                comment.find("COMMENT"),
                Some(60),
                "comment marker should located @ 60"
            );
        }
    }

    #[test]
    fn fmt_wrapped_comments() {
        for desc in [
            "just trying to form a very lengthy comment that will overflow since it does not fit in a single line",
            "just trying to form a very very lengthy comment that will overflow since it does fit on three very meaningful lines. Imazdmazdpoakzdpoakzpdokpokddddddddddddddddddaaaaaaaaaaaaaaaaaaaaaaa",
        ] {
            let nb_lines = num_integer::div_ceil(desc.len(), 60);
            let comments = fmt_comment(desc);

            assert_eq!(comments.lines().count(), nb_lines);

            for line in comments.lines() {
                assert!(
                    line.len() >= 60,
                    "comment line should be at least 60 byte long"
                );
                assert_eq!(
                    line.find("COMMENT"),
                    Some(60),
                    "comment marker should located @ 60"
                );
            }
        }
    }

    #[test]
    fn rectangle_decomposition() {
        for (rect, corners) in [
            (
                Rect::new(coord!(x: -30.0, y: -30.0), coord!(x: 30.0, y: 30.0)),
                ((30.0, 30.0), (-30.0, 30.0), (-30.0, -30.0), (30.0, -30.0)),
            ),
            (
                Rect::new(coord!(x: -40.0, y: -40.0), coord!(x: 40.0, y: 50.0)),
                ((50.0, 40.0), (-40.0, 40.0), (-40.0, -40.0), (50.0, -40.0)),
            ),
        ] {
            let (ne, se, sw, nw) = rectangle_quadrant_decomposition(rect);
            assert_eq!((ne, se, sw, nw), corners, "failed for {:?}", rect);
            assert_eq!(quadrant_to_rectangle((ne, se, sw, nw)), rect);
        }
    }
}
