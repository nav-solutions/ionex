use crate::{
    epoch::format as format_epoch,
    fmt_ionex,
    prelude::{Duration, FormattingError, Header, Version},
};

use std::io::{BufWriter, Write};

impl Header {
    /// Format this [Header] into [Write]able interface, using efficient buffering.
    pub fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        let (major, minor) = (self.version.major, self.version.minor);

        writeln!(
            w,
            "{}",
            fmt_ionex(
                &format!("{major:6}.{minor:01}            IONOSPHERE MAPS     GNSS"),
                "IONEX VERSION / TYPE"
            )
        )?;

        let mut string = if let Some(program) = &self.program {
            format!("{program:<20}")
        } else {
            "                    ".to_string()
        };

        if let Some(runby) = &self.run_by {
            string.push_str(&format!("{runby:<20}"));
        } else {
            string.push_str("                    ");
        }

        if let Some(date) = &self.date {
            string.push_str(date);
        } else {
            string.push_str("                    ");
        };

        writeln!(w, "{}", fmt_ionex(&string, "PGM / RUN BY / DATE"))?;

        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:6}", self.map_dimension), "MAP DIMENSION")
        )?;

        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:6}", self.number_of_maps), "# OF MAPS IN FILE")
        )?;

        // altitude grid
        let (start, end, spacing) = (
            self.grid.altitude.start,
            self.grid.altitude.end,
            self.grid.altitude.spacing,
        );

        writeln!(
            w,
            "{}",
            fmt_ionex(
                &format!("  {:6.1}{:6.1}{:6.1}", start, end, spacing),
                "HGT1 / HGT2 / DHGT"
            )
        )?;

        // latitude grid
        let (start, end, spacing) = (
            self.grid.latitude.start,
            self.grid.latitude.end,
            self.grid.latitude.spacing,
        );

        writeln!(
            w,
            "{}",
            fmt_ionex(
                &format!("  {:6.1}{:6.1}{:6.1}", start, end, spacing),
                "LAT1 / LAT2 / DLAT"
            )
        )?;

        // longitude grid
        let (start, end, spacing) = (
            self.grid.longitude.start,
            self.grid.longitude.end,
            self.grid.longitude.spacing,
        );

        writeln!(
            w,
            "{}",
            fmt_ionex(
                &format!("  {:6.1}{:6.1}{:6.1}", start, end, spacing),
                "LON1 / LON2 / DLON"
            )
        )?;

        // INTERVAL
        let sampling_period_secs = self.sampling_period.to_seconds().round() as u32;

        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{sampling_period_secs:6}"), "INTERVAL")
        )?;

        // time of first map
        writeln!(
            w,
            "{}",
            fmt_ionex(&format_epoch(self.epoch_of_first_map), "EPOCH OF FIRST MAP")
        )?;

        // time of last map
        writeln!(
            w,
            "{}",
            fmt_ionex(&format_epoch(self.epoch_of_last_map), "EPOCH OF LAST MAP")
        )?;

        // elevation cutoff
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:8}", self.elevation_cutoff), "ELEVATION CUTOFF")
        )?;

        // mapping function
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("  {}", self.mapf), "MAPPING FUNCTION")
        )?;

        // Base radius
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:6}", self.base_radius_km), "BASE RADIUS")
        )?;

        // First exponent
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:6}", self.exponent), "EXPONENT")
        )?;

        // COMMENTS
        for comment in self.comments.iter() {
            writeln!(w, "{}", fmt_ionex(comment, "COMMENTS"))?;
        }

        writeln!(w, "{}", fmt_ionex("", "END OF HEADER"))?;

        Ok(())
    }
}
