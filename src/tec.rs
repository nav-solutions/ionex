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
        let tecu = tec / 10.0E16;
        Self {
            rms: None,
            height: None,
            tecu: Quantized::auto_scaled(tecu),
        }
    }

    /// Updates this [TEC] with new TEC value in m^-2
    pub fn with_tec_m2(mut self, tec: f64) -> Self {
        let tecu = tec / 10.0E16;
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

    /// Updates [TEC] Root Mean Square
    pub(crate) fn set_quantized_root_mean_square(&mut self, rms: i64, exponent: i8) {
        self.rms = Some(Quantized {
            exponent: -exponent,
            value: rms,
        });
    }

    /// Returns Total Electron Content estimate, in TECu (=10^-16 m-2)
    pub fn tecu(&self) -> f64 {
        self.tecu.real_value()
    }

    /// Returns Total Electron Content estimate, in m-2
    pub fn tec(&self) -> f64 {
        self.tecu() * 10.0E16
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
        assert_eq!(tec.tec(), 3.0 * 10E16);

        let tec = TEC::from_quantized(30, -2);
        assert_eq!(tec.tecu(), 0.3);
        assert_eq!(tec.tec(), 0.3 * 10E16);

        let tec = TEC::from_tec_m2(1.0 * 10E16);
        assert_eq!(tec.tecu(), 1.0);
        assert_eq!(tec.tec(), 1.0 * 10E16);
        assert_eq!(tec, TEC::from_tecu(1.0));

        let tec = TEC::from_tec_m2(3.5 * 10E16);
        assert_eq!(tec.tecu(), 3.5);
        assert_eq!(tec.tec(), 3.5 * 10E16);
        assert_eq!(tec, TEC::from_tecu(3.5));

        let tec = TEC::from_tec_m2(190355078157525800.0);
        assert_eq!(tec.tec(), 1.903550781575258e17);
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
