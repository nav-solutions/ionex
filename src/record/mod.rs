mod formatting;
mod parsing;

#[cfg(feature = "qc")]
mod qc;

use std::collections::{BTreeMap, btree_map::Iter};

use itertools::Itertools;

use crate::prelude::{Epoch, Key, MapCell, TEC};

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
            let epoch = cell.epoch;

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
        self.epochs_iter().nth(0)
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    #[ignore]
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
    #[ignore]
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
