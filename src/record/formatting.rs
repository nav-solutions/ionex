use crate::{
    coordinates::QuantizedCoordinates,
    epoch::format_body as format_epoch,
    error::FormattingError,
    fmt_ionex,
    prelude::{Epoch, Header, Key, Record, TEC},
};

use std::io::{BufWriter, Write};

/// Number of values per line
const VALUES_PER_LINE: usize = 16;

/// Omitted value
const OMITTED: &str = " 9999";

impl Record {
    /// Formats one map (TEC or RMS) at this [Epoch], for all altitude layers.
    fn format_map<W: Write, F: Fn(&TEC) -> Option<f64>>(
        &self,
        header: &Header,
        epoch: Epoch,
        value: F,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        // IONEX stores quantized values as v = 10^-k (with k the exponent)
        let scaling = 10.0_f64.powi(-header.exponent as i32);

        for altitude in header.grid.altitude.quantize() {
            for latitude in header.grid.latitude.quantize() {
                // grid specs
                writeln!(
                    w,
                    "{}",
                    fmt_ionex(
                        &format!(
                            "  {:6.1}{:6.1}{:6.1}{:6.1}{:6.1}",
                            latitude.real_value(),
                            header.grid.longitude.start,
                            header.grid.longitude.end,
                            header.grid.longitude.spacing,
                            altitude.real_value(),
                        ),
                        "LAT/LON1/LON2/DLON/H"
                    )
                )?;

                let mut nth = 0;

                for longitude in header.grid.longitude.quantize() {
                    let key = Key {
                        epoch,
                        coordinates: QuantizedCoordinates::from_quantized(
                            latitude, longitude, altitude,
                        ),
                    };

                    match self.get(&key).and_then(&value) {
                        Some(value) => {
                            let quantized = (value * scaling).round() as i64;
                            write!(w, "{:5}", quantized)?;
                        },
                        None => {
                            write!(w, "{}", OMITTED)?;
                        },
                    }

                    nth += 1;

                    if nth % VALUES_PER_LINE == 0 {
                        writeln!(w)?;
                    }
                }

                if nth % VALUES_PER_LINE != 0 {
                    writeln!(w)?;
                }
            }
        }

        Ok(())
    }

    /// Format IONEX [Record] into [Write]able interface, using efficient buffering.
    /// This requires reference to attached [Header] section, which describes the grid
    /// (2D or 3D) and the scaling exponent of the TEC and RMS values.
    pub fn format<W: Write>(
        &self,
        header: &Header,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        let has_rms = self.map.values().any(|tec| tec.rms.is_some());

        // TEC maps
        for (nth_map, epoch) in self.epochs_iter().enumerate() {
            writeln!(
                w,
                "{}",
                fmt_ionex(&format!("{:6}", nth_map + 1), "START OF TEC MAP")
            )?;

            writeln!(
                w,
                "{}",
                fmt_ionex(&format_epoch(epoch), "EPOCH OF CURRENT MAP")
            )?;

            self.format_map(header, epoch, |tec| Some(tec.tecu()), w)?;

            writeln!(
                w,
                "{}",
                fmt_ionex(&format!("{:6}", nth_map + 1), "END OF TEC MAP")
            )?;
        }

        // RMS maps (if any)
        if has_rms {
            for (nth_map, epoch) in self.epochs_iter().enumerate() {
                writeln!(
                    w,
                    "{}",
                    fmt_ionex(&format!("{:6}", nth_map + 1), "START OF RMS MAP")
                )?;

                writeln!(
                    w,
                    "{}",
                    fmt_ionex(&format_epoch(epoch), "EPOCH OF CURRENT MAP")
                )?;

                self.format_map(header, epoch, |tec| tec.root_mean_square(), w)?;

                writeln!(
                    w,
                    "{}",
                    fmt_ionex(&format!("{:6}", nth_map + 1), "END OF RMS MAP")
                )?;
            }
        }

        // mark END of file
        writeln!(w, "{}", fmt_ionex("", "END OF FILE"))?;

        Ok(())
    }
}
