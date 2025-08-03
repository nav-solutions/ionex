//! IONEX file revision
use crate::prelude::ParsingError;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// [Version] is used to describe the file revision.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Version {
    /// Version major number
    pub major: u8,

    /// Version minor number
    pub minor: u8,
}

impl Default for Version {
    /// Returns a default v1.0 [Version]
    fn default() -> Self {
        Version::new(1, 0)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

impl std::ops::Add<u8> for Version {
    type Output = Version;
    fn add(self, major: u8) -> Version {
        Version {
            major: self.major + major,
            minor: self.minor,
        }
    }
}

impl std::ops::AddAssign<u8> for Version {
    fn add_assign(&mut self, major: u8) {
        self.major += major;
    }
}

impl std::ops::Sub<u8> for Version {
    type Output = Version;
    fn sub(self, major: u8) -> Version {
        if major >= self.major {
            // clamp @ V1.X
            Version {
                major: 1,
                minor: self.minor,
            }
        } else {
            Version {
                major: self.major - major,
                minor: self.minor,
            }
        }
    }
}

impl std::ops::SubAssign<u8> for Version {
    fn sub_assign(&mut self, major: u8) {
        if major >= self.major {
            // clamp @ V1.X
            self.major = 1;
        } else {
            self.major -= major;
        }
    }
}

impl From<Version> for (u8, u8) {
    fn from(v: Version) -> Self {
        (v.major, v.minor)
    }
}

impl std::str::FromStr for Version {
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut digits = s.split('.');

        match s.contains('.') {
            true => {
                let major = digits.next().ok_or(ParsingError::VersionParsing)?;

                let minor = digits.next().ok_or(ParsingError::VersionParsing)?;

                let major = major.parse::<u8>().or(Err(ParsingError::VersionParsing))?;
                let minor = minor.parse::<u8>().or(Err(ParsingError::VersionParsing))?;

                Ok(Self { major, minor })
            },
            false => {
                let major = digits.next().ok_or(ParsingError::VersionParsing)?;

                let major = major.parse::<u8>().or(Err(ParsingError::VersionParsing))?;

                Ok(Self { major, minor: 0 })
            },
        }
    }
}

impl Version {
    /// Builds a new [Version]
    pub fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn version() {
        for (version_str, major, minor) in
            [("1.0", 1, 0), ("1.2", 1, 2), ("2.0", 2, 0), ("3.2", 3, 2)]
        {
            let version = Version::from_str(version_str).unwrap_or_else(|e| {
                panic!("Failed to parse version from \"{}\"", version_str);
            });

            assert_eq!(version.major, major);
            assert_eq!(version.minor, minor);

            let formatted = version.to_string();
            assert_eq!(formatted, version_str);
        }
    }

    #[test]
    fn version_comparison() {
        let v_a = Version::from_str("1.2").unwrap();
        let v_b = Version::from_str("3.02").unwrap();
        assert!(v_b > v_a);
        assert!(v_b != v_a);
    }

    #[test]
    fn version_arithmetics() {
        let version = Version::new(3, 2);
        assert_eq!(version + 1, Version::new(4, 2));
        assert_eq!(version + 2, Version::new(5, 2));
        assert_eq!(version - 2, Version::new(1, 2));
        assert_eq!(version - 3, Version::new(1, 2)); // clamped

        let (maj, min): (u8, u8) = version.into();
        assert_eq!(maj, 3);
        assert_eq!(min, 2);
    }
}
