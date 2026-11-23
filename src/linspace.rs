use crate::{
    error::{Error, ParsingError},
    quantized::Quantized,
};

/// Quantized [Linspace] iterator, returning each grid node exactly
/// (integer arithmetic), from [Linspace::start] to [Linspace::end] (included),
/// whatever the direction of the axis (increasing or decreasing).
#[derive(Debug, Copy, Clone, Default, PartialEq, PartialOrd)]
pub struct QuantizedLinspace {
    /// Index of the next point
    index: i64,

    /// Total number of points
    num_points: i64,

    /// Common exponent
    exponent: i8,

    /// First point (included)
    pub start: Quantized,

    /// Last point (included)
    pub end: Quantized,

    /// Signed spacing
    pub spacing: Quantized,
}

impl QuantizedLinspace {
    /// Returns the i-th point of this [QuantizedLinspace] (whatever the
    /// current state of the iterator), if it exists.
    pub fn nth_point(&self, i: usize) -> Option<Quantized> {
        let i = i as i64;

        if i >= self.num_points {
            return None;
        }

        Some(Quantized {
            exponent: self.exponent,
            value: self.start.value + i * self.spacing.value,
        })
    }
}

impl Iterator for QuantizedLinspace {
    type Item = Quantized;

    fn next(&mut self) -> Option<Self::Item> {
        let value = self.nth_point(self.index as usize)?;
        self.index += 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = (self.num_points - self.index).max(0) as usize;
        (remaining, Some(remaining))
    }
}

impl ExactSizeIterator for QuantizedLinspace {}

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
    /// Tolerance (in the unit of this axis) used when comparing
    /// coordinates against grid nodes.
    const TOLERANCE: f64 = 1.0E-9;

    /// Returns the exponent that allows exact (integer) representation
    /// of all the nodes of this [Linspace].
    fn common_exponent(&self) -> i8 {
        Quantized::find_exponent(self.start)
            .max(Quantized::find_exponent(self.end))
            .max(Quantized::find_exponent(self.spacing))
    }

    /// Quantizes this [Linspace] returning a [QuantizedLinspace] iterator,
    /// that returns every grid node exactly, in the order of this axis.
    /// This works for both increasing and decreasing axes
    /// (for example, IONEX latitudes are usually described North to South).
    pub fn quantize(&self) -> QuantizedLinspace {
        let exponent = self.common_exponent();

        QuantizedLinspace {
            index: 0,
            exponent,
            num_points: self.num_points() as i64,
            start: Quantized::new(self.start, exponent),
            end: Quantized::new(self.end, exponent),
            spacing: Quantized::new(self.spacing, exponent),
        }
    }

    /// Returns the total number of points (grid nodes) described by this [Linspace],
    /// both ends included. A single point [Linspace] contains one point.
    pub fn num_points(&self) -> usize {
        if self.spacing == 0.0 || self.start == self.end {
            return 1;
        }

        let n = ((self.end - self.start) / self.spacing).round();

        if n < 0.0 { 1 } else { n as usize + 1 }
    }

    /// Returns true if this value lies within this [Linspace] (ends included).
    pub fn contains(&self, value: f64) -> bool {
        let (min, max) = self.minmax();
        value >= min - Self::TOLERANCE && value <= max + Self::TOLERANCE
    }

    /// Returns the two consecutive grid nodes surrounding (wrapping) this value,
    /// in the order of this axis (first node, then second node),
    /// as exact [Quantized] values.
    /// When the value lies exactly on a node, it is either the first or the second node
    /// of the returned pair. Returns None when the value is outside this [Linspace].
    /// For a single point [Linspace], both nodes are that point.
    pub(crate) fn quantized_bracket(&self, value: f64) -> Option<(Quantized, Quantized)> {
        if !self.contains(value) {
            return None;
        }

        let quantized = self.quantize();
        let num_points = self.num_points();

        if num_points < 2 {
            let point = quantized.nth_point(0)?;
            return Some((point, point));
        }

        let k = ((value - self.start) / self.spacing).floor();
        let k = (k.max(0.0) as usize).min(num_points - 2);

        Some((quantized.nth_point(k)?, quantized.nth_point(k + 1)?))
    }

    /// Returns the two consecutive grid nodes surrounding (wrapping) this value,
    /// in the order of this axis. Returns None when the value is outside this [Linspace].
    /// ```
    /// use ionex::prelude::Linspace;
    ///
    /// // IONEX latitudes are usually described North to South
    /// let latitudes = Linspace::new(87.5, -87.5, -2.5).unwrap();
    /// assert_eq!(latitudes.bracket(44.0), Some((45.0, 42.5)));
    /// assert_eq!(latitudes.bracket(87.5), Some((87.5, 85.0)));
    /// assert_eq!(latitudes.bracket(-87.5), Some((-85.0, -87.5)));
    /// assert_eq!(latitudes.bracket(89.0), None);
    ///
    /// let longitudes = Linspace::new(-180.0, 180.0, 5.0).unwrap();
    /// assert_eq!(longitudes.bracket(2.0), Some((0.0, 5.0)));
    /// ```
    pub fn bracket(&self, value: f64) -> Option<(f64, f64)> {
        let (a, b) = self.quantized_bracket(value)?;
        Some((a.real_value(), b.real_value()))
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

    /// Stretch this mutable [Linspace] by a finite positive number (possibly fractional),
    /// while preserving the initial grid quantization (point spacing).
    /// This only modifies the [Linspace] dimensions.
    /// To modify the sampling, use [Self::resample_mut].
    pub fn stretch_mut(&mut self, factor: f64) -> Result<(), Error> {
        if !factor.is_normal() {
            return Err(Error::InvalidStretchFactor);
        }
        self.start *= factor;
        self.end *= factor;
        Ok(())
    }

    /// Stretch this [Linspace] to a new [Linspace] definition.
    /// The streetching factor must be a finite positive (possibly fractional) number.
    /// This preserves the initial grid quantization (point spacing) and
    /// only affects the [Linspace] dimensions. Use [Self::resampled] to resample
    /// the [Linspace].
    pub fn stretched(&self, factor: f64) -> Result<Self, Error> {
        let mut s = self.clone();
        s.stretch_mut(factor)?;
        Ok(s)
    }

    /// Resample this mutable [Linspace] so the point spacing is modified.
    /// This is a multiplicative stretching factor, which must be a finite positive number (possibly fractional).
    /// This does not modify the [Linspace] dimensions, use [Self::stretch_mut] to modify dimensions.
    pub fn resample_mut(&mut self, factor: f64) -> Result<(), Error> {
        if !factor.is_normal() {
            return Err(Error::InvalidStretchFactor);
        }
        self.spacing *= factor;
        Ok(())
    }

    /// Resample this [Linspace] to a new [Linspace] definition, modifying the point
    /// spacing but preserving the initial dimensions. The resampling factor
    /// must be a finite positive, possibly fractional number. This does not modify the dimensions,
    /// use [Self::stretched] to modify the dimensions.
    pub fn resampled(&self, factor: f64) -> Result<Self, Error> {
        let mut s = self.clone();
        s.resample_mut(factor)?;
        Ok(s)
    }

    /// Builds a new Linear space, from start to end (both included), with this spacing.
    /// The spacing must be signed accordingly (negative for decreasing spaces),
    /// and the width must be a multiple of the spacing.
    /// A null spacing describes a single point (start = end).
    /// ```
    /// use ionex::prelude::Linspace;
    ///
    /// assert!(Linspace::new(87.5, -87.5, -2.5).is_ok());
    /// assert!(Linspace::new(-180.0, 180.0, 5.0).is_ok());
    /// assert!(Linspace::new(350.0, 350.0, 0.0).is_ok());
    ///
    /// // regional map
    /// assert!(Linspace::new(87.5, 80.0, -2.5).is_ok());
    ///
    /// // width is not a multiple of the spacing
    /// assert!(Linspace::new(0.0, 10.0, 3.0).is_err());
    ///
    /// // incorrect direction
    /// assert!(Linspace::new(87.5, -87.5, 2.5).is_err());
    /// ```
    pub fn new(start: f64, end: f64, spacing: f64) -> Result<Self, ParsingError> {
        if spacing == 0.0 {
            if start == end {
                return Ok(Self {
                    start,
                    end,
                    spacing,
                });
            } else {
                return Err(ParsingError::InvalidGridDefinition);
            }
        }

        let n = (end - start) / spacing;

        if !n.is_finite() || n < 0.0 || (n - n.round()).abs() > 1.0E-6 {
            return Err(ParsingError::InvalidGridDefinition);
        }

        Ok(Self {
            start,
            end,
            spacing,
        })
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
    fn quantized_iteration() {
        // decreasing axis (usual IONEX latitude)
        let linspace = Linspace::new(87.5, -87.5, -2.5).unwrap();
        let points = linspace
            .quantize()
            .map(|q| q.real_value())
            .collect::<Vec<_>>();
        assert_eq!(points.len(), 71);
        assert_eq!(linspace.num_points(), 71);
        assert_eq!(points[0], 87.5);
        assert_eq!(points[1], 85.0);
        assert_eq!(points[70], -87.5);

        // increasing axis (usual IONEX longitude)
        let linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();
        let points = linspace
            .quantize()
            .map(|q| q.real_value())
            .collect::<Vec<_>>();
        assert_eq!(points.len(), 73);
        assert_eq!(points[0], -180.0);
        assert_eq!(points[36], 0.0);
        assert_eq!(points[72], 180.0);

        // single point (2D altitude)
        let linspace = Linspace::new(350.0, 350.0, 0.0).unwrap();
        let points = linspace
            .quantize()
            .map(|q| q.real_value())
            .collect::<Vec<_>>();
        assert_eq!(points, vec![350.0]);

        // mixed exponents
        let linspace = Linspace::from((-1.25, 1.25, 0.5));
        let points = linspace
            .quantize()
            .map(|q| q.real_value())
            .collect::<Vec<_>>();
        assert_eq!(points, vec![-1.25, -0.75, -0.25, 0.25, 0.75, 1.25]);

        // exact size
        assert_eq!(linspace.quantize().len(), 6);
    }

    #[test]
    fn brackets() {
        let linspace = Linspace::new(87.5, -87.5, -2.5).unwrap();
        assert_eq!(linspace.bracket(0.0), Some((0.0, -2.5)));
        assert_eq!(linspace.bracket(1.0), Some((2.5, 0.0)));
        assert_eq!(linspace.bracket(-87.5), Some((-85.0, -87.5)));
        assert_eq!(linspace.bracket(-88.0), None);

        let linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();
        assert_eq!(linspace.bracket(-180.0), Some((-180.0, -175.0)));
        assert_eq!(linspace.bracket(179.9), Some((175.0, 180.0)));
        assert_eq!(linspace.bracket(180.0), Some((175.0, 180.0)));
        assert_eq!(linspace.bracket(181.0), None);

        let linspace = Linspace::new(350.0, 350.0, 0.0).unwrap();
        assert_eq!(linspace.bracket(350.0), Some((350.0, 350.0)));
        assert_eq!(linspace.bracket(450.0), None);

        let linspace = Linspace::new(100.0, 1000.0, 50.0).unwrap();
        assert_eq!(linspace.bracket(425.0), Some((400.0, 450.0)));
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
        assert_eq!(
            linspace.spacing, 5.0,
            "linspace quantization not preserved!"
        );

        linspace.stretch_mut(0.75).unwrap();
        assert_eq!(linspace.minmax(), (-67.5, 67.5));
        assert_eq!(
            linspace.spacing, 5.0,
            "linspace quantization not preserved!"
        );

        linspace.stretch_mut(0.5).unwrap();
        assert_eq!(linspace.minmax(), (-33.75, 33.75));
        assert_eq!(
            linspace.spacing, 5.0,
            "linspace quantization not preserved!"
        );

        linspace.stretch_mut(2.0).unwrap();
        assert_eq!(linspace.minmax(), (-67.5, 67.5));
        assert_eq!(
            linspace.spacing, 5.0,
            "linspace quantization not preserved!"
        );
    }

    #[test]
    fn linspace_resampling() {
        let mut linspace = Linspace::new(-180.0, 180.0, 5.0).unwrap();

        linspace.resample_mut(0.5).unwrap();
        assert_eq!(linspace.spacing, 2.5);
        assert_eq!(
            linspace.minmax(),
            (-180.0, 180.0),
            "dimensions not preserved!"
        );

        linspace.resample_mut(0.5).unwrap();
        assert_eq!(linspace.spacing, 1.25);
        assert_eq!(
            linspace.minmax(),
            (-180.0, 180.0),
            "dimensions not preserved!"
        );

        linspace.resample_mut(2.0).unwrap();
        assert_eq!(linspace.spacing, 2.5);
        assert_eq!(
            linspace.minmax(),
            (-180.0, 180.0),
            "dimensions not preserved!"
        );

        linspace.resample_mut(2.0).unwrap();
        assert_eq!(linspace.spacing, 5.0);
        assert_eq!(
            linspace.minmax(),
            (-180.0, 180.0),
            "dimensions not preserved!"
        );
    }
}
