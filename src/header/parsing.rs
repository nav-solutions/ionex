use crate::{
    epoch::parse_utc as parse_utc_epoch,
    error::ParsingError,
    linspace::Linspace,
    prelude::{BiasSource, Constellation, Duration, Epoch, Header, ReferenceSystem, Version},
};

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

impl Header {
    /// Parse [Header] by consuming [BufReader] until end of this section
    pub fn parse<R: Read>(reader: &mut BufReader<R>) -> Result<Self, ParsingError> {
        let mut header = Self::default();

        for line in reader.lines() {
            if line.is_err() {
                continue;
            }

            let line = line.unwrap();

            if line.len() < 60 {
                continue; // invalid content
            }

            let (content, marker) = line.split_at(60);

            if marker.contains("END OF HEADER") {
                // special marker: exit
                break;
            } else if marker.contains("COMMENT") {
                // Comments are stored as is
                header.comments.push(content.trim().to_string());
            } else if marker.contains("IONEX VERSION / TYPE") {
                let (vers_str, _) = line.split_at(20);
                header.version = Version::from_str(vers_str.trim())?;
            } else if marker.contains("# OF MAPS IN FILE") {
                let number = line.split_at(20).0.trim();

                header.number_of_maps = number
                    .parse::<u32>()
                    .map_err(|_| ParsingError::NumberofMaps)?;
            } else if marker.contains("EPOCH OF FIRST MAP") {
                let epoch_str = line.split_at(60).0;
                let epoch = parse_utc_epoch(epoch_str)?;
                header.epoch_of_first_map = epoch;
            } else if marker.contains("EPOCH OF LAST MAP") {
                let epoch_str = line.split_at(60).0;
                let epoch = parse_utc_epoch(epoch_str)?;
                header.epoch_of_last_map = epoch;
            } else if marker.contains("PGM / RUN BY / DATE") {
                let (pgm, rem) = line.split_at(20);

                let pgm = pgm.trim();

                if !pgm.is_empty() {
                    header.program = Some(pgm.to_string());
                }

                let (runby, rem) = rem.split_at(20);

                let runby = runby.trim();

                if !runby.is_empty() {
                    header.run_by = Some(runby.to_string());
                }

                let date_str = rem.split_at(20).0.trim();

                if !date_str.is_empty() {
                    header.date = Some(date_str.to_string());
                }
            } else if marker.contains("LICENSE OF USE") {
                let license = content.split_at(40).0.trim(); //TODO confirm please

                if !license.is_empty() {
                    header.license = Some(license.to_string());
                }
            } else if marker.contains("INTERVAL") {
                let interval = content.split_at(20).0.trim();

                let interval = interval
                    .parse::<f64>()
                    .map_err(|_| ParsingError::SamplingPeriod)?;

                header.sampling_period = Duration::from_seconds(interval);
            } else if marker.contains("LAT1 / LAT2 / DLAT") {
                // latitude grid specs
                let (start_str, rem) = content.split_at(8);
                let (end_str, rem) = rem.split_at(6);
                let (spacing_str, rem) = rem.split_at(6);

                let start = start_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let end = end_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let spacing = spacing_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let linspace = Linspace::new(start, end, spacing)?;

                header.grid.latitude = linspace;
            } else if marker.contains("LON1 / LON2 / DLON") {
                // longitude grid specs
                let (start_str, rem) = content.split_at(8);
                let (end_str, rem) = rem.split_at(6);
                let (spacing_str, rem) = rem.split_at(6);

                let start = start_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let end = end_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let spacing = spacing_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let linspace = Linspace::new(start, end, spacing)?;

                header.grid.longitude = linspace;
            } else if marker.contains("HGT1 / HGT2 / DHGT") {
                // altitude grid specs
                let (start_str, rem) = content.split_at(8);
                let (end_str, rem) = rem.split_at(6);
                let (spacing_str, rem) = rem.split_at(6);

                let start = start_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let end = end_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let spacing = spacing_str
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| ParsingError::InvalidGridDefinition)?;

                let linspace = Linspace::new(start, end, spacing)?;

                header.grid.altitude = linspace;
            }
        }

        Ok(header)
    }
}
