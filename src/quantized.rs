#[cfg(doc)]
use crate::{coordinates::QuantizedCoordinates, prelude::TEC};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use std::cmp::Ordering;

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
    /// Maximal number of decimal digits we will ever preserve.
    /// This is far beyond the precision of any IONEX content
    /// and bounds the quantization of computed (interpolated) values,
    /// that otherwise carry all the floating point noise.
    pub(crate) const MAX_EXPONENT: i8 = 15;

    /// Determines best suited exponent to quantize given value.
    /// Non finite values (NaN, infinite) are not quantizable and return 0.
    /// The exponent is bounded so the quantized value always fits in a i64.
    pub(crate) fn find_exponent(value: f64) -> i8 {
        if !value.is_finite() {
            return 0;
        }

        let mut val = value;
        let mut exponent = 0;

        while val.fract() != 0.0 && exponent < Self::MAX_EXPONENT {
            val *= 10.0;
            exponent += 1;
        }

        // make sure the scaled value fits in a i64
        while exponent > 0 && (value * 10.0_f64.powi(exponent as i32)).abs() >= i64::MAX as f64 {
            exponent -= 1;
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

        // non quantizable values
        assert_eq!(Quantized::find_exponent(f64::NAN), 0);
        assert_eq!(Quantized::find_exponent(f64::INFINITY), 0);
        assert_eq!(Quantized::find_exponent(f64::NEG_INFINITY), 0);

        // floating point noise is bounded
        assert!(Quantized::find_exponent(0.1 + 0.2) <= Quantized::MAX_EXPONENT);
        assert!(Quantized::find_exponent(1.0 / 3.0) <= Quantized::MAX_EXPONENT);
        assert!(Quantized::find_exponent(1.0E-30) <= Quantized::MAX_EXPONENT);
    }

    #[test]
    fn test_auto_scaled_noisy_values() {
        for value in [
            0.1 + 0.2,
            1.0 / 3.0,
            23.456789123456789,
            -12.000000000000002,
        ] {
            let q = Quantized::auto_scaled(value);
            assert!(
                (q.real_value() - value).abs() < 1.0E-12,
                "{} -> {:?}",
                value,
                q
            );
        }
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
        for (real_value, exponent) in [
            (1.0, 0),
            (1.0, 1),
            (1.1, 1),
            (1.25, 2),
            (1.333, 3),
            (-3.215, 3),
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
