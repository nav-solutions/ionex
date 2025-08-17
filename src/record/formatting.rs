use crate::{
    coordinates::QuantizedCoordinates,
    epoch::format_body as format_epoch,
    error::FormattingError,
    fmt_ionex,
    prelude::{Header, Key, Record},
};

use std::io::{BufWriter, Write};

impl Record {
    /// Format IONEX [Record] into [Write]able interface, using efficient buffering.
    /// This requires reference to attached [Header] section.
    pub fn format<W: Write>(
        &self,
        header: &Header,
        w: &mut BufWriter<W>,
    ) -> Result<(), FormattingError> {
        const FORMATTED_OFFSET: usize = 5;

        // NB: this will not work if
        // - grid accuracy changes between regions or epochs
        // - map is not 2D
        // - does not support scaling update very smoothly

        let (latitude_min, latitude_max) = header.grid.latitude.minmax();
        let (longitude_min, longitude_max) = header.grid.longitude.minmax();

        let mut line_offset;
        let mut longitude_ptr_ddeg;

        // TEC MAPs. Grid browsing:
        // - browse latitude (starting on northernmost.. to southernmost)
        //  - browse longitude (starting on easternmost.. to westernmost)
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

            let mut latitude_ptr_ddeg = latitude_max; // following GEO standard angles

            while latitude_ptr_ddeg >= latitude_min {
                // println!("lat_ptr={}", latitude_ptr_ddeg);

                line_offset = 0;
                longitude_ptr_ddeg = longitude_min;

                // grid specs
                writeln!(
                    w,
                    "{}",
                    fmt_ionex(
                        &format!(
                            "  {:6.1}{:6.1}{:6.1}{:6.1}{:6.1}",
                            latitude_ptr_ddeg,
                            header.grid.longitude.start,
                            header.grid.longitude.end,
                            header.grid.longitude.spacing,
                            header.grid.altitude.start,
                        ),
                        "LAT/LON1/LON2/DLON/H"
                    )
                )?;

                while longitude_ptr_ddeg <= longitude_max {
                    // println!("long_ptr={}", longitude_ptr_ddeg);
                    // obtain coordinates
                    let coordinates = QuantizedCoordinates::from_decimal_degrees(
                        latitude_ptr_ddeg,
                        longitude_ptr_ddeg,
                        header.grid.altitude.start,
                    );

                    let key = Key { epoch, coordinates };

                    // format map
                    if let Some(tec) = self.get(&key) {
                        write!(w, "{:5}", tec.tecu.value)?;
                    } else {
                        write!(w, " 9999")?; // standardized
                    }

                    line_offset += FORMATTED_OFFSET;

                    if line_offset >= 80 {
                        write!(w, "{}", '\n')?;
                        line_offset = 0;
                    }

                    longitude_ptr_ddeg += header.grid.longitude.spacing;
                }

                if line_offset != 80 {
                    // needs termination
                    write!(w, "{}", '\n')?;
                }

                latitude_ptr_ddeg += header.grid.latitude.spacing;
            }

            writeln!(
                w,
                "{}",
                fmt_ionex(&format!("{:6}", nth_map + 1), "END OF TEC MAP")
            )?;
        }

        // mark END of file
        writeln!(w, "{}", fmt_ionex("", "END OF FILE"))?;

        Ok(())
    }
}
