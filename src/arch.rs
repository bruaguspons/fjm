use crate::version::Version;
use clap::ValueEnum;

#[derive(Clone, Copy, PartialEq, Eq, Debug, ValueEnum)]
pub enum Arch {
    X86,
    X64,
    X64Musl,
    X64Glibc217,
    Arm64,
    Armv7l,
    Ppc64le,
    Ppc64,
    S390x,
}

impl Arch {
    /// Maps this `Arch` to Adoptium's `(os, architecture)` query params.
    ///
    /// `X64Musl` and `X64Glibc217` always resolve to a fixed `os` regardless
    /// of the running platform, since they encode a libc flavor rather than
    /// an OS. Adoptium doesn't publish 32-bit x86 JDK builds, so `X86`
    /// returns an explicit error instead of silently mapping to an asset
    /// that will never exist.
    pub fn adoptium_os_arch(self) -> Result<(&'static str, &'static str), ArchError> {
        let os = crate::system_info::os();
        match self {
            Arch::X86 => Err(ArchError::new(format!(
                "{self} is not supported: Adoptium doesn't publish 32-bit x86 JDK builds"
            ))),
            Arch::X64 => Ok((os, "x64")),
            Arch::X64Musl => Ok(("alpine-linux", "x64")),
            // Adoptium has no glibc-version axis, so this loses the "217"
            // precision and maps to the same asset as plain X64 on Linux.
            Arch::X64Glibc217 => Ok(("linux", "x64")),
            Arch::Arm64 => Ok((os, "aarch64")),
            Arch::Armv7l => Ok((os, "arm")),
            Arch::Ppc64le => Ok((os, "ppc64le")),
            Arch::Ppc64 => Ok((os, "ppc64")),
            Arch::S390x => Ok((os, "s390x")),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Arch::X86 => "x86",
            Arch::X64 => "x64",
            Arch::X64Musl => "x64-musl",
            Arch::X64Glibc217 => "x64-glibc-217",
            Arch::Arm64 => "arm64",
            Arch::Armv7l => "armv7l",
            Arch::Ppc64le => "ppc64le",
            Arch::Ppc64 => "ppc64",
            Arch::S390x => "s390x",
        }
    }
}

#[cfg(unix)]
/// handle common case: Apple Silicon / JDK < 16 (inherited from the original scaffold; kept as-is, not Adoptium-specific)
pub fn get_safe_arch(arch: Arch, version: &Version) -> Arch {
    use crate::system_info::{platform_arch, platform_name};

    match (platform_name(), platform_arch(), version) {
        ("darwin", "arm64", Version::Semver(v)) if v.major < 16 => Arch::X64,
        _ => arch,
    }
}

#[cfg(windows)]
/// handle common case: Apple Silicon / JDK < 16 (inherited from the original scaffold; kept as-is, not Adoptium-specific)
pub fn get_safe_arch(arch: Arch, _version: &Version) -> Arch {
    arch
}

impl Default for Arch {
    fn default() -> Arch {
        match crate::system_info::platform_arch().parse() {
            Ok(arch) => arch,
            Err(e) => panic!("{}", e.details),
        }
    }
}

impl std::str::FromStr for Arch {
    type Err = ArchError;
    fn from_str(s: &str) -> Result<Arch, Self::Err> {
        match s {
            "x86" => Ok(Arch::X86),
            "x64" => Ok(Arch::X64),
            "x64-musl" => Ok(Arch::X64Musl),
            "x64-glibc-217" => Ok(Arch::X64Glibc217),
            "arm64" => Ok(Arch::Arm64),
            "armv7l" => Ok(Arch::Armv7l),
            "ppc64le" => Ok(Arch::Ppc64le),
            "ppc64" => Ok(Arch::Ppc64),
            "s390x" => Ok(Arch::S390x),
            unknown => Err(ArchError::new(format!("Unknown Arch: {unknown}"))),
        }
    }
}

impl std::fmt::Display for Arch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug)]
pub struct ArchError {
    details: String,
}

impl ArchError {
    fn new(msg: String) -> ArchError {
        ArchError { details: msg }
    }
}

impl std::fmt::Display for ArchError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.details)
    }
}

impl std::error::Error for ArchError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_musl_maps_to_alpine_linux() {
        assert_eq!(
            Arch::X64Musl.adoptium_os_arch().unwrap(),
            ("alpine-linux", "x64")
        );
    }

    #[test]
    fn test_glibc217_maps_to_plain_linux_x64() {
        assert_eq!(
            Arch::X64Glibc217.adoptium_os_arch().unwrap(),
            ("linux", "x64")
        );
    }

    #[test]
    fn test_x86_is_unsupported() {
        assert!(Arch::X86.adoptium_os_arch().is_err());
    }

    #[test]
    fn test_arm64_maps_to_aarch64() {
        let (_, arch) = Arch::Arm64.adoptium_os_arch().unwrap();
        assert_eq!(arch, "aarch64");
    }

    #[test]
    fn test_armv7l_maps_to_arm() {
        let (_, arch) = Arch::Armv7l.adoptium_os_arch().unwrap();
        assert_eq!(arch, "arm");
    }

    #[test]
    fn test_ppc64le_maps_correctly() {
        let (_, arch) = Arch::Ppc64le.adoptium_os_arch().unwrap();
        assert_eq!(arch, "ppc64le");
    }

    #[test]
    fn test_ppc64_maps_correctly() {
        let (_, arch) = Arch::Ppc64.adoptium_os_arch().unwrap();
        assert_eq!(arch, "ppc64");
    }

    #[test]
    fn test_s390x_maps_correctly() {
        let (_, arch) = Arch::S390x.adoptium_os_arch().unwrap();
        assert_eq!(arch, "s390x");
    }

    #[test]
    fn test_x64_uses_platform_os() {
        let (os, arch) = Arch::X64.adoptium_os_arch().unwrap();
        assert_eq!(arch, "x64");
        assert_eq!(os, crate::system_info::os());
    }
}
