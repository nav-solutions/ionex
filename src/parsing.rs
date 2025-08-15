use crate::{
    epoch::parse_utc as parse_utc_epoch,
    error::ParsingError,
    grid::GridSpecs,
    is_comment,
    prelude::{Comments, Epoch, Header, Key, Quantized, QuantizedCoordinates, Record, TEC},
};

use std::{
    io::{BufRead, BufReader, Read},
    str::FromStr,
};

#[cfg(feature = "log")]
use log::{debug, error, trace};

pub(crate) fn parse_record<R: Read>(
    header: &mut Header,
    reader: &mut BufReader<R>,
) -> Result<(Record, Comments), ParsingError> {
    let mut eos = false;
    let mut new_map = false;
    let mut rms_map = false;
    let mut height_map = false;

    let mut exponent = header.exponent;
    let mut epoch = header.epoch_of_first_map;

    let mut grid_specs = GridSpecs::default();
    let mut next_grid_specs = grid_specs.clone();

    let mut long_ptr = 0.0;
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
                "updated grid specs (lat={}, long={} dlon={}, z={})",
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

#[cfg(test)]
mod test {
    use super::Quantized;

    use crate::prelude::{Epoch, Key, QuantizedCoordinates, Record};

    //    #[test]
    //    fn tec_map_parsing() {
    //        let mut record = Record::default();
    //
    //
    //        let content =
    //            "     1                                                      START OF TEC MAP
    //  2017     1     1     0     0     0                        EPOCH OF CURRENT MAP
    //    87.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
    //   33   33   32   32   32   31   31   30   30   30   29   29   28   28   28   27
    //   27   27   26   26   26   26   26   26   26   26   26   26   26   26   26   26
    //   27   27   27   28   28   29   29   30   30   31   31   32   32   33   33   33
    //   34   34   35   35   35   35   36   36   36   36   36   36   36   36   36   35
    //   35   35   35   35   34   34   34   33   33
    //    85.0-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
    //   36   36   35   35   34   34   33   33   32   31   31   30   29   28   28   27
    //   26   25   25   24   24   23   23   22   22   22   22   22   22   23   23   24
    //   24   25   25   26   27   28   29   29   30   31   32   33   34   35   36   37
    //   38   39   39   40   41   41   41   41   42   42   42   41   41   41   41   40
    //   40   40   39   39   38   38   37   37   36
    //    27.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
    //   235  230  222  212  200  187  173  157  141  126  110   95   92   92   92   92
    //    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    //    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    //    92   92   92   92   92   92   92   92   92  104  120  136  151  166  180  193
    //   205  215  224  231  236  239  240  239  235
    //     2.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
    //   364  370  374  378  380  380  378  375  370  364  356  346  336  324  311  298
    //   283  269  253  238  222  207  191  175  159  143  127  111   96   92   92   92
    //    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    //    92   92   92   92   92  106  124  141  158  175  191  207  223  238  252  266
    //   280  293  305  317  328  339  348  356  364
    //    -2.5-180.0 180.0   5.0 450.0                            LAT/LON1/LON2/DLON/H
    //   363  370  375  380  383  385  385  384  381  376  370  363  354  343  332  319
    //   305  291  276  260  244  227  210  194  176  159  143  126  109   93   92   92
    //    92   92   92   92   92   92   92   92   92   92   92   92   92   92   92   92
    //    92   92   92   92  103  120  136  152  168  183  198  212  226  240  253  266
    //   279  291  303  315  326  336  346  355  363
    //     1                                                      END OF TEC MAP      ";
    //
    //        parse_tec_map(
    //            content,
    //            lat_exponent,
    //            long_exponent,
    //            alt_exponent,
    //            tec_exponent,
    //            epoch,
    //            &mut record,
    //        )
    //        .unwrap();
    //
    //        for (coordinates, quantized_tecu) in [
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    -180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                33,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    -175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                33,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    -170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                32,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                34,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                33,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    87.5,
    //                    lat_exponent,
    //                    180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                33,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    -180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                36,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    -175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                36,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    -170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                35,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                37,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                37,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    85.0,
    //                    lat_exponent,
    //                    180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                36,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    27.5,
    //                    lat_exponent,
    //                    170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                240,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    27.5,
    //                    lat_exponent,
    //                    175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                239,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    27.5,
    //                    lat_exponent,
    //                    180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                235,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    2.5,
    //                    lat_exponent,
    //                    -170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                374,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    2.5,
    //                    lat_exponent,
    //                    170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                348,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    2.5,
    //                    lat_exponent,
    //                    175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                356,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    2.5,
    //                    lat_exponent,
    //                    180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                364,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    -2.5,
    //                    lat_exponent,
    //                    -170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                375,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    -2.5,
    //                    lat_exponent,
    //                    170.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                346,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    -2.5,
    //                    lat_exponent,
    //                    175.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                355,
    //            ),
    //            (
    //                QuantizedCoordinates::new(
    //                    -2.5,
    //                    lat_exponent,
    //                    180.0,
    //                    long_exponent,
    //                    450.0,
    //                    alt_exponent,
    //                ),
    //                363,
    //            ),
    //        ] {
    //            let key = Key { epoch, coordinates };
    //
    //            let tec = record
    //                .get(&key)
    //                .expect(&format!("missing value at {:#?}", key));
    //
    //            let tecu = tec.tecu();
    //            let expected = quantized_tecu as f64 * 10.0_f64.powi(tec_exponent as i32);
    //            let err = (tecu - expected).abs();
    //
    //            assert!(err < 1.0E-6);
    //        }
    //    }
}
