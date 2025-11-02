use std::ops::Rem;

use crate::{error::{ParsingError, Error}, quantized::Quantized};

/// Quantized Linspace for iteration
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct QuantizedLinspace {
    ptr: Quantized,
    pub start: Quantized,
    pub end: Quantized,
    pub spacing: Quantized,
}

impl Iterator for QuantizedLinspace {
    type Item = Quantized;

    fn next(&mut self) -> Option<Self::Item> {
        if self.ptr.value > self.end.value {
            return None;
        }

        let value = self.ptr;
        self.ptr.value += self.spacing.value;

        Some(value)
    }
}

/// Linear space as used in IONEX or Antenna grid definitions.
/// Linear space starting from `start` ranging to `end` (included).
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Linspace {
    /// First value
    pub start: f64,

    /// Last value (included)
    pub end: f64,

    /// Spacing (increment)
    pub spacing: f64,
}

impl Linspace {
    /// Quantized this [Linspace] returning a [QuantizedLinspace]
    pub fn quantize(&self) -> QuantizedLinspace {
        QuantizedLinspace {
            ptr: Quantized::auto_scaled(self.start),
            start: Quantized::auto_scaled(self.start),
            end: Quantized::auto_scaled(self.end),
            spacing: Quantized::auto_scaled(self.spacing),
        }
    }

    /// Returns smallest value between [Self::start] and [Self::end]
    pub fn min(&self) -> f64 {
        self.start.min(self.end)
    }

    /// Returns largest value between [Self::start] and [Self::end]
    pub fn max(&self) -> f64 {
        self.start.max(self.end)
    }

    /// Returns (smallest, largest) values tuplet
    pub fn minmax(&self) -> (f64, f64) {
        (self.min(), self.max())
    }

    /// Stretch this mutable [Linspace] by a positive, possibly fractional number,
    /// while preserving the initial grid quantization (point spacing).
    /// This only modifies the [Linspace] dimensions.
    /// To modify the sampling, use [Self::resample_mut].
    pub fn stretch_mut(&mut self, factor: f64) -> Result<(), Error> {
        if factor.is_sign_negative() {
            return Err(Error::NegativeStretchFactor);
        }
        self.start *= factor;
        self.end *= factor;
        Ok(())
    }

    /// Stretch this [Linspace] to a new [Linspace] definition. 
    /// The streetching factor must be a positive number.
    /// this preserves the initial grid quantization (point spacing).
    /// This only modifies the [Linspace] dimensions.
    /// To modify the sampling, use [Self::resampled].
    pub fn stretched(&self, factor: f64) -> Result<Self, Error> {
        let mut s = self.clone();
        s.stretch_mut(factor)?;
        Ok(s)
    }

    /// Resample this mutable [Linspace] so the point spacing is modified.
    /// This is a multiplicative stretching factor, which must be a positive number.
    /// This does not modify the [Linspace] dimensions, use [Self::stretch_mut] to modify dimensions.
    pub fn resample_mut(&mut self, factor: f64) -> Result<(), Error> {
        if factor.is_sign_negative() {
            return Err(Error::NegativeStretchFactor);
        }
        self.spacing *= factor;
        Ok(())
    }

    /// Resample this [Linspace] to a new [Linspace] definition, modifying the point 
    /// spacing but preserving the initial dimensions. The resampling factor
    /// must be positive, possibly fractional number. This does not modify the dimensions,
    /// use [Self::stretched] to modify the dimensions.
    pub fn resampled(&self, factor: f64) -> Result<Self, Error> {
        let mut s = self.clone();
        s.resample_mut(factor)?;
        Ok(s)
    }

    /// Builds a new Linear space
    pub fn new(start: f64, end: f64, spacing: f64) -> Result<Self, ParsingError> {
        if start == end && spacing == 0.0 {
            Ok(Self {
                start,
                end,
                spacing,
            })
        } else {
            let r = end.rem(start);
            /*
             * End / Start must be multiple of one another
             */
            if r == 0.0 {
                if end.rem(spacing) == 0.0 {
                    Ok(Self {
                        start,
                        end,
                        spacing,
                    })
                } else {
                    Err(ParsingError::InvalidGridDefinition)
                }
            } else {
                Err(ParsingError::InvalidGridDefinition)
            }
        }
    }

    /// Returns total width
    pub fn width(&self) -> f64 {
        self.end - self.start
    }

    /// Returns grid length, in terms of data points
    pub fn length(&self) -> usize {
        (self.end / self.spacing).floor() as usize
    }

    /// Returns true if self is a single point space
    pub fn is_single_point(&self) -> bool {
        (self.end == self.start) && self.spacing == 0.0
    }

    /// Returns nearest lower bound from point p in the [Linspace]
    pub fn nearest_lower(&self, p: f64) -> Option<f64> {
        let mut start = self.start;
        while start < self.end {
            if start > p {
                return Some(start - self.spacing);
            }
            start += self.spacing;
        }

        None
    }

    /// Returns nearest lower bound from point p in the [Linspace]
    pub fn nearest_above(&self, p: f64) -> Option<f64> {
        let lower = self.nearest_lower(p)?;
        Some(lower + self.spacing)
    }
}

impl From<(f64, f64, f64)> for Linspace {
    fn from(tuple: (f64, f64, f64)) -> Self {
        Self {
            start: tuple.0,
            end: tuple.1,
            spacing: tuple.2,
        }
    }
}

#[cfg(test)]
mod test {
    use super::Linspace;

    #[test]
    fn linspace() {
        let linspace = Linspace::new(1.0, 180.0, 1.0).unwrap();
        assert_eq!(linspace.length(), 180);
        assert!(!linspace.is_single_point());

        let linspace = Linspace::new(1.0, 180.0, 0.5).unwrap();
        assert_eq!(linspace.length(), 180 * 2);
        assert!(!linspace.is_single_point());

        let linspace = Linspace::new(350.0, 350.0, 0.0).unwrap();
        assert!(linspace.is_single_point());
    }

    #[test]
    fn latitude_linspace() {
        let linspace = Linspace::new(-87.5, 87.5, 2.5).unwrap();
        assert_eq!(linspace.nearest_lower(-85.0), Some(-85.0));
    }

    #[test]
    fn longitude_linspace() {
        let linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();
        assert_eq!(linspace.nearest_lower(-179.0), Some(-180.0));
    }

    #[test]
    fn test_grid() {
        let default = Linspace::default();

        assert_eq!(
            default,
            Linspace {
                start: 0.0,
                end: 0.0,
                spacing: 0.0,
            }
        );

        let linspace = Linspace::new(1.0, 10.0, 1.0).unwrap();

        assert_eq!(linspace.length(), 10);
        assert!(!linspace.is_single_point());
    }

    #[test]
    fn linspace_stretching() {
        let mut linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();

        linspace.stretch_mut(0.5).unwrap();
        assert_eq!(linspace.minmax(), (-90.0, 90.0));
        assert_eq!(linspace.spacing, 5.0, "linspace quantization not preserved!");
        
        linspace.stretch_mut(0.75).unwrap();
        assert_eq!(linspace.minmax(), (-67.5, 67.5));
        assert_eq!(linspace.spacing, 5.0, "linspace quantization not preserved!");
        
        linspace.stretch_mut(0.5).unwrap();
        assert_eq!(linspace.minmax(), (-33.75, 33.75));
        assert_eq!(linspace.spacing, 5.0, "linspace quantization not preserved!");
        
        linspace.stretch_mut(2.0).unwrap();
        assert_eq!(linspace.minmax(), (-67.5, 67.5));
        assert_eq!(linspace.spacing, 5.0, "linspace quantization not preserved!");
    }
    
    #[test]
    fn linspace_resampling() {
        let mut linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();

        linspace.resample_mut(0.5).unwrap();
        assert_eq!(linspace.spacing, 2.5);
        assert_eq!(linspace.minmax(), (-180.0, 180.0), "dimensions not preserved!");
        
        linspace.resample_mut(0.5).unwrap();
        assert_eq!(linspace.spacing, 1.25);
        assert_eq!(linspace.minmax(), (-180.0, 180.0), "dimensions not preserved!");
        
        linspace.resample_mut(2.0).unwrap();
        assert_eq!(linspace.spacing, 2.5);
        assert_eq!(linspace.minmax(), (-180.0, 180.0), "dimensions not preserved!");
        
        linspace.resample_mut(2.0).unwrap();
        assert_eq!(linspace.spacing, 5.0);
        assert_eq!(linspace.minmax(), (-180.0, 180.0), "dimensions not preserved!");
    }
}
