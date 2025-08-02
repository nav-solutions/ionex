use crate::{
    error::ParsingError,
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
            if marker.trim().eq("END OF HEADER") {
                // special marker: exit
                break;
            }

            if marker.trim().eq("COMMENT") {
                // Comments are stored as is
                header.comments.push(content.trim().to_string());
            } else if marker.contains("IONEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20);
                header.version = Version::from_str(type_str)?;
            } else if marker.contains("PGM / RUN BY / DATE") {
                let (pgm, rem) = line.split_at(20);

                let pgm = pgm.trim();

                if pgm.len() > 0 {
                    header.program = Some(pgm.to_string());
                }

                let (runby, rem) = rem.split_at(20);

                let runby = runby.trim();

                if runby.len() > 0 {
                    header.run_by = Some(runby.to_string());
                }

                let date_str = rem.split_at(20).0.trim();

                if date_str.len() > 0 {
                    header.date = Some(date_str.to_string());
                }
            } else if marker.contains("MERGED FILE") {
            } else if marker.contains("LICENSE OF USE") {
                let lic = content.split_at(40).0.trim(); //TODO confirm please
                if lic.len() > 0 {
                    header.license = Some(lic.to_string());
                }
            } else if marker.contains("INTERVAL") {
                let intv_str = content.split_at(20).0.trim();
                if let Ok(interval) = f64::from_str(intv_str) {
                    header.sampling_period = Duration::from_seconds(interval);
                }
            }
        }

        Ok(header)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Epoch, Header};
    use std::str::FromStr;
}
