mod parsing;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::{
    fmt_ionex,
    linspace::Linspace,
    prelude::{
        BiasSource, Comments, Duration, Epoch, FormattingError, Grid, MappingFunction,
        ReferenceSystem, Version,
    },
};

use std::io::{BufWriter, Write};

/// IONEX file [Header]
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
    /// File [Version]
    pub version: Version,

    /// Name of production software
    pub program: Option<String>,

    /// Name of operator (usually agency in IONEX)
    /// running this software
    pub run_by: Option<String>,

    /// Product date and time as readable string
    pub date: Option<String>,

    /// Possible file license
    pub license: Option<String>,

    /// Possible Digital Object ID (DOI)
    pub doi: Option<String>,

    /// Total number of TEC maps
    pub number_of_maps: usize,

    /// Epoch of first map
    pub epoch_of_first_map: Epoch,

    /// Epoch of last map
    pub epoch_of_last_map: Epoch,

    /// [ReferenceSystem] used in the following evaluation
    /// of the TEC maps.
    pub reference_system: ReferenceSystem,

    /// It is highly recommended to give a brief description
    /// of the technique, model.. description is not a
    /// general purpose comment
    pub description: Option<String>,

    /// Mapping function adopted for TEC determination,
    /// if None: No mapping function, e.g altimetry
    pub mapf: MappingFunction,

    /// Maps dimension, can either be a 2D (= fixed altitude mode), or 3D
    pub map_dimension: u8,

    /// Mean earth radius or bottom of height grid, in kilometers.
    pub base_radius_km: f32,

    /// Sampling period, duration gap between two maps.
    pub sampling_period: Duration,

    /// Map [Grid] definition.
    pub grid: Grid,

    /// Minimum elevation angle filter used. In degrees.
    pub elevation_cutoff: f32,

    /// exponent: scaling to apply in current TEC blocs
    exponent: i8,

    /// Comments found in the header section
    pub comments: Comments,
}

impl Default for Header {
    fn default() -> Self {
        Self {
            // default exponent value
            // this is very important: it allows to parse correctly
            // files that omit the exponent
            exponent: -1,
            number_of_maps: 0,
            // 2D by default
            map_dimension: 2,
            mapf: Default::default(),
            comments: Default::default(),
            description: Default::default(),
            elevation_cutoff: 0.0,
            // Standard Earth radius [km]
            base_radius_km: 6371.0,
            grid: Grid::default(),
            epoch_of_last_map: Epoch::default(),
            epoch_of_first_map: Epoch::default(),
            sampling_period: Duration::from_hours(1.0),
            reference_system: ReferenceSystem::default(),
            version: Default::default(),
            program: Default::default(),
            run_by: Default::default(),
            date: Default::default(),
            license: Default::default(),
            doi: Default::default(),
        }
    }
}

impl Header {
    /// Formats [Header] into [BufWriter].
    pub(crate) fn format<W: Write>(&self, w: &mut BufWriter<W>) -> Result<(), FormattingError> {
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{:6}", self.map_dimension), "MAP DIMENSION")
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
                &format!("{} {} {}", start, end, spacing),
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
                &format!("{} {} {}", start, end, spacing),
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
                &format!("{} {} {}", start, end, spacing),
                "LON1 / LON2 / DLON"
            )
        )?;

        // elevation cutoff
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{}", self.elevation_cutoff), "ELEVATION CUTOFF")
        )?;

        // mapping function
        writeln!(
            w,
            "{}",
            fmt_ionex(&format!("{}", self.mapf), "MAPPING FUNCTION")
        )?;

        // time of first map
        writeln!(w, "{}", fmt_ionex("TODO", "EPOCH OF FIRST MAP"))?;

        // time of last map
        writeln!(w, "{}", fmt_ionex("TODO", "EPOCH OF LAST MAP"))?;

        Ok(())
    }

    /// Copies self and returns with updated number of maps
    pub fn with_number_of_maps(&self, num: usize) -> Self {
        let mut s = self.clone();
        s.number_of_maps = num;
        s
    }

    /// Copies self with given time of first map
    pub fn with_epoch_of_first_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_first_map = t;
        s
    }

    /// Copies self with given time of last map
    pub fn with_epoch_of_last_map(&self, t: Epoch) -> Self {
        let mut s = self.clone();
        s.epoch_of_last_map = t;
        s
    }

    /// Copies and builds Self with updated [ReferenceSystem].
    pub fn with_reference_system(&self, reference: ReferenceSystem) -> Self {
        let mut s = self.clone();
        s.reference_system = reference;
        s
    }

    /// Copies and sets exponent / scaling to currently use
    pub fn with_exponent(&self, e: i8) -> Self {
        let mut s = self.clone();
        s.exponent = e;
        s
    }

    /// Copies and sets model description
    pub fn with_description(&self, desc: &str) -> Self {
        let mut s = self.clone();
        if let Some(ref mut d) = s.description {
            d.push(' ');
            d.push_str(desc)
        } else {
            s.description = Some(desc.to_string())
        }
        s
    }

    /// Copies and returns new [Header] with updated [MappingFunction];
    pub fn with_mapping_function(&self, mapf: MappingFunction) -> Self {
        let mut s = self.clone();
        s.mapf = mapf;
        s
    }

    /// Copies & sets minimum elevation angle used.
    pub fn with_elevation_cutoff(&self, e: f32) -> Self {
        let mut s = self.clone();
        s.elevation_cutoff = e;
        s
    }

    /// Copies & set Base Radius in km
    pub fn with_base_radius_km(&self, base_radius_km: f32) -> Self {
        let mut s = self.clone();
        s.base_radius_km = base_radius_km;
        s
    }

    pub fn with_map_dimension(&self, dim: u8) -> Self {
        let mut s = self.clone();
        s.map_dimension = dim;
        s
    }

    /// Adds latitude grid definition
    pub fn with_latitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.latitude = grid;
        s
    }

    /// Adds longitude grid definition
    pub fn with_longitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.longitude = grid;
        s
    }

    /// Adds altitude grid definition
    pub fn with_altitude_grid(&self, grid: Linspace) -> Self {
        let mut s = self.clone();
        s.grid.altitude = grid;
        s
    }
}
