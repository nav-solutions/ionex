//! Interpolation strategies.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [TemporalInterpolation] strategy, used when the desired instant does not
/// line up with one of the maps. These are the three strategies described
/// in the IONEX specifications (Schaer et al., 1998), section 2.
/// When the instant lines up with a map, all strategies return that map.
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum TemporalInterpolation {
    /// Use the map that is the nearest in time.
    /// Simplest but least accurate, introduces discontinuities.
    NearestMap,

    /// Linear interpolation between the two consecutive maps,
    /// evaluated at the same (Earth-fixed) coordinates.
    Linear,

    /// Linear interpolation between the two consecutive maps, each rotated
    /// around the Earth axis by the time difference (15°/h), to compensate for the
    /// strong correlation between the ionosphere and the Sun position.
    /// This is the strategy recommended by the IONEX specifications,
    /// and the default one.
    ///
    /// E(t) = (T1 - t)/(T1 - T0) E0(lat, long + (t - T0)) + (t - T0)/(T1 - T0) E1(lat, long + (t - T1))
    ///
    /// Note that the rotated coordinates must still be within the map: this does not
    /// matter for worldwide maps (longitude wrapping), but it may fail near the
    /// eastern/western borders of regional maps, in which case you should prefer [Self::Linear].
    #[default]
    RotatedMaps,
}

impl TemporalInterpolation {
    /// Earth rotation rate relative to the Sun, in degrees per second (360° per day).
    pub const SUN_FIXED_ROTATION_DEG_S: f64 = 360.0 / 86400.0;
}
