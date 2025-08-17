pub struct IonosphereParameters {
    /// Amplitude of the ionospheric delay (seconds)
    pub amplitude_s: f64,

    /// Period of the ionospheric delay (seconds)
    pub period_s: f64,

    /// Phase of the ionospheric delay (rad)
    pub phase_rad: f64,

    /// Slant factor
    pub slant: f64,
}
