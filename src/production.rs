//! File production infrastructure.
use thiserror::Error;

/// File Production identification errors
#[derive(Error, Debug)]
pub enum Error {
    #[error("filename does not follow naming conventions")]
    NonStandardFilename,
}

#[derive(Debug, Copy, Default, Clone, PartialEq)]
pub enum Region {
    /// Local IONEX (Regional maps)
    Regional,

    /// Global IONEX (Worldwide maps)
    #[default]
    Global,
}

impl std::fmt::Display for Region {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Regional => write!(f, "{}", 'R'),
            Self::Global => write!(f, "{}", 'G'),
        }
    }
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

impl std::fmt::Display for ProductionAttributes {
    #[cfg(feature = "flate2")]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let len = std::cmp::min(self.agency.len(), 3);

        write!(
            f,
            "{}{}{:03}0.{:02}I",
            &self.agency[..len],
            self.region,
            self.doy,
            self.year - 2000
        )?;

        if self.gzip_compressed {
            write!(f, ".gz")?;
        }

        Ok(())
    }

    #[cfg(not(feature = "flate2"))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let len = std::cmp::min(self.agency.len(), 3);
        write!(
            f,
            "{}{}{:03}0.{:02}I",
            &self.agency[..len],
            self.region,
            self.doy,
            self.year - 2000
        )
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

        let offset = filename.find('.').unwrap_or(0);

        let agency = filename[..3].to_string();

        let year = filename[offset + 1..offset + 3]
            .parse::<u32>()
            .map_err(|_| Error::NonStandardFilename)?;

        let region = if filename[3..4].eq("G") {
            Region::Global
        } else {
            Region::Regional
        };

        Ok(Self {
            region,
            agency,
            doy: {
                filename[4..7]
                    .parse::<u32>()
                    .map_err(|_| Error::NonStandardFilename)?
            },
            year: year + 2_000,
            #[cfg(feature = "flate2")]
            gzip_compressed: filename.ends_with(".GZ"),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn standard_filenames() {
        for (filename, agency, year, doy, region) in [
            ("CKMG0020.22I", "CKM", 2022, 2, Region::Global),
            ("CKMG0090.21I", "CKM", 2021, 9, Region::Global),
            ("JPLG0010.17I", "JPL", 2017, 1, Region::Global),
            ("JPLR0010.17I", "JPL", 2017, 1, Region::Regional),
            ("JPLR0010.17I", "JPL", 2017, 1, Region::Regional),
        ] {
            println!("Testing IONEX filename \"{}\"", filename);

            let attrs = ProductionAttributes::from_str(filename).unwrap();

            assert_eq!(attrs.agency, agency);
            assert_eq!(attrs.year, year);
            assert_eq!(attrs.doy, doy);
            assert_eq!(attrs.region, region);
            assert!(!attrs.gzip_compressed);

            let formatted = attrs.to_string();
            assert_eq!(formatted, filename);
        }
    }

    #[test]
    fn gzip_filenames() {
        for (filename, agency, year, doy, region) in [
            ("CKMG0020.22I.gz", "CKM", 2022, 2, Region::Global),
            ("CKMR0020.22I.gz", "CKM", 2022, 2, Region::Regional),
            ("JPLG0010.17I.gz", "JPL", 2017, 1, Region::Global),
            ("CKMR0020.22I.gz", "CKM", 2022, 2, Region::Regional),
        ] {
            let attrs = ProductionAttributes::from_str(filename).unwrap();

            assert_eq!(attrs.agency, agency);
            assert_eq!(attrs.year, year);
            assert_eq!(attrs.doy, doy);
            assert_eq!(attrs.region, region);
            assert!(attrs.gzip_compressed);

            let formatted = attrs.to_string();
            assert_eq!(formatted, filename);
        }
    }
}
