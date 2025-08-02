use std::collections::{BTreeMap, HashMap};
use std::str::FromStr;
use thiserror::Error;

use crate::prelude::{
    Quantized,
    Key,
    Epoch,
    TEC,
    Header,
};

pub(crate) fn is_new_tec_plane(line: &str) -> bool {
    line.contains("START OF TEC MAP")
}

pub(crate) fn is_new_rms_plane(line: &str) -> bool {
    line.contains("START OF RMS MAP")
}

// pub(crate) fn is_new_height_map(line: &str) -> bool {
//     line.contains("START OF HEIGHT MAP")
// }

/// [Record] describes IONEX data.
pub type Record = BTreeMap<Key, TEC>;

/*
 * Parses following map, which can either be
 *  - a TEC map
 *  - an RMS tec map
 *  - an height map
 * Returns: Epoth(t), nth Map index, latitude, altitude and TEC plane accross longitudes
 */
pub(crate) fn parse_plane(
    content: &str,
    header: &mut Header,
    is_rms_plane: bool,
) -> Result<(Epoch, i32, TECPlane), Error> {
    let lines = content.lines();
    let mut epoch = Epoch::default();
    let mut plane = TECPlane::with_capacity(128);

    // this can't fail at this point
    let ionex = header
        .ionex
        .as_mut()
        .expect("faulty ionex context: missing specific header definitions");

    // current {lat, lon} within current grid def.
    let mut latitude = 0_i32;
    let mut longitude = 0_i32;
    let mut altitude = 0_i32;
    let mut dlon = (ionex.grid.longitude.spacing * 1000.0) as i32;

    for line in lines {
        if line.len() > 60 {
            let (content, marker) = line.split_at(60);
            if marker.contains("START OF") {
                continue; // skip that one
            } else if marker.contains("END OF") && marker.contains("MAP") {
                let index = content.split_at(6).0;
                let index = index.trim();
                let _map_index = index
                    .parse::<u32>()
                    .or(Err(Error::MapIndexParsing(index.to_string())))?;

                return Ok((epoch, altitude, plane));
            } else if marker.contains("LAT/LON1/LON2/DLON/H") {
                // grid definition for next block
                let (_, rem) = content.split_at(2);

                let (lat, rem) = rem.split_at(6);
                let lat = lat.trim();
                let lat = f64::from_str(lat).or(Err(Error::CoordinatesParsing(
                    String::from("latitude"),
                    lat.to_string(),
                )))?;

                let (lon1, rem) = rem.split_at(6);
                let lon1 = lon1.trim();
                let lon1 = f64::from_str(lon1).or(Err(Error::CoordinatesParsing(
                    String::from("longitude"),
                    lon1.to_string(),
                )))?;

                let (_lon2, rem) = rem.split_at(6);
                //let lon2 = lon2.trim();
                //let lon2 = f64::from_str(lon2).or(Err(Error::CoordinatesParsing(
                //    String::from("longitude"),
                //    lon2.to_string(),
                //)))?;

                let (dlon_str, rem) = rem.split_at(6);
                let dlon_str = dlon_str.trim();
                let dlon_f64 = f64::from_str(dlon_str).or(Err(Error::CoordinatesParsing(
                    String::from("longitude"),
                    dlon_str.to_string(),
                )))?;

                let (h, _) = rem.split_at(6);
                let h = h.trim();
                let alt = f64::from_str(h).or(Err(Error::CoordinatesParsing(
                    String::from("altitude"),
                    h.to_string(),
                )))?;

                altitude = (alt.round() * 100.0_f64) as i32;
                latitude = (lat.round() * 1000.0_f64) as i32;
                longitude = (lon1.round() * 1000.0_f64) as i32;
                dlon = (dlon_f64.round() * 1000.0_f64) as i32;

                // debug
                // println!("NEW GRID : h: {} lat : {} lon : {}, dlon: {}", altitude, latitude, longitude, dlon);
            } else if marker.contains("EPOCH OF CURRENT MAP") {
                epoch = epoch::parse_utc(content)?;
            } else if marker.contains("EXPONENT") {
                // update current scaling
                if let Ok(e) = content.trim().parse::<i8>() {
                    ionex.exponent = e;
                }
            } else {
                // parsing TEC values
                for item in line.split_ascii_whitespace() {
                    if let Ok(v) = item.trim().parse::<i32>() {
                        let mut value = v as f64;
                        // current scaling
                        value *= 10.0_f64.powf(ionex.exponent as f64);

                        let tec = match is_rms_plane {
                            true => {
                                TEC {
                                    tec: 0.0_f64, // DONT CARE
                                    rms: Some(value),
                                }
                            }
                            false => TEC {
                                tec: value,
                                rms: None,
                            },
                        };

                        plane.insert((latitude, longitude), tec);
                    }

                    longitude += dlon;
                    //debug
                    //println!("longitude: {}", longitude);
                }
            }
        } else {
            // less than 60 characters
            // parsing TEC values
            for item in line.split_ascii_whitespace() {
                if let Ok(v) = item.trim().parse::<i32>() {
                    let mut value = v as f64;
                    // current scaling
                    value *= 10.0_f64.powf(ionex.exponent as f64);

                    let tec = match is_rms_plane {
                        true => {
                            TEC {
                                tec: 0.0_f64, // DONT CARE
                                rms: Some(value),
                            }
                        }
                        false => TEC {
                            tec: value,
                            rms: None,
                        },
                    };

                    plane.insert((latitude, longitude), tec);
                }

                longitude += dlon;
                //debug
                //println!("longitude: {}", longitude);
            }
        }
    }
    Ok((epoch, altitude, plane))
}

#[cfg(feature = "qc")]
use qc_traits::MergeError;

#[cfg(feature = "qc")]
pub(crate) fn merge_mut(lhs: &mut Record, rhs: &Record) -> Result<(), MergeError> {
    for (eh, plane) in rhs {
        if let Some(lhs_plane) = lhs.get_mut(eh) {
            for (latlon, plane) in plane {
                if let Some(tec) = lhs_plane.get_mut(latlon) {
                    if let Some(rms) = plane.rms {
                        if tec.rms.is_none() {
                            tec.rms = Some(rms);
                        }
                    }
                } else {
                    lhs_plane.insert(*latlon, plane.clone());
                }
            }
        } else {
            lhs.insert(*eh, plane.clone());
        }
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_new_tec_map() {
        assert!(is_new_tec_plane(
            "1                                                      START OF TEC MAP"
        ));
        assert!(!is_new_tec_plane(
            "1                                                      START OF RMS MAP"
        ));
        assert!(is_new_rms_plane(
            "1                                                      START OF RMS MAP"
        ));
        // assert_eq!(
        //     is_new_height_map(
        //         "1                                                      START OF HEIGHT MAP"
        //     ),
        //     true
        // );
    }
    //#[test]
    //fn test_merge_map2d() {
    //}
}
