//! File production infrastructure.
use thiserror::Error;

/// File Production identification errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("filename does not follow naming conventions")]
    NonStandardFilename,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub enum Region {
    /// Local IONEX (Regional maps)
    Regional,

    /// Global IONEX (Worldwide maps)
    #[default]
    Global,
}

/// File production attributes. Used when generating
/// RINEX data that follows standard naming conventions,
/// or attached to data parsed from such files.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct ProductionAttributes {
    /// Production agency
    pub agency: String,

    /// Year of production
    pub year: u32,

    /// Production Day of Year (DOY).
    /// We assume past J2000.
    pub doy: u32,

    /// Regional code present in IONEX file names.
    pub region: Region,

    /// True if this file was gzip compressed
    #[cfg(feature = "flate2")]
    #[cfg_attr(docsrs, doc(cfg(feature = "flate2")))]
    pub gzip_compressed: bool,
}

impl ProductionAttributes {
    /// Format [Self] as standardized file name
    pub(crate) fn format(name: &str, region: char, ddd: &str, yy: &str) -> String {
        format!("{}{}{}0.{}I", name, region, ddd, yy,)
    }
}

impl std::str::FromStr for ProductionAttributes {
    type Err = Error;

    fn from_str(filename: &str) -> Result<Self, Self::Err> {
        let filename = filename.to_uppercase();

        let name_len = filename.len();

        if name_len != 12 && name_len != 15 {
            return Err(Error::NonStandardFilename);
        }

        let offset = fname.find('.').unwrap_or(0);

        let agency = filename[..4].to_string();

        let year = filename[offset + 1..offset + 3]
            .parse::<u32>()
            .map_err(|_| Error::NonStandardFileName)?;

        let name_offset = match rtype {
            "I" => 3usize, // only 3 digits on IONEX
            _ => 4usize,
        };

        let region = if filename[4..5].eq("G") {
            Region::Global
        } else {
            Region::Regional
        };

        Ok(Self {
            region,
            agency,
            year: year + 2_000, // year uses 2 digit in old format
            doy: {
                filename[4..7]
                    .parse::<u32>()
                    .map_err(|_| Error::NonStandardFileName)?
            },
            #[cfg(feature = "flate2")]
            gzip_compressed: filename.ends_with(".gz"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::DetailedProductionAttributes;
    use super::ProductionAttributes;

    use hifitime::Unit;
    use std::str::FromStr;

    #[test]
    fn filenames() {
        for (filename, name, year, doy, region) in [
            ("CKMG0020.22I", "CKM", 2022, 2, 'G'),
            ("CKMG0090.21I", "CKM", 2021, 9, 'G'),
            ("jplg0010.17i", "JPL", 2017, 1, 'G'),
        ] {
            println!("Testing IONEX filename \"{}\"", filename);
            let attrs = ProductionAttributes::from_str(filename).unwrap();
            assert_eq!(attrs.name, name);
            assert_eq!(attrs.year, year);
            assert_eq!(attrs.doy, doy);
            assert_eq!(attrs.region, Some(region));
        }
    }
}
