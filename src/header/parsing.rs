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
                comments.push(content.trim().to_string());
                continue;
            } else if marker.contains("IONEX VERSION / TYPE") {
                let (vers, rem) = line.split_at(20);
                let (type_str, rem) = rem.split_at(20);
                s.version = Version::from_str(type_str)?;
            } else if marker.contains("PGM / RUN BY / DATE") {
                let (pgm, rem) = line.split_at(20);

                let pgm = pgm.trim();

                if pgm.len() > 0 {
                    s.program = Some(pgm.to_string());
                }

                let (runby, rem) = rem.split_at(20);

                let runby = runby.trim();

                if runby.len() > 0 {
                    s.run_by = Some(runby.to_string());
                }

                let date_str = rem.split_at(20).0.trim();

                if date_str.len() > 0 {
                    s.date = Some(date_str.to_string());
                }
            } else if marker.contains("MERGED FILE") {
            } else if marker.contains("LICENSE OF USE") {
                let lic = content.split_at(40).0.trim(); //TODO confirm please
                if lic.len() > 0 {
                    license = Some(lic.to_string());
                }
            } else if marker.contains("INTERVAL") {
                let intv_str = content.split_at(20).0.trim();
                if let Ok(interval) = f64::from_str(intv_str) {
                    s.sampling_interval = Duration::from_seconds(interval);
                }
            }
        }

        Ok(header)
    }

    fn parse_time_of_obs(content: &str) -> Result<Epoch, ParsingError> {
        let (_, rem) = content.split_at(2);
        let (y, rem) = rem.split_at(4);
        let (m, rem) = rem.split_at(6);
        let (d, rem) = rem.split_at(6);
        let (hh, rem) = rem.split_at(6);
        let (mm, rem) = rem.split_at(6);
        let (ss, rem) = rem.split_at(5);
        let (_dot, rem) = rem.split_at(1);
        let (ns, rem) = rem.split_at(8);

        // println!("Y \"{}\" M \"{}\" D \"{}\" HH \"{}\" MM \"{}\" SS \"{}\" NS \"{}\"", y, m, d, hh, mm, ss, ns); // DEBUG
        let mut y = y
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        // handle OLD RINEX problem
        if y >= 79 && y <= 99 {
            y += 1900;
        } else if y < 79 {
            y += 2000;
        }

        let m = m
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let d = d
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let hh = hh
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let mm = mm
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let ss = ss
            .trim()
            .parse::<u8>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        let ns = ns
            .trim()
            .parse::<u32>()
            .map_err(|_| ParsingError::DatetimeParsing)?;

        /*
         * We set TAI as "default" Timescale.
         * Timescale might be omitted in Old RINEX formats,
         * In this case, we exit with "TAI" and handle that externally.
         */
        let mut ts = TimeScale::TAI;
        let rem = rem.trim();

        /*
         * Handles DORIS measurement special case,
         * offset from TAI, that we will convert back to TAI later
         */
        if !rem.is_empty() && rem != "DOR" {
            ts = TimeScale::from_str(rem.trim())?;
        }

        Epoch::from_str(&format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:08} {}",
            y, m, d, hh, mm, ss, ns, ts
        ))
        .map_err(|_| ParsingError::DatetimeParsing)
    }

    /// Parse list of [Observable]s which applies to both METEO and OBS RINEX
    pub(crate) fn parse_v2_observables(
        line: &str,
        constell: Option<Constellation>,
        meteo: &mut MeteoHeader,
        observation: &mut ObservationHeader,
    ) {
        lazy_static! {
            /*
             *  We support GPS, Glonass, Galileo, SBAS and BDS as per v2.11.
             */
            static ref KNOWN_V2_CONSTELLS: [Constellation; 5] = [
                Constellation::GPS,
                Constellation::SBAS,
                Constellation::Glonass,
                Constellation::Galileo,
                Constellation::BeiDou,
            ];
        }
        let line = line.split_at(6).1;
        for item in line.split_ascii_whitespace() {
            if let Ok(obs) = Observable::from_str(item.trim()) {
                match constell {
                    Some(Constellation::Mixed) => {
                        for constell in KNOWN_V2_CONSTELLS.iter() {
                            if let Some(codes) = observation.codes.get_mut(constell) {
                                codes.push(obs.clone());
                            } else {
                                observation.codes.insert(*constell, vec![obs.clone()]);
                            }
                        }
                    },
                    Some(c) => {
                        if let Some(codes) = observation.codes.get_mut(&c) {
                            codes.push(obs.clone());
                        } else {
                            observation.codes.insert(c, vec![obs.clone()]);
                        }
                    },
                    None => meteo.codes.push(obs),
                }
            }
        }
    }

    /// Parse list of [Observable]s which applies to both METEO and OBS RINEX
    fn parse_v3_observables(
        line: &str,
        current_constell: &mut Option<Constellation>,
        observation: &mut ObservationHeader,
    ) {
        let (possible_counter, items) = line.split_at(6);
        if !possible_counter.is_empty() {
            let code = &possible_counter[..1];
            if let Ok(c) = Constellation::from_str(code) {
                *current_constell = Some(c);
            }
        }
        if let Some(constell) = current_constell {
            // system correctly identified
            for item in items.split_ascii_whitespace() {
                if let Ok(observable) = Observable::from_str(item) {
                    if let Some(codes) = observation.codes.get_mut(constell) {
                        codes.push(observable);
                    } else {
                        observation.codes.insert(*constell, vec![observable]);
                    }
                }
            }
        }
    }
    /*
     * Parse list of DORIS observables
     */
    fn parse_doris_observables(line: &str, doris: &mut DorisHeader) {
        let items = line.split_at(6).1;
        for item in items.split_ascii_whitespace() {
            if let Ok(observable) = Observable::from_str(item) {
                doris.observables.push(observable);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::{Epoch, Header};
    use std::str::FromStr;

    #[test]
    fn parse_time_of_obs() {
        let content = "  2021    12    21     0     0    0.0000000     GPS";
        let parsed = Header::parse_time_of_obs(&content).unwrap();
        assert_eq!(parsed, Epoch::from_str("2021-12-21T00:00:00 GPST").unwrap());

        let content = "  1995    01    01    00    00   00.000000             ";
        let parsed = Header::parse_time_of_obs(&content).unwrap();
        assert_eq!(parsed, Epoch::from_str("1995-01-01T00:00:00 TAI").unwrap());
    }
}
