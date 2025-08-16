#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nav-solutions/.github/master/logos/logo2.jpg"
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

/*
 * IONEX is part of the nav-solutions framework.
 *
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/nav-solutions/ionex/graphs/contributors),
 * licensed under Mozilla Public license V2.
 *
 * Documentation: https://github.com/nav-solutions/ionex
 */

extern crate num_derive;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate gnss_rs as gnss;
extern crate num;

pub mod bias;
pub mod error;
pub mod grid;
pub mod header;
pub mod key;
pub mod linspace;
pub mod mapf;
pub mod production;
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

use geo::{coord, Rect};
use itertools::Itertools;

#[cfg(feature = "flate2")]
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};

use std::collections::BTreeMap;

use hifitime::prelude::Epoch;

use crate::{
    cell::MapCell,
    error::{FormattingError, ParsingError},
    header::Header,
    key::Key,
    production::{ProductionAttributes, Region},
    record::Record,
    tec::TEC,
};

pub mod prelude {
    // export
    pub use crate::{
        bias::BiasSource,
        cell::MapCell,
        coordinates::QuantizedCoordinates,
        error::{FormattingError, ParsingError},
        grid::Grid,
        header::Header,
        ionosphere::IonosphereParameters,
        key::Key,
        linspace::Linspace,
        mapf::MappingFunction,
        production::*,
        quantized::Quantized,
        record::Record,
        system::ReferenceSystem,
        tec::TEC,
        version::Version,
        Comments, IONEX,
    };

    // pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries};
}

/// IONEX comments are readable descriptions.
pub type Comments = Vec<String>;

/// macro to format one header line or a comment
pub(crate) fn fmt_ionex(content: &str, marker: &str) -> String {
    if content.len() < 60 {
        format!("{:<padding$}{}", content, marker, padding = 60)
    } else {
        let mut string = String::new();
        let nb_lines = num_integer::div_ceil(content.len(), 60);
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

#[derive(Clone, Default, Debug)]
/// [IONEX] is composed of a [Header] section and a [Record] section.
/// It is the discrete estimation of the Total Electron Content (TEC)
/// over a plane layer or volume of the ionosphere.
///
/// ```
/// use geo::{Rect, coord};
/// use ionex::prelude::*;
///
/// // Parse Global/worldwide map
/// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
///     .unwrap();
///
/// // header contains high level information
/// // like file standard revision:
/// assert_eq!(ionex.header.version.major, 2);
/// assert_eq!(ionex.header.version.minor, 11);
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
/// // this file is named acoording to IGS standards
/// let descriptor = ionex.production.unwrap();
///
/// // convert to region of interest (in decimal degrees)
/// let europe = Rect::new(coord!(x: -23.0, y: -1.0), coord!(x: -25.0, y: -2.0));
///
/// let regional = ionex.to_region_degrees(europe);
///
/// // Convenient helper to follow standard conventions
/// let filename = regional.standardized_filename();
///
/// // Convenient dump function
/// let fd = File::create("region.txt").unwrap();
/// let mut writer = BufWriter::new(fd);
///
/// regional.format(&writer)
///     .unwrap_or_else(|e| {
///         panic!("failed to format region of interest");
///     });
///
/// // parse back
/// let _ = IONEX::from_file("region.txt")
///     .unwrap_or_else(|e| {
///         panic!("failed to parse region of interest");
///     });
/// ```
pub struct IONEX {
    /// [Header] gives general information and describes following content.
    pub header: Header,

    /// IONEX [Record].
    pub record: Record,

    /// [Comments] found in file record
    pub comments: Comments,

    /// [ProductionAttributes] resolved for file names that follow
    /// according to the standards.
    pub production: Option<ProductionAttributes>,
}

impl IONEX {
    /// Builds a new [IONEX] struct from given header & body sections.
    pub fn new(header: Header, record: Record) -> Self {
        Self {
            header,
            record,
            production: None,
            comments: Default::default(),
        }
    }

    /// Copy and return this [IONEX] with updated [Header].
    pub fn with_header(&self, header: Header) -> Self {
        Self {
            header,
            record: self.record.clone(),
            comments: self.comments.clone(),
            production: self.production.clone(),
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
            production: self.production.clone(),
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

    /// Returns Total Electron Content ([TEC]) Iterator
    pub fn tec_maps_iter(&self) -> Box<dyn Iterator<Item = (&Key, &TEC)> + '_> {
        Box::new(self.record.iter())
    }

    /// Returns a file name that would describe [Self] according to the
    /// standards.
    pub fn standardized_filename(&self) -> String {
        let (agency, region, year, doy) = if let Some(production) = &self.production {
            (
                production.agency.clone(),
                production.region,
                production.year - 2000,
                production.doy,
            )
        } else {
            ("XXX".to_string(), Region::default(), 0, 0)
        };

        let extension = if let Some(production) = &self.production {
            #[cfg(feature = "flate2")]
            if production.gzip_compressed {
                ".gz"
            } else {
                ""
            }
        } else {
            ""
        };

        format!("{}{}{:03}0.{:02}I{}", agency, region, doy, year, extension)
    }

    /// Guesses File [ProductionAttributes] from actual record content.
    /// This is useful to generate a standardized file name, from good data content
    /// parsed from files that do not follow the standard naming conventions.
    /// The agency is only determined by the file name so you should provide one,
    /// and it must be a at least 3 letter code.
    pub fn guess_production_attributes(&self, agency: &str) -> Option<ProductionAttributes> {
        if agency.len() < 3 {
            return None;
        }

        let first_epoch = self.record.first_epoch()?;
        let year = first_epoch.year();
        let doy = first_epoch.day_of_year().round() as u32;

        let region = Region::Global; // TODO: study the grid specs

        Some(ProductionAttributes {
            doy,
            region,
            year: year as u32,
            agency: agency.to_string(),

            #[cfg(feature = "flate2")]
            gzip_compressed: if let Some(attributes) = &self.production {
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
            production: Default::default(),
        })
    }

    /// Format [RINEX] into writable I/O using efficient buffered writer
    /// and following standard specifications. The revision to be followed is defined
    /// in [Header] section. This is the mirror operation of [Self::parse].
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        self.header.format(writer)?;

        // format all comments at beginning of file
        for comment in self.comments.iter() {
            writeln!(writer, "{}", fmt_ionex(comment, "COMMENT"),)?;
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
    /// will truly be complete. Otherwise [ProductionAttributes] cannot be fully determined.
    /// If you want or need to you can either
    ///  1. define it yourself with further customization
    ///  2. use the smart guesser (after parsing): [Self::guess_production_attributes]
    ///
    /// This is typically needed in data production contexts.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<IONEX, ParsingError> {
        let path = path.as_ref();

        // deduce all we can from file name
        let file_attributes = match path.file_name() {
            Some(filename) => {
                let filename = filename.to_string_lossy().to_string();
                if let Ok(prod) = ProductionAttributes::from_str(&filename) {
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
        ionex.production = file_attributes;

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
    /// Other useful links are in data production contexts:
    ///   * [Self::standard_filename] to generate a standardized filename
    ///   * [Self::guess_production_attributes] helps generate standardized filenames for
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
    /// use ionex::prelude::Rinex;
    ///
    /// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///     .unwrap();
    ///
    /// assert!(ionex.is_2d_maps());
    ///
    /// // fixed altitude IONEX (=single isosurface)
    /// assert_eq!(ionex.header.grid.height.start, 350.0);
    /// assert_eq!(ionex.header.grid.height.end, 350.0);
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
    /// assert_eq!(ionex.header.mapping, None);
    /// ```
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub fn from_gzip_file<P: AsRef<Path>>(path: P) -> Result<IONEX, ParsingError> {
        let path = path.as_ref();

        // deduce all we can from file name
        let file_attributes = match path.file_name() {
            Some(filename) => {
                let filename = filename.to_string_lossy().to_string();
                if let Ok(prod) = ProductionAttributes::from_str(&filename) {
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
        ionex.production = file_attributes;

        Ok(ionex)
    }

    /// Dumps and gzip encodes [IONEX] into writable local file,
    /// using efficient buffered formatting.
    /// This is the mirror operation of [Self::from_gzip_file].
    /// ```
    /// // Read a IONEX and dump it without any modifications
    /// use rinex::prelude::*;
    ///
    /// let ionex = IONEX::from_file("data/IONEX/V1/CKMG0020.22I.gz")
    ///   .unwrap();
    ///
    /// assert!(ionex.to_gzip_file("test.txt.gz").is_ok());
    /// ```
    ///
    /// Other useful links are in data production contexts:
    ///   * [Self::standard_filename] to generate a standardized filename
    ///   * [Self::guess_production_attributes] helps generate standardized filenames for
    ///     files that do not follow naming conventions
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

    /// Describe the planary map borders as [Retc]angle. This uses
    /// the [Header] description and assumes the following map respects
    /// that description.
    pub fn map_borders_degrees(&self) -> Rect {
        Rect::new(
            coord!( x: self.header.grid.longitude.start, y: self.header.grid.latitude.start ),
            coord!( x: self.header.grid.longitude.end, y: self.header.grid.latitude.end),
        )
    }
    /// Converts this Global/Worldwide [IONEX] to Regional [IONEX]
    /// restraning map to the provided borders, expressed in degrees.
    /// Borders expressed as [Rect]angle (min, max),
    /// where x is the longitude and y the latitude angle, both
    /// expressed in decimal degrees.
    pub fn to_region_degrees(&self, region: Rect) -> IONEX {
        let mut ionex = IONEX::default();

        let (min_long, min_lat) = (region.min().x, region.min().y);
        let (max_long, max_lat) = (region.max().x, region.max().y);

        // copy attributes
        ionex.production = self.production.clone();

        if let Some(production) = &mut ionex.production {
            production.region = Region::Regional;
        }

        // copy & rework header
        ionex.header = self.header.clone();

        if min_lat > ionex.header.grid.latitude.start {
            ionex.header.grid.latitude.start = min_lat;
        }

        if max_lat < ionex.header.grid.latitude.end {
            ionex.header.grid.latitude.end = max_lat;
        }

        if min_long > ionex.header.grid.longitude.start {
            ionex.header.grid.longitude.start = min_long;
        }

        if max_long < ionex.header.grid.longitude.end {
            ionex.header.grid.longitude.end = max_long;
        }

        // restrain map
        ionex.record.map = self
            .record
            .map
            .iter()
            .filter_map(|(k, v)| {
                let latitude_ddeg = k.latitude_ddeg();
                let longitude_ddeg = k.longitude_ddeg();

                if latitude_ddeg >= min_lat && latitude_ddeg <= max_lat {
                    if longitude_ddeg >= min_long && longitude_ddeg <= max_long {
                        Some((*k, *v))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .collect();

        ionex
    }

    /// Converts this Global/Worldwide [IONEX] to Regional [IONEX]
    /// restraining map to the provided borders, expressed in radians.
    /// Borders expressed as [Rect]angle (min, max),
    /// where x is the longitude and y the latitude angle, both
    /// expressed in radians.
    pub fn to_region_radians(&self, region: Rect) -> IONEX {
        self.to_region_degrees(Rect::new(
            coord!(x: region.min().x.to_radians(), y: region.min().y.to_radians()),
            coord!(x: region.max().x.to_radians(), y: region.max().y.to_radians()),
        ))
    }

    /// Stretch this map returning a new [IONEX], increasing grid granularity
    /// by applying the 2D planar interpolation equation.
    pub fn planar_stretch(&self, stretch_factor: f64) -> IONEX {
        let mut s = self.clone();
        s.planar_stretch_mut(stretch_factor);
        s
    }

    /// Stretch this mutable map, increasing grid granularity
    /// by applying the 2D planar interpolation equation.
    pub fn planar_stretch_mut(&mut self, stretch_factor: f64) {
        // update grid
        self.header.grid.latitude.start *= stretch_factor;
        self.header.grid.latitude.end *= stretch_factor;
        self.header.grid.longitude.start *= stretch_factor;
        self.header.grid.longitude.end *= stretch_factor;

        // update map
        for epoch in self.record.epochs_iter() {}
    }
}

#[cfg(test)]
mod test {
    use crate::fmt_comment;
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
            let nb_lines = num_integer::div_ceil(desc.len(), 60);
            let comments = fmt_comment(desc);

            assert_eq!(comments.lines().count(), nb_lines);

            for line in comments.lines() {
                assert!(line.len() >= 60, "comment line should be at least 60 byte long");
                assert_eq!(line.find("COMMENT"), Some(60), "comment marker should located @ 60");
            }
        }
    }
}
