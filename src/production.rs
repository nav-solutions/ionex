/*
 * File Production infrastructure.
 */
use thiserror::Error;

#[derive(Error, Debug)]
/// File Production identification errors
pub enum Error {
    #[error("filename does not follow naming conventions")]
    NonStandardFileName,
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
        if fname.len() < 13 {
            let offset = fname.find('.').unwrap_or(0);
            if offset != 8 {
                return Err(Error::NonStandardFileName);
            };

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
                v3_details: None,
            })
        } else {
            let offset = fname.find('.').unwrap_or(0);
            if offset < 30 {
                return Err(Error::NonStandardFileName);
            };

            let year = fname[12..16]
                .parse::<u32>()
                .map_err(|_| Error::NonStandardFileName)?;

            let batch = fname[5..6]
                .parse::<u8>()
                .map_err(|_| Error::NonStandardFileName)?;

            // determine type of RINEX first
            // because it determines how to parse the "name" field
            let rtype = &fname[offset + 3..offset + 4];
            let name_offset = match rtype {
                "I" => 3usize, // only 3 digits on IONEX
                _ => 4usize,
            };

            Ok(Self {
                year,
                name: fname[..name_offset].to_string(),
                doy: {
                    fname[16..19]
                        .parse::<u32>()
                        .map_err(|_| Error::NonStandardFileName)?
                },
                region: None, // IONEX files only use a short format
                v3_details: Some(DetailedProductionAttributes {
                    batch,
                    country: fname[6..9].to_string(),
                    ppu: PPU::from_str(&fname[24..27])?,
                    data_src: DataSource::from_str(&fname[10..11])?,
                    hh: {
                        fname[19..21]
                            .parse::<u8>()
                            .map_err(|_| Error::NonStandardFileName)?
                    },
                    mm: {
                        fname[21..23]
                            .parse::<u8>()
                            .map_err(|_| Error::NonStandardFileName)?
                    },
                    ffu: match offset {
                        34 => Some(FFU::from_str(&fname[28..32])?),
                        _ => None, // NAV FILE case
                    },
                }),
            })
        }
    }
}

#[cfg(test)]
mod test {
    use super::DetailedProductionAttributes;
    use super::ProductionAttributes;
    use super::{DataSource, FFU, PPU};

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
