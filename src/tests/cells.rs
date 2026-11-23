use std::str::FromStr;

use crate::prelude::{Epoch, Geometry, IONEX, Key, Point};

#[test]
fn ckmg_map_cells() {
    let ionex = IONEX::from_gzip_file("data/IONEX/V1/CKMG0020.22I.gz").unwrap();

    let noon = Epoch::from_str("2022-01-02T12:00:00 UTC").unwrap();

    // 70 latitude intervals x 72 longitude intervals
    let cells = ionex.synchronous_map_cell_iter(noon).collect::<Vec<_>>();
    assert_eq!(cells.len(), 70 * 72);

    // 25 maps
    assert_eq!(ionex.map_cell_iter().count(), 25 * 70 * 72);

    let altitude_km = ionex.header.grid.altitude.start;

    for cell in cells.iter() {
        assert_eq!(cell.epoch, noon);
        assert_eq!(cell.latitude_longitude_span_degrees(), (2.5, 5.0));

        // corners are correctly oriented
        assert!(cell.north_east.point.y() > cell.south_east.point.y());
        assert!(cell.north_east.point.x() > cell.north_west.point.x());

        // corners match the record
        for corner in [
            cell.north_east,
            cell.north_west,
            cell.south_east,
            cell.south_west,
        ] {
            let key =
                Key::from_decimal_degrees_km(noon, corner.point.y(), corner.point.x(), altitude_km);

            let tec = ionex.record.get(&key).unwrap();
            assert_eq!(*tec, corner.tec);

            // the interpolation preserves the grid nodes
            let interpolated = cell.spatial_tec_interp(corner.point).unwrap();
            assert!((interpolated.tecu() - tec.tecu()).abs() < 1E-9);
        }

        // direct lookup returns a cell containing the center
        let center = cell.center();
        let found = ionex.map_cell_at(noon, center).unwrap();
        assert_eq!(&found, cell);
        assert!(found.contains(&Geometry::Point(center)));
    }

    // longitude wrapping
    let cell = ionex.map_cell_at(noon, Point::new(182.0, 10.0)).unwrap();
    assert!(cell.contains(&Geometry::Point(Point::new(-178.0, 10.0))));

    // outside latitude range
    assert!(ionex.map_cell_at(noon, Point::new(0.0, 88.0)).is_none());

    // epoch that does not exist
    let t = Epoch::from_str("2022-01-02T12:30:00 UTC").unwrap();
    assert!(ionex.map_cell_at(t, Point::new(0.0, 0.0)).is_none());
}
