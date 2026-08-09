use crate::remote_node_index::IndexedJdkVersion;
use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone)]
pub enum LtsType {
    /// lts-*, lts/*, lts-latest
    Latest,
    /// lts-17, lts/21 — JDK has no codenames, only numeric LTS majors
    Major(u32),
}

impl FromStr for LtsType {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "*" || s == "latest" {
            Ok(Self::Latest)
        } else {
            s.parse::<u32>().map(Self::Major)
        }
    }
}

impl Display for LtsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Latest => f.write_str("latest"),
            Self::Major(major) => write!(f, "{major}"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("JDK {major} is not an LTS release")]
    NotAnLtsRelease { major: u32 },
}

impl LtsType {
    /// Resolves this LTS selector against a listing.
    ///
    /// `available_lts_releases`'s highest major is always the most recent LTS,
    /// so `Latest` doesn't need a separate "most recent" flag on each entry.
    pub fn pick_latest<'vec>(
        &self,
        versions: &'vec [IndexedJdkVersion],
    ) -> Result<Option<&'vec IndexedJdkVersion>, Error> {
        match self {
            Self::Latest => Ok(versions.iter().filter(|v| v.lts).max_by_key(|v| v.major)),
            Self::Major(major) => match versions.iter().find(|v| v.major == *major) {
                Some(v) if v.lts => Ok(Some(v)),
                Some(_) => Err(Error::NotAnLtsRelease { major: *major }),
                None => Ok(None),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<IndexedJdkVersion> {
        vec![
            IndexedJdkVersion {
                major: 8,
                lts: true,
            },
            IndexedJdkVersion {
                major: 11,
                lts: true,
            },
            IndexedJdkVersion {
                major: 20,
                lts: false,
            },
            IndexedJdkVersion {
                major: 21,
                lts: true,
            },
        ]
    }

    #[test]
    fn test_parse_latest() {
        assert_eq!("*".parse::<LtsType>(), Ok(LtsType::Latest));
        assert_eq!("latest".parse::<LtsType>(), Ok(LtsType::Latest));
    }

    #[test]
    fn test_parse_numeric_major() {
        assert_eq!("17".parse::<LtsType>(), Ok(LtsType::Major(17)));
    }

    #[test]
    fn test_parse_non_numeric_is_error() {
        assert!("erbium".parse::<LtsType>().is_err());
    }

    #[test]
    fn test_pick_latest_picks_highest_lts_major() {
        let versions = fixture();
        let picked = LtsType::Latest.pick_latest(&versions).unwrap().unwrap();
        assert_eq!(picked.major, 21);
    }

    #[test]
    fn test_pick_specific_lts_major() {
        let versions = fixture();
        let picked = LtsType::Major(11).pick_latest(&versions).unwrap().unwrap();
        assert_eq!(picked.major, 11);
    }

    #[test]
    fn test_pick_major_not_lts_is_error() {
        let versions = fixture();
        assert!(matches!(
            LtsType::Major(20).pick_latest(&versions),
            Err(Error::NotAnLtsRelease { major: 20 })
        ));
    }

    #[test]
    fn test_pick_major_missing_entirely_is_none() {
        let versions = fixture();
        assert_eq!(LtsType::Major(99).pick_latest(&versions).unwrap(), None);
    }
}
