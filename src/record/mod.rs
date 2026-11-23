mod formatting;
mod parsing;

#[cfg(feature = "qc")]
mod qc;

use std::collections::{BTreeMap, btree_map::Iter};

use itertools::Itertools;

use crate::{
    coordinates::QuantizedCoordinates,
    prelude::{Epoch, Key, MapCell, TEC},
    quantized::Quantized,
};

/// IONEX [Record] contains [MapCell]s in chronological order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Record {
    pub(crate) map: BTreeMap<Key, TEC>,
}

impl Record {
    /// Insert new [TEC] value into IONEX [Record]
    pub fn insert(&mut self, key: Key, tec: TEC) {
        self.map.insert(key, tec);
    }

    /// Obtain [Record] iterator.
    pub fn iter(&self) -> Iter<'_, Key, TEC> {
        self.map.iter()
    }

    /// Obtain mutable [Record] iterator.
    pub fn iter_mut(&mut self) -> Box<dyn Iterator<Item = (Key, &mut TEC)> + '_> {
        Box::new(self.map.iter_mut().map(|(k, v)| (*k, v)))
    }

    /// Obtain [Record] Iterator at specific point in time
    pub fn synchronous_iter(&self, epoch: Epoch) -> Box<dyn Iterator<Item = (Key, TEC)> + '_> {
        Box::new(self.iter().filter_map(move |(k, v)| {
            if k.epoch == epoch {
                Some((*k, *v))
            } else {
                None
            }
        }))
    }

    /// Obtain mutable synchronous [Record] iterator
    pub fn synchronous_iter_mut(
        &mut self,
        epoch: Epoch,
    ) -> Box<dyn Iterator<Item = (Key, &mut TEC)> + '_> {
        Box::new(self.iter_mut().filter_map(
            move |(k, v)| {
                if k.epoch == epoch { Some((k, v)) } else { None }
            },
        ))
    }

    /// Obtain [TEC] (single point) from IONEX [Record], at specified spatial and temporal coordinates that must exist.
    /// This is an indexing method, not an interpolation method.  
    pub fn get(&self, key: &Key) -> Option<&TEC> {
        self.map.get(key)
    }

    /// Obtain a [MapCell] (4 single points) from IONEX [Record], at specified point in time and coordinates.
    /// The coordinates

    /// Obtain mutable [TEC] reference from IONEX [Record], at specified spatial and temporal coordinates that must exist.
    /// This is an indexing method, not an interpolation method.  
    /// For interpolation, use the [MapCell] API.
    pub fn get_mut(&mut self, key: &Key) -> Option<&mut TEC> {
        self.map.get_mut(key)
    }

    /// Collect IONEX [Record] from a list of [MapCell]s.
    /// This is particularly useful to reconstruct a [IONEX] file from a possibly processed
    /// and modified slice of [MapCell]s.
    ///
    /// ## Input
    /// - slice: slice of [MapCell]s that must have identical dimensions,
    /// otherwise this operation will result in corrupt/illegal content,
    /// and we do not verify it!
    /// - fixed_altitude_km: the fixed altitude in kilometers,
    /// use to represent the IONEX plane from the slice of planar [MapCell]s
    pub fn from_map_cells(slice: &[MapCell], fixed_altitude_km: f64) -> Self {
        let mut map = BTreeMap::<Key, TEC>::default();

        for cell in slice.iter() {
            // for each cell, we can produce 4 points
            // we take advantage of the map to avoid replicated points
            let (ne_point, nw_point, sw_point, se_point) = (
                cell.north_east.point,
                cell.north_west.point,
                cell.south_west.point,
                cell.south_east.point,
            );

            let (ne_tec, nw_tec, sw_tec, se_tec) = (
                cell.north_east.tec,
                cell.north_west.tec,
                cell.south_west.tec,
                cell.south_east.tec,
            );

            let (ne_key, nw_key, sw_key, se_key) = (
                Key::from_decimal_degrees_km(
                    cell.epoch,
                    ne_point.y(),
                    ne_point.x(),
                    fixed_altitude_km,
                ),
                Key::from_decimal_degrees_km(
                    cell.epoch,
                    nw_point.y(),
                    nw_point.x(),
                    fixed_altitude_km,
                ),
                Key::from_decimal_degrees_km(
                    cell.epoch,
                    sw_point.y(),
                    sw_point.x(),
                    fixed_altitude_km,
                ),
                Key::from_decimal_degrees_km(
                    cell.epoch,
                    se_point.y(),
                    se_point.x(),
                    fixed_altitude_km,
                ),
            );

            map.insert(ne_key, ne_tec);
            map.insert(nw_key, nw_tec);
            map.insert(sw_key, sw_tec);
            map.insert(se_key, se_tec);
        }

        Self { map }
    }

    /// Obtain [Epoch]s Iterator in chronological order.
    pub fn epochs_iter(&self) -> Box<dyn Iterator<Item = Epoch> + '_> {
        Box::new(self.map.keys().map(|k| k.epoch).unique())
    }

    /// Returns first [Epoch] in chronological order
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.map.keys().next().map(|k| k.epoch)
    }

    /// Returns last [Epoch] in chronological order
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.map.keys().next_back().map(|k| k.epoch)
    }

    /// Smallest (resp. largest) possible [Key] at this [Epoch]
    fn epoch_bound(epoch: Epoch, lower: bool) -> Key {
        let value = if lower { i64::MIN } else { i64::MAX };
        let q = Quantized { value, exponent: 0 };

        Key {
            epoch,
            coordinates: QuantizedCoordinates::from_quantized(q, q, q),
        }
    }

    /// Returns true if this [Record] contains a map at this [Epoch]
    pub fn has_epoch(&self, epoch: Epoch) -> bool {
        self.map
            .range(Self::epoch_bound(epoch, true)..=Self::epoch_bound(epoch, false))
            .next()
            .is_some()
    }

    /// Returns the latest [Epoch] that is prior or equal to this [Epoch]
    pub fn epoch_before(&self, epoch: Epoch) -> Option<Epoch> {
        self.map
            .range(..=Self::epoch_bound(epoch, false))
            .next_back()
            .map(|(k, _)| k.epoch)
    }

    /// Returns the earliest [Epoch] that is posterior or equal to this [Epoch]
    pub fn epoch_after(&self, epoch: Epoch) -> Option<Epoch> {
        self.map
            .range(Self::epoch_bound(epoch, true)..)
            .next()
            .map(|(k, _)| k.epoch)
    }

    /// Returns the (t0, t1) pair of map [Epoch]s that surround this [Epoch],
    /// with t0 <= epoch <= t1. Both are equal when a map exists at this very [Epoch].
    /// Returns None when this [Epoch] is outside of the temporal axis.
    pub fn epoch_bracket(&self, epoch: Epoch) -> Option<(Epoch, Epoch)> {
        let (t0, t1) = (self.epoch_before(epoch)?, self.epoch_after(epoch)?);
        Some((t0, t1))
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::str::FromStr;

    #[test]
    fn epoch_bracketing() {
        let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap();

        let t0 = Epoch::from_str("2022-01-02T00:00:00 UTC").unwrap();
        let t_end = Epoch::from_str("2022-01-03T00:00:00 UTC").unwrap();

        assert_eq!(ionex.record.first_epoch(), Some(t0));
        assert_eq!(ionex.record.last_epoch(), Some(t_end));

        let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();
        assert!(ionex.record.has_epoch(noon));
        assert_eq!(ionex.record.epoch_bracket(noon), Some((noon, noon)));

        let t = Epoch::from_str("2022-01-02T12:20:00 UTC").unwrap();
        assert!(!ionex.record.has_epoch(t));

        let one_pm = Epoch::from_str("2022-01-02T13:00:00 UTC").unwrap();
        assert_eq!(ionex.record.epoch_bracket(t), Some((noon, one_pm)));

        let before = t0 - Duration::from_seconds(1.0);
        let after = t_end + Duration::from_seconds(1.0);
        assert_eq!(ionex.record.epoch_bracket(before), None);
        assert_eq!(ionex.record.epoch_bracket(after), None);
    }

    #[test]
    fn ckmg_maps_cells_repiprocal() {
        let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap_or_else(|e| {
            panic!("Failed to parse CKMG0020: {}", e);
        });

        // grab all cells
        let map_cells = ionex.map_cell_iter().collect::<Vec<_>>();

        // build from scratch
        let record = Record::from_map_cells(&map_cells, ionex.header.grid.altitude.start);

        // reciprocal
        assert_eq!(record, ionex.record);
    }

    #[test]
    fn jplg_maps_cells_repiprocal() {
        let ionex = IONEX::from_gzip_file("data/IONEX/V1/jplg0010.17i.gz").unwrap_or_else(|e| {
            panic!("Failed to parse CKMG0020: {}", e);
        });

        // grab all cells
        let map_cells = ionex.map_cell_iter().collect::<Vec<_>>();

        // build from scratch
        let record = Record::from_map_cells(&map_cells, ionex.header.grid.altitude.start);

        // reciprocal
        assert_eq!(record, ionex.record);
    }
}
