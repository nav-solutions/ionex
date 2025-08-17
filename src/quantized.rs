const EPSILON: f64 = 1e-5;

#[cfg(doc)]
use crate::{coordinates::QuantizedCoordinates, prelude::TEC};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

/// [Quantized] value representing either a [TEC] estimate,
/// or discrete coordinates as [QuantizedCoordinates].
#[derive(Debug, Default, Copy, Clone, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Quantized {
    /// Exponent (scaling)
    pub exponent: i8,

    /// Quantized value
    pub value: i64,
}

impl PartialEq for Quantized {
    fn eq(&self, rhs: &Self) -> bool {
        self.real_value().eq(&rhs.real_value())
    }
}

impl PartialOrd for Quantized {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        self.real_value().partial_cmp(&rhs.real_value())
    }
}

impl Eq for Quantized {}

impl Ord for Quantized {
    fn cmp(&self, rhs: &Self) -> Ordering {
        self.real_value().total_cmp(&rhs.real_value())
    }
}

impl Quantized {
    /// Determines best suited exponent to quantize given value
    pub(crate) fn find_exponent(value: f64) -> i8 {
        let mut val = value;
        let mut exponent = 0;

        while val.fract() != 0.0 {
            val *= 10.0;
            exponent += 1;
        }

        exponent
    }

    pub fn new(value: f64, exponent: i8) -> Self {
        let value = (value * 10.0_f64.powi(exponent as i32)).round() as i64;
        Self { value, exponent }
    }

    /// Quantizes given value, automatically selecting most appropriate
    /// scaling, returning a [Quantized] value.
    pub fn auto_scaled(value: f64) -> Self {
        let exponent = Self::find_exponent(value);
        Self::new(value, exponent)
    }

    /// Returns quantized value
    pub fn real_value(&self) -> f64 {
        self.value as f64 / 10.0_f64.powi(self.exponent as i32)
    }
}

#[cfg(test)]
mod test {
    use super::Quantized;

    #[test]
    fn test_exponent_finder() {
        assert_eq!(Quantized::find_exponent(5.0), 0);
        assert_eq!(Quantized::find_exponent(5.5), 1);
        assert_eq!(Quantized::find_exponent(0.5), 1);
        assert_eq!(Quantized::find_exponent(1.25), 2);
        assert_eq!(Quantized::find_exponent(0.25), 2);
        assert_eq!(Quantized::find_exponent(0.333), 3);
    }

    #[test]
    fn test_quantized_ordering() {
        assert!(Quantized::new(1.0, 0) > Quantized::new(0.1, 0));
        assert!(Quantized::new(1.0, 0) < Quantized::new(1.1, 1));
        assert!(Quantized::new(1.0, 0) <= Quantized::new(1.1, 1));
        assert!(Quantized::new(1.12, 3) > Quantized::new(1.1, 1));
        assert!(Quantized::new(1.101, 4) > Quantized::new(1.1, 1));
        assert!(Quantized::new(-1.0, 1) < Quantized::new(0.0, 1));
    }

    #[test]
    fn test_quantization() {
        for (real_value, exponent, quantized) in [
            (1.0, 0, 1),
            (1.0, 1, 10),
            (1.1, 1, 11),
            (1.25, 2, 125),
            (1.333, 3, 3333),
            (-3.215, 3, -3215),
        ] {
            let q = Quantized::new(real_value, exponent);

            assert_eq!(
                q.real_value(),
                real_value,
                "test failed for {} 10**{}",
                real_value,
                exponent
            );
        }
    }
}
