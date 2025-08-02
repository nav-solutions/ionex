#[cfg(doc)]
use crate::prelude::{QuantizedCoordinates, TEC};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [Quantized] value representing either a [TEC] estimate,
/// or discrete coordinates as [QuantizedCoordinates].
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Quantized {
    /// Exponent (scaling)
    pub(crate) exponent: i8,

    /// Quantized value
    pub(crate) quantized: i64,
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

    /// Quantizes given value, using exponent scaling, returning
    /// a [Quantized] value.
    pub fn new(value: f64, exponent: i8) -> Self {
        let quantized = (value * 10.0_f64.powi(exponent as i32)).round() as i64;
        Self {
            quantized,
            exponent,
        }
    }

    /// Quantizes given value, automatically selecting most appropriate
    /// scaling, returning a [Quantized] value.
    pub fn new_auto_scaled(value: f64) -> Self {
        let exponent = Self::find_exponent(value);
        Self::new(value, exponent)
    }

    /// Returns real value as [f64]
    pub fn real_value_f64(&self) -> f64 {
        self.quantized as f64 / 10.0_f64.powi(self.exponent as i32)
    }

    /// Returns real value as [f32]
    pub fn real_value_f32(&self) -> f32 {
        self.real_value_f64() as f32
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
    }

    #[test]
    fn test_quantization() {
        let q = Quantized::new(1.0, 0);
        assert_eq!(
            q,
            Quantized {
                quantized: 1,
                exponent: 0,
            },
        );
        assert_eq!(q.real_value_f64(), 1.0);

        let q = Quantized::new(1.0, 1);
        assert_eq!(
            q,
            Quantized {
                quantized: 10,
                exponent: 1,
            },
        );
        assert_eq!(q.real_value_f64(), 1.0);

        let q = Quantized::new(1.25, 2);
        assert_eq!(
            q,
            Quantized {
                quantized: 125,
                exponent: 2,
            },
        );
        assert_eq!(q.real_value_f64(), 1.25);

        let q = Quantized::new(-3.215, 3);
        assert_eq!(
            q,
            Quantized {
                quantized: -3215,
                exponent: 3,
            },
        );
    }
}
