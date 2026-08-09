#[cfg(target_os = "macos")]
pub fn platform_name() -> &'static str {
    "darwin"
}

#[cfg(target_os = "linux")]
pub fn platform_name() -> &'static str {
    "linux"
}

/// Adoptium's `os` query param, which uses different values than
/// `platform_name()` (e.g. `mac` instead of `darwin`).
#[cfg(target_os = "macos")]
pub fn os() -> &'static str {
    "mac"
}

#[cfg(target_os = "linux")]
pub fn os() -> &'static str {
    "linux"
}

#[cfg(target_os = "windows")]
pub fn os() -> &'static str {
    "windows"
}

#[cfg(all(
    target_pointer_width = "32",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> &'static str {
    "armv7l"
}

#[cfg(all(
    target_pointer_width = "32",
    not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> &'static str {
    "x86"
}

#[cfg(all(
    target_pointer_width = "64",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub fn platform_arch() -> &'static str {
    "arm64"
}

#[cfg(all(
    target_pointer_width = "64",
    not(any(target_arch = "arm", target_arch = "aarch64"))
))]
pub fn platform_arch() -> &'static str {
    "x64"
}
