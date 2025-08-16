//! Epoch parsing helper
use crate::prelude::{Epoch, ParsingError};

// use std::str::FromStr;

/// Formats given epoch to string, matching standard specifications
pub(crate) fn format_header(epoch: Epoch) -> String {
    let (y, m, d, hh, mm, ss, _nanos) = epoch.to_gregorian(epoch.time_scale);
    format!(
        "{:04}   {:>2}    {:>2}    {:>2}    {:>2}    {:>2}",
        y, m, d, hh, mm, ss
    )
}

pub(crate) fn format_body(epoch: Epoch) -> String {
    let (y, m, d, hh, mm, ss, _nanos) = epoch.to_gregorian(epoch.time_scale);
    format!(
        "  {:04}    {:>2}    {:>2}    {:>2}    {:>2}    {:>2}",
        y, m, d, hh, mm, ss
    )
}

pub(crate) fn parse_utc(s: &str) -> Result<Epoch, ParsingError> {
    let (mut y, mut m, mut d, mut hh, mut mm, mut ss) = (0_i32, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8);
    for (index, field) in s.split_ascii_whitespace().enumerate() {
        match index {
            0 => {
                y = field
                    .trim()
                    .parse::<i32>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            1 => {
                m = field
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            2 => {
                d = field
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            3 => {
                hh = field
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            4 => {
                mm = field
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            5 => {
                ss = field
                    .trim()
                    .parse::<u8>()
                    .map_err(|_| ParsingError::DatetimeParsing)?;
            },
            _ => {},
        }
    }

    Ok(Epoch::from_gregorian_utc(y, m, d, hh, mm, ss, 0))
}

// /*
//  * Until Hifitime provides a decomposition method in timescale other than UTC
//  * we have this tweak to decompose %Y %M %D %HH %MM %SS and without nanoseconds
//  */
// pub(crate) fn epoch_decompose(e: Epoch) -> (i32, u8, u8, u8, u8, u8, u32) {
//     let isofmt = e.to_gregorian_str(e.time_scale);
//     let mut datetime = isofmt.split('T');
//     let date = datetime.next().unwrap();
//     let mut date = date.split('-');
//
//     let time = datetime.next().unwrap();
//     let mut time_scale = time.split(' ');
//     let time = time_scale.next().unwrap();
//     let mut time = time.split(':');
//
//     let years = date.next().unwrap().parse::<i32>().unwrap();
//     let months = date.next().unwrap().parse::<u8>().unwrap();
//     let days = date.next().unwrap().parse::<u8>().unwrap();
//
//     let hours = time.next().unwrap().parse::<u8>().unwrap();
//     let mins = time.next().unwrap().parse::<u8>().unwrap();
//     let seconds = f64::from_str(time.next().unwrap()).unwrap();
//
//     (
//         years,
//         months,
//         days,
//         hours,
//         mins,
//         seconds.floor() as u8,
//         (seconds.fract() * 1E9).round() as u32,
//     )
// }

#[cfg(test)]
mod test {
    use super::*;
    use hifitime::Epoch;
    use hifitime::TimeScale;
    use std::str::FromStr;

    #[test]
    fn datetime_parsing() {
        for (desc, expected) in [(
            "  2022     1     2     0     0     0                        ",
            Epoch::from_str("2022-01-02T00:00:00 UTC").unwrap(),
        )] {
            let epoch = parse_utc(desc).unwrap_or_else(|e| {
                panic!("Failed to parse datetime from \"{}\": {}", desc, e);
            });

            assert_eq!(epoch, expected);
        }
    }

    // #[test]
    // fn epoch_decomposition() {
    //     for (epoch, y, m, d, hh, mm, ss, ns) in [
    //         ("2021-01-01T00:00:00 GPST", 2021, 1, 1, 0, 0, 0, 0),
    //         ("2021-01-01T00:00:01 GPST", 2021, 1, 1, 0, 0, 1, 0),
    //         ("2021-01-01T23:59:58 GPST", 2021, 1, 1, 23, 59, 58, 0),
    //         ("2021-01-01T23:59:59 GPST", 2021, 1, 1, 23, 59, 59, 0),
    //         ("2021-01-01T00:00:00 GST", 2021, 1, 1, 0, 0, 0, 0),
    //         ("2021-01-01T00:00:01 GST", 2021, 1, 1, 0, 0, 1, 0),
    //         ("2021-01-01T23:59:58 GST", 2021, 1, 1, 23, 59, 58, 0),
    //         ("2021-01-01T23:59:59 GST", 2021, 1, 1, 23, 59, 59, 0),
    //     ] {
    //         let e = Epoch::from_str(epoch).unwrap();
    //         assert_eq!(
    //             epoch_decompose(e),
    //             (y, m, d, hh, mm, ss, ns),
    //             "failed for {}",
    //             epoch
    //         );
    //     }
    // }
}
