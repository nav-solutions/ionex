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
    fn from_str(fname: &str) -> Result<Self, Self::Err> {
        let fname = fname.to_uppercase();
        if fname.len() != 13 {
            return Err(Error::NonStandardFilename);
        }

        let offset = fname.find('.').unwrap_or(0);

        // determine type of RINEX first
        // because it determines how to parse the "name" field
        let year = fname[offset + 1..offset + 3]
            .parse::<u32>()
            .map_err(|_| Error::NonStandardFileName)?;

        let rtype = &fname[offset + 3..offset + 4];

        let name_offset = match rtype {
            "I" => 3usize, // only 3 digits on IONEX
            _ => 4usize,
        };

        Ok(Self {
            year: year + 2_000, // year uses 2 digit in old format
            name: fname[..name_offset].to_string(),
            doy: {
                fname[4..7]
                    .parse::<u32>()
                    .map_err(|_| Error::NonStandardFileName)?
            },
            region: match rtype {
                "I" => fname.chars().nth(3),
                _ => None,
            },
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
