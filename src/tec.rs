use crate::quantized::Quantized;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Total Electron Content (TEC) estimate
#[derive(Debug, Default, Copy, Clone, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TEC {
    /// TEC quantized in TEcu
    pub(crate) tecu: Quantized,

    /// RMS (TEC)
    pub(crate) rms: Option<Quantized>,

    /// Altitude offset for complex 3D height map
    pub(crate) height: Option<Quantized>,
}

impl std::ops::Mul<f64> for TEC {
    type Output = TEC;

    fn mul(self, rhs: f64) -> Self::Output {
        let tecu = self.tecu() * rhs;

        let mut tec = TEC::from_tecu(tecu);
        tec.rms = self.rms.clone();
        tec.height = self.height.clone();
        tec
    }
}

impl std::ops::MulAssign<f64> for TEC {
    fn mul_assign(&mut self, rhs: f64) {
        let tecu = self.tecu() * rhs;
        *self = self.with_tecu(tecu);
    }
}

impl std::ops::Div<f64> for TEC {
    type Output = TEC;

    fn div(self, rhs: f64) -> Self::Output {
        let tecu = self.tecu() / rhs;

        let mut tec = TEC::from_tecu(tecu);
        tec.rms = self.rms.clone();
        tec.height = self.height.clone();
        tec
    }
}

impl std::ops::DivAssign<f64> for TEC {
    fn div_assign(&mut self, rhs: f64) {
        let tecu = self.tecu() / rhs;
        *self = self.with_tecu(tecu);
    }
}

/// One TEC Unit, in electrons per squared meter.
pub const TECU_M2: f64 = 1.0E16;

/// First order ionospheric refraction constant (in m^3.s^-2): e²/(8π²ε0.me).
/// The first order ionospheric (group) delay is 40.3 TEC / f².
pub const IONOSPHERIC_CONSTANT: f64 = 40.308193;

impl TEC {
    /// Builds new [TEC] from TEC estimate expressed in TECu (=10^16 m-2)
    pub fn from_tecu(tecu: f64) -> Self {
        Self {
            rms: None,
            height: None,
            tecu: Quantized::auto_scaled(tecu),
        }
    }

    /// Updates this [TEC] with new TECu value
    pub fn with_tecu(mut self, tecu: f64) -> Self {
        self.tecu = Quantized::auto_scaled(tecu);
        self
    }

    /// Builds new [TEC] from raw TEC estimate in m^-2
    pub fn from_tec_m2(tec: f64) -> Self {
        let tecu = tec / TECU_M2;
        Self {
            rms: None,
            height: None,
            tecu: Quantized::auto_scaled(tecu),
        }
    }

    /// Updates this [TEC] with new TEC value in m^-2
    pub fn with_tec_m2(mut self, tec: f64) -> Self {
        let tecu = tec / TECU_M2;
        self.tecu = Quantized::auto_scaled(tecu);
        self
    }

    /// Copyes and returns [Self] with update TEC RMS.
    pub fn with_rms(mut self, rms: f64) -> Self {
        self.rms = Some(Quantized::auto_scaled(rms));
        self
    }

    /// Builds new [TEC] from TEC quantization in TECu
    pub(crate) fn from_quantized(tecu: i64, exponent: i8) -> Self {
        // IONEX stores quantized TEC as i=10*-k TECu
        Self {
            rms: None,
            height: None,
            tecu: Quantized {
                value: tecu,
                exponent: -exponent,
            },
        }
    }

    /// Updates [TEC] estimate from its quantized IONEX representation,
    /// preserving the other fields.
    pub(crate) fn set_quantized_tecu(&mut self, tecu: i64, exponent: i8) {
        self.tecu = Quantized {
            value: tecu,
            exponent: -exponent,
        };
    }

    /// Updates [TEC] Root Mean Square
    pub(crate) fn set_quantized_root_mean_square(&mut self, rms: i64, exponent: i8) {
        self.rms = Some(Quantized {
            exponent: -exponent,
            value: rms,
        });
    }

    /// Returns Total Electron Content estimate, in TECu (=10^16 m-2)
    pub fn tecu(&self) -> f64 {
        self.tecu.real_value()
    }

    /// Returns Total Electron Content estimate, in m-2
    pub fn tec(&self) -> f64 {
        self.tecu() * TECU_M2
    }

    /// Linear interpolation between self (weight 1-w) and rhs (weight w).
    /// The RMS is interpolated as well when defined on both sides.
    pub(crate) fn lerp(&self, rhs: &Self, w: f64) -> Self {
        let mut tec = Self::from_tecu((1.0 - w) * self.tecu() + w * rhs.tecu());

        if let (Some(rms0), Some(rms1)) = (self.root_mean_square(), rhs.root_mean_square()) {
            tec.rms = Some(Quantized::auto_scaled((1.0 - w) * rms0 + w * rms1));
        }

        tec
    }

    /// Returns this [TEC] (and its RMS, if defined) scaled by this factor.
    /// For example, to convert vertical TEC to slant TEC using a mapping function.
    pub fn scaled(&self, factor: f64) -> Self {
        let mut tec = Self::from_tecu(self.tecu() * factor);

        if let Some(rms) = self.root_mean_square() {
            tec.rms = Some(Quantized::auto_scaled(rms * factor));
        }

        tec.height = self.height;
        tec
    }

    /// Returns the first order ionospheric delay (in meters) experienced by a signal
    /// of this frequency (in Hz) when crossing this [TEC] (which should then
    /// be a slant TEC): d = 40.3 TEC / f².
    /// This is the group delay (code), the carrier phase is advanced by the same amount.
    ///
    /// ```
    /// use ionex::prelude::TEC;
    ///
    /// // 1 TECu is about 16 cm on GPS L1
    /// let tec = TEC::from_tecu(1.0);
    /// let delay_m = tec.ionospheric_delay_m(1575.42E6);
    /// assert!((delay_m - 0.1624).abs() < 1E-4);
    /// ```
    pub fn ionospheric_delay_m(&self, frequency_hz: f64) -> f64 {
        IONOSPHERIC_CONSTANT * self.tec() / frequency_hz.powi(2)
    }

    /// Returns TEC Root Mean Square (if determined).
    pub fn root_mean_square(&self) -> Option<f64> {
        let rms = self.rms?;
        Some(rms.real_value())
    }
}

#[cfg(test)]
mod test {
    use super::TEC;

    #[test]
    fn quantized_tec() {
        let tec = TEC::from_quantized(30, -1);
        assert_eq!(tec.tecu(), 3.0);
        assert_eq!(tec.tec(), 3.0E16);

        let tec = TEC::from_quantized(30, -2);
        assert_eq!(tec.tecu(), 0.3);
        assert_eq!(tec.tec(), 0.3E16);

        // 1 TECu = 1E16 el/m^2
        let tec = TEC::from_tec_m2(1.0E16);
        assert_eq!(tec.tecu(), 1.0);
        assert_eq!(tec.tec(), 1.0E16);
        assert_eq!(tec, TEC::from_tecu(1.0));

        let tec = TEC::from_tec_m2(3.5E16);
        assert_eq!(tec.tecu(), 3.5);
        assert_eq!(tec.tec(), 3.5E16);
        assert_eq!(tec, TEC::from_tecu(3.5));

        let tec = TEC::from_tec_m2(190355078157525800.0);
        assert!((tec.tecu() - 19.03550781575258).abs() < 1.0E-9);
        assert!((tec.tec() - 1.903550781575258e17).abs() < 1.0E3);
    }

    #[test]
    fn tec_arithmetics() {
        let mut tec = TEC::from_tecu(9.0);
        assert_eq!(tec.tecu(), 9.0);

        assert_eq!((tec * 2.0).tecu(), 18.0);
        assert_eq!((tec / 2.0).tecu(), 4.5);

        tec *= 2.0;
        assert_eq!(tec.tecu(), 18.0);

        tec /= 2.0;
        assert_eq!(tec.tecu(), 9.0);

        tec /= 2.0;
        assert_eq!(tec.tecu(), 4.5);
    }
}
