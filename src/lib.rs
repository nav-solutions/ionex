#![doc(
    html_logo_url = "https://raw.githubusercontent.com/nav-solutions/.github/master/logos/logo2.jpg"
)]
#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![allow(clippy::type_complexity)]

/*
 * IONEX is part of the nav-solutions framework.
 * Authors: Guillaume W. Bres <guillaume.bressaix@gmail.com> et al.
 * (cf. https://github.com/nav-solutions/ionex/graphs/contributors)
 * This framework is shipped under Mozilla Public V2 license.
 *
 * Documentation: https://github.com/nav-solutions/ionex
 */

extern crate num_derive;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde;

extern crate gnss_rs as gnss;
extern crate num;

pub mod error;
pub mod header;
pub mod production;
pub mod record;
pub mod version;

mod epoch;
mod linspace;

#[cfg(test)]
mod tests;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};

use itertools::Itertools;

#[cfg(feature = "flate2")]
use flate2::{read::GzDecoder, write::GzEncoder, Compression as GzCompression};

use std::collections::BTreeMap;

use crate::{
    epoch::epoch_decompose,
    production::{DataSource, DetailedProductionAttributes, ProductionAttributes, FFU, PPU},
};

pub mod prelude {
    // export
    pub use crate::{
        error::{Error, FormattingError, ParsingError},
        header::Header,
        production::*,
        version::Version,
        IONEX,
    };

    pub use crate::record::{Comments, IonexKey, QuantizedCoordinates, TEC};

    // pub re-export
    pub use gnss::prelude::{Constellation, SV};
    pub use hifitime::{Duration, Epoch, TimeScale, TimeSeries};
}

/// returns true if given line is a comment
pub(crate) fn is_comment(content: &str) -> bool {
    content.len() > 60 && content.trim_end().ends_with("COMMENT")
}

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
    fmt_rinex(content, "COMMENT")
}

#[derive(Clone, Debug)]
/// [IONEX] is composed of a [Header] section and a [Record] section.
/// ```
/// use ionex::prelude::*;
///
/// let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz")
///     .unwrap();
///
/// // header contains high level information
/// // like file standard revision:
/// assert_eq!(ionex.header.version.major, 2);
/// assert_eq!(ionex.header.version.minor, 11);
/// ```
pub struct IONEX {
    /// [Header] gives general information and describes following content.
    pub header: Header,

    /// [Comments] stored as they appeared in file body
    pub comments: Comments,

    /// [Record] is the actual file content and is heavily [RinexType] dependent
    pub record: Record,

    /// [ProductionAttributes] resolved for file names that follow
    /// according to the standards.
    pub production: Option<ProductionAttributes>,
}

impl IONEX {
    /// Builds a new [IONEX] struct from given header & body sections.
    pub fn new(header: Header, record: record::Record) -> Rinex {
        Rinex {
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

    /// Returns a file name that would describe [Self] according to the
    /// standards.
    pub fn standardized_filename(&self) -> String {
        let header = &self.header;

        let (agency, region, year, doy) = if let Some(producution) = self.production {
            (
                production.agency,
                production.region,
                production.year - 2000,
                production.doy,
            )
        } else {
            ("XXX", 'G', 0, 0)
        };

        let extension = if let Some(production) = &self.production {
            if production.gzip_compressed {
                ".gz"
            } else {
                ""
            }
        } else {
            ""
        };

        format!("{}{}{:03}.{:02}I", agency, doy, year,)
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

        let first_epoch = self.first_epoch()?;

        match first_epoch.to_gregorian_utc() {
            Some((y, _, _, _, _, _, _)) => attributes.year = y as u32,
            _ => {}
        }

        let doy = first_epoch.day_of_year().round() as u32;

        let region = Region::Global; // TODO: study the grid specs

        Some(ProductionAttributes {
            aagency: agency.to_string(),
            doy,
            year,
            region,
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
            comments,
            record,
            production: Default::default(),
        })
    }

    /// Format [RINEX] into writable I/O using efficient buffered writer
    /// and following standard specifications. The revision to be followed is defined
    /// in [Header] section. This is the mirror operation of [Self::parse].
    pub fn format<W: Write>(&self, writer: &mut BufWriter<W>) -> Result<(), FormattingError> {
        self.header.format(writer)?;
        self.record.format(writer, &self.header)?;
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
            }
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
            }
            _ => None,
        };

        let fd = File::open(path)?;

        let reader = GzDecoder::new(fd);
        let mut reader = BufReader::new(reader);
        let mut ionex = Self::parse(&mut reader)?;
        ionex.production = file_attributes;
        Ok(rinex)
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
}

impl IONEX {
    /// Returns [Epoch] Iterator.
    pub fn epoch_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.record.iter().map(|(k, _)| k.epoch))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{fmt_comment, is_rinex_comment};
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
            assert!(is_rinex_comment(&comment), "should be valid comment");
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
                assert!(is_rinex_comment(line), "should be valid comment");
            }
        }
    }
}
