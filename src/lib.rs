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
    collections::BTreeMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};

use itertools::Itertools;

use geo::{coord, BoundingRect, Geometry, LineString, Point, Polygon, Rect};

#[cfg(feature = "flate2")]
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};

use hifitime::prelude::Epoch;

use crate::{
    cell::{MapCell, MapPoint},
    coordinates::QuantizedCoordinates,
    error::{FormattingError, ParsingError},
    file_attributes::{FileAttributes, Region},
    header::Header,
    key::Key,
    quantized::Quantized,
    record::Record,
    tec::TEC,
};

pub mod prelude {
    // export
    pub use crate::{
        bias::BiasSource,
        cell::MapCell,
        error::{FormattingError, ParsingError, Error},
        file_attributes::*,
        grid::Grid,
        header::Header,
        ionosphere::IonosphereParameters,
        key::Key,
        linspace::Linspace,
        mapf::MappingFunction,
        record::Record,
        system::ReferenceSystem,
        tec::TEC,
        version::Version,
        Comments, IONEX,
    };

    // pub re-export
    pub use geo::{
        algorithm::contains::Contains, coord, BoundingRect, GeodesicArea, Geometry, LineString,
        Point, Polygon, Rect,
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
    if r == 0 {
        q
    } else {
        q + 1
    }
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
/// // to obtain TEC values at any coordinates, you
/// // should use the [MapCell] local region (rectangle quanta)
/// // that offers many functions based off the Geo crate.
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

        let extension = if let Some(attributes) = &self.attributes {
            #[cfg(feature = "flate2")]
            if attributes.gzip_compressed {
                ".gz"
            } else {
                ""
            }
        } else {
            ""
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

        let region = Region::WorldWide; // TODO: study the grid specs

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
    ///   * [Self::to_regional_roi]: to reduce a larger (possibly [Region::GlobalWorldWide] map)
    /// to a given ROI
    ///   * [Self::to_worldwide_map]: to enlarge a possibly [Region::Regional] map to a [Region::GlobalWorldWide] IONEX.
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
            attributes.region == Region::WorldWide
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
            attributes.region = Region::WorldWide;
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
    /// [Polygon::bounding_rect] must be defined for this operation to work correctly.
    pub fn to_regional_ionex(&self, roi: Polygon) -> Option<IONEX> {
        let mut ionex = IONEX::default();

        let bounding_rect = roi.bounding_rect()?;

        let (min_long, min_lat) = (bounding_rect.min().x, bounding_rect.min().y);
        let (max_long, max_lat) = (bounding_rect.max().x, bounding_rect.max().y);

        // copy attributes
        ionex.attributes = self.attributes.clone();

        if let Some(attributes) = &mut ionex.attributes {
            attributes.region = Region::Regional;
        }

        // copy header
        ionex.header = self.header.clone();

        // rework header
        ionex.header.grid.latitude.start = max_lat;
        ionex.header.grid.latitude.end = min_lat;
        ionex.header.grid.longitude.start = min_long;
        ionex.header.grid.longitude.end = max_long;

        let roi = Geometry::Polygon(roi);

        // restrict area
        let cells = self
            .map_cell_iter()
            .filter(|cell| cell.contains(&roi))
            .collect::<Vec<_>>();

        ionex.record = Record::from_map_cells(
            self.header.grid.altitude.start,
            min_lat,
            max_lat,
            min_long,
            max_long,
            &cells,
        );

        Some(ionex)
    }

    /// Stretch this [IONEX] into a new file [IONEX], modifying grid
    /// spatial precision.
    /// ## Inputs
    /// - stretch > 1.0: increases precision.  
    /// The 2D planar interpolation is applied to define the TEC as originally
    /// non existing coordinates.
    /// - stretch < 1.0: reduces precision.  
    /// NB: work in progress, not validated yet.
    pub fn grid_precision_stretch(&self, stretch_factor: f64) -> IONEX {
        let mut s = self.clone();
        s.grid_precision_stretch_mut(stretch_factor);
        s
    }

    /// Stretch this mutable [IONEX], modifying the grid precision.
    /// See [Self::grid_precision_stretch] for more information.  
    /// NB: work in progress, not validated yet.
    pub fn grid_precision_stretch_mut(&mut self, stretch_factor: f64) {
        // update grid
        self.header.grid.latitude.start *= stretch_factor;
        self.header.grid.latitude.end *= stretch_factor;
        self.header.grid.longitude.start *= stretch_factor;
        self.header.grid.longitude.end *= stretch_factor;

        // // update map
        // for epoch in self.record.epochs_iter() {
        // }
    }

    /// Designs a [MapCell] iterator (micro rectangle region made of 4 local points)
    /// that can then be interpolated.
    pub fn map_cell_iter(&self) -> Box<dyn Iterator<Item = MapCell> + '_> {
        let timeseries = self.header.timeseries();

        let lat_pairs = self.header.grid.latitude.quantize().tuple_windows();

        let long_pairs = self.header.grid.longitude.quantize().tuple_windows();

        let fixed_altitude_km = self.header.grid.altitude.start;
        let fixed_altitude_q = Quantized::auto_scaled(fixed_altitude_km);

        Box::new(
            timeseries
                .cartesian_product(lat_pairs.cartesian_product(long_pairs))
                .filter_map(move |(epoch, ((lat1, lat2), (long1, long2)))| {
                    let northeast = Key {
                        epoch,
                        coordinates: QuantizedCoordinates::from_quantized(
                            lat1,
                            long1,
                            fixed_altitude_q,
                        ),
                    };

                    let northeast = self.record.get(&northeast)?;

                    let northwest = Key {
                        epoch,
                        coordinates: QuantizedCoordinates::from_quantized(
                            lat1,
                            long2,
                            fixed_altitude_q,
                        ),
                    };

                    let northwest = self.record.get(&northwest)?;

                    let southeast = Key {
                        epoch,
                        coordinates: QuantizedCoordinates::from_quantized(
                            lat2,
                            long2,
                            fixed_altitude_q,
                        ),
                    };

                    let southeast = self.record.get(&southeast)?;

                    let southwest = Key {
                        epoch,
                        coordinates: QuantizedCoordinates::from_quantized(
                            lat2,
                            long2,
                            fixed_altitude_q,
                        ),
                    };

                    let southwest = self.record.get(&southwest)?;

                    Some(MapCell {
                        epoch,
                        north_east: MapPoint {
                            tec: *northeast,
                            point: Point::new(long1.real_value(), lat1.real_value()),
                        },
                        north_west: MapPoint {
                            tec: *northwest,
                            point: Point::new(long2.real_value(), lat1.real_value()),
                        },
                        south_east: MapPoint {
                            tec: *southeast,
                            point: Point::new(long1.real_value(), lat2.real_value()),
                        },
                        south_west: MapPoint {
                            tec: *southwest,
                            point: Point::new(long2.real_value(), lat2.real_value()),
                        },
                    })
                }),
        )
    }

    /// Returns the [MapCell] that contains following [Geometry].
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

    /// Obtain a synchronous [MapCell] iterator at specific point in time.
    pub fn synchronous_map_cell_iter(
        &self,
        epoch: Epoch,
    ) -> Box<dyn Iterator<Item = MapCell> + '_> {
        Box::new(
            self.map_cell_iter()
                .filter_map(move |v| if v.epoch == epoch { Some(v) } else { None }),
        )
    }

    /// Interpolate TEC values for all discrete coordinates described by the following [LineString]
    /// (in decimal degrees), at specific point in time that must exist within this record.
    /// Otherwise, you should use [Self::temporal_spatial_area_interpolation] to also
    /// use temporal interpolation from two existing data points.
    /// ```
    /// use std::str::FromStr;
    /// use ionex::prelude::{IONEX, Epoch, LineString, coord, Contains};
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///    .unwrap();
    ///
    /// // ROI (ddeg) must be within borders
    /// let roi_ddeg = LineString::new(vec![
    ///     coord! { x: -50.0, y: -23.0 },
    ///     coord! { x: -50.25, y: -23.1 },
    ///     coord! { x: -50.5, y: -23.2 },
    /// ]);
    ///
    /// // you can double check that
    /// assert!(ionex.bounding_rect_degrees().contains(&roi_ddeg));
    ///
    /// // Epoch must exist in the record
    /// let noon = Epoch::from_str("2022-01-02T12:00:00 UTC")
    ///     .unwrap();
    ///
    ///
    /// ```
    pub fn spatial_area_interpolation(
        &self,
        area: &LineString,
        epoch: Epoch,
    ) -> BTreeMap<Key, TEC> {
        let mut values = BTreeMap::new();

        let fixed_altitude_km = self.header.grid.altitude.start;

        // for all requested coordinates
        for point in area.points() {
            let geo = Geometry::Point(point);

            let key = Key::from_decimal_degrees_km(epoch, point.y(), point.x(), fixed_altitude_km);

            for cell in self.synchronous_map_cell_iter(epoch) {
                if cell.contains(&geo) {
                    let tec = cell.spatial_interpolation(point);
                    values.insert(key, tec);
                }
            }
        }

        values
    }

    /// Interpolate TEC values for all discrete coordinates described by the following [LineString]
    /// (in decimal degrees), at specific point in time that does exist within this record.
    /// We will interpolate the two neighbouring data points, which is not feasible at day boundaries.
    pub fn temporal_spatial_area_interpolation(
        &self,
        area: &LineString,
        epoch: Epoch,
    ) -> BTreeMap<Key, TEC> {
        let mut values = BTreeMap::new();

        let (min_t, max_t) = (
            (epoch - self.header.sampling_period),
            (epoch + self.header.sampling_period),
        );

        let (min_t, max_t) = (
            min_t.round(self.header.sampling_period),
            max_t.round(self.header.sampling_period),
        );

        // for all requested coordinates
        for point in area.points() {
            let geo = Geometry::Point(point);

            for cell_t0 in self.synchronous_map_cell_iter(min_t) {
                if cell_t0.contains(&geo) {
                    for cell_t1 in self.synchronous_map_cell_iter(max_t) {
                        if cell_t1.contains(&geo) {
                            if let Some(interpolated) =
                                cell_t0.temporal_spatial_interpolation(epoch, point, &cell_t1)
                            {
                                let key = Key {
                                    epoch,
                                    coordinates: QuantizedCoordinates::from_decimal_degrees(
                                        point.y(),
                                        point.x(),
                                        self.header.grid.altitude.start,
                                    ),
                                };

                                values.insert(key, interpolated);
                            }
                        }
                    }
                }
            }
        }

        values
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
    use crate::{div_ceil, fmt_comment, prelude::*};

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
        for desc in ["just trying to form a very lengthy comment that will overflow since it does not fit in a single line",
            "just trying to form a very very lengthy comment that will overflow since it does fit on three very meaningful lines. Imazdmazdpoakzdpoakzpdokpokddddddddddddddddddaaaaaaaaaaaaaaaaaaaaaaa"] {
            let nb_lines = div_ceil(desc.len(), 60);
            let comments = fmt_comment(desc);

            assert_eq!(comments.lines().count(), nb_lines);

            for line in comments.lines() {
                assert!(line.len() >= 60, "comment line should be at least 60 byte long");
                assert_eq!(line.find("COMMENT"), Some(60), "comment marker should located @ 60");
            }
        }
    }

    #[test]
    #[ignore]
    fn regional_ionex() {
        let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
            panic!("Failed to parse CKMG0020: {}", e);
        });

        let roi = Rect::new(coord!(x: -180.0, y: -85.0), coord!(x: 180.0, y: -82.5));

        let regional = ionex.to_regional_ionex(roi.into())
            .unwrap();

        // dump
        regional.to_file("region.txt").unwrap();

        // parse
        let parsed = IONEX::from_file("region.txt").unwrap_or_else(|e| {
            panic!("Failed to parse region.txt: {}", e);
        });
    }

}
