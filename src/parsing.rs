use crate::{
    epoch::parse_utc as parse_utc_epoch,
    error::ParsingError,
    grid::GridSpecs,
    prelude::{Comments, Header, Key, Quantized, QuantizedCoordinates, Record, TEC},
};

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

#[cfg(feature = "log")]
use log::{
    //debug,
    error,
    trace,
};

pub(crate) fn parse_record<R: Read>(
    header: &mut Header,
    reader: &mut BufReader<R>,
) -> Result<(Record, Comments), ParsingError> {
    let mut eos = false;
    let mut rms_map = false;
    let mut height_map = false;

    let mut exponent = header.exponent;
    let mut epoch = header.epoch_of_first_map;

    let mut grid_specs = GridSpecs::default();
    let mut next_grid_specs = grid_specs.clone();

    let mut long_ptr;
    let mut longitude_exponent = 0i8;

    let mut record = Record::default();
    let mut comments = Comments::default();

    let mut line_buf = String::with_capacity(128);
    let mut epoch_buf = String::with_capacity(1024);

    let latitude_exponent: i8 = Quantized::find_exponent(header.grid.latitude.spacing);
    let altitude_exponent: i8 = Quantized::find_exponent(header.grid.altitude.spacing);

    while let Ok(size) = reader.read_line(&mut line_buf) {
        if size == 0 {
            // reached EOS
            eos = true;
        }

        let mut skip = false;

        let mut grid_specs_updated = false;

        if line_buf.len() > 60 {
            let (content, marker) = line_buf.split_at(60);

            // COMMENTS are stored as is
            if marker.contains("COMMENTS") {
                skip = true;
                let comment = line_buf.split_at(60).0.trim_end();
                comments.push(comment.to_string());
            }

            // Scaling update
            if marker.contains("EXPONENT") {
                skip = true;

                // parsing must pass
                exponent = content.trim().parse::<i8>().map_err(|e| {
                    #[cfg(feature = "log")]
                    error!("exponent parsing error: {}", e);
                    ParsingError::ExponentScaling
                })?;

                trace!("{} exponent updated to {}", epoch, exponent);
            }

            // Epoch update
            if marker.contains("EPOCH OF CURRENT MAP") {
                skip = true;
                epoch = parse_utc_epoch(content)?;
            }

            // New map
            if marker.contains("START OF TEC MAP") {
                skip = true;
                rms_map = false;
                height_map = false;
            }

            // New RMS map
            if marker.contains("START OF RMS MAP") {
                skip = true;
                rms_map = true;
                height_map = false;
            }

            // New height map
            if marker.contains("START OF HEIGHT MAP") {
                skip = true;
                rms_map = false;
                height_map = true;
            }

            // Specs update
            if marker.contains("LAT/LON1/LON2/DLON/H") {
                skip = true;

                match GridSpecs::from_str(content) {
                    Ok(specs) => {
                        next_grid_specs = specs;
                        grid_specs_updated = true;
                    },
                    Err(e) => {
                        error!("failed to parse grid specs: {}", e);
                    },
                }
            }

            // block parsing
            if marker.contains("END OF") || grid_specs_updated {
                skip = true;

                long_ptr = grid_specs.longitude_space.start;

                for item in epoch_buf.split_ascii_whitespace() {
                    let item = item.trim();
                    println!("ptr={} lat={}", long_ptr, grid_specs.latitude_ddeg);

                    // handles coordinates overflow (invalid file/specs)
                    if long_ptr > grid_specs.longitude_space.end {
                        break;
                    }

                    // omitted data
                    if item.eq("9999") {
                        // skip parsing
                        long_ptr += grid_specs.longitude_space.spacing;
                        continue;
                    }

                    // parsing
                    match item.parse::<i64>() {
                        Ok(value) => {
                            let (lat, long, alt) = (
                                Quantized::new(grid_specs.latitude_ddeg, latitude_exponent),
                                Quantized::new(long_ptr, longitude_exponent),
                                Quantized::new(grid_specs.altitude_km, altitude_exponent),
                            );

                            let coordinates = QuantizedCoordinates::from_quantized(lat, long, alt);

                            let key = Key { epoch, coordinates };

                            if rms_map {
                                if let Some(tec) = record.get_mut(&key) {
                                    tec.set_quantized_root_mean_square(value, exponent);
                                } else {
                                    let mut tec = TEC::default();
                                    tec.set_quantized_root_mean_square(value, exponent);
                                    record.insert(key, tec);
                                }
                            } else if height_map {
                                // TODO: Height map not supported.
                            } else {
                                if let Some(tec) = record.get_mut(&key) {
                                    *tec = tec.with_tecu(value as f64);
                                } else {
                                    let tec = TEC::from_quantized(value, exponent);
                                    record.insert(key, tec);
                                }
                            }
                        },
                        Err(e) => {
                            #[cfg(feature = "log")]
                            error!("tecu parsing error: {} (\"{}\")", e, item);
                        },
                    } // parsing

                    long_ptr += grid_specs.longitude_space.spacing;
                } // parsing

                epoch_buf.clear();
            } // block parsing attempt

            if marker.contains("END OF FILE") {
                eos = true;
            }

            if marker.contains("END OF RMS MAP") {
                rms_map = false;
            }

            if marker.contains("END OF HEIGHT MAP") {
                height_map = false;
            }
        } // line > 60

        if !skip {
            epoch_buf.push_str(&line_buf);
        }

        line_buf.clear();

        if grid_specs_updated {
            #[cfg(feature = "log")]
            trace!(
                "updated grid specs (lat={:+03.3}, long={:+03.3} dlon={:+03.1}, z={:+03.3})",
                next_grid_specs.latitude_ddeg,
                next_grid_specs.longitude_space.start,
                next_grid_specs.longitude_space.spacing,
                next_grid_specs.altitude_km
            );

            longitude_exponent = Quantized::find_exponent(next_grid_specs.longitude_space.spacing);

            grid_specs = next_grid_specs;
        }

        if eos {
            break;
        }
    }

    Ok((record, comments))
}
