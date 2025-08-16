use crate::{
    epoch::format_body as format_epoch,
    error::FormattingError,
    fmt_ionex,
    prelude::{Header, Record},
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
        const NUM_LONGITUDES_PER_LINE: usize = 16;

        let grid = header.grid;

        let (altitude_low_km, altitude_high_km, altitude_spacing_km) = (
            grid.altitude.start,
            grid.altitude.end,
            grid.altitude.spacing,
        );

        let (latitude_north_ddeg, latitude_south_ddeg, latitude_spacing_ddeg) = (
            grid.latitude.start,
            grid.latitude.end,
            grid.latitude.spacing,
        );

        let (longitude_east_ddeg, longitude_west_ddeg, longitude_spacing_ddeg) = (
            grid.longitude.start,
            grid.longitude.end,
            grid.longitude.spacing,
        );

        let nth_map = 0;
        let has_h = false;

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

            writeln!(
                w,
                "{}",
                fmt_ionex(&format!("{:6}", nth_map + 1), "END OF TEC MAP")
            )?;
        }

        Ok(())
    }
}
