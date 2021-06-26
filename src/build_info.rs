// The file has been placed there by the build script.
#[doc(hidden)]
mod internal {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

#[doc = r#"The Continuous Integration platform detected during compilation."#]
#[allow(dead_code)]
pub const CI_PLATFORM: Option<&str> = internal::CI_PLATFORM;
#[doc = r#"The full version."#]
#[allow(dead_code)]
pub const PKG_VERSION: &str = internal::PKG_VERSION;
#[doc = r#"The major version."#]
#[allow(dead_code)]
pub const PKG_VERSION_MAJOR: &str = internal::PKG_VERSION_MAJOR;
#[doc = r#"The minor version."#]
#[allow(dead_code)]
pub const PKG_VERSION_MINOR: &str = internal::PKG_VERSION_MINOR;
#[doc = r#"The patch version."#]
#[allow(dead_code)]
pub const PKG_VERSION_PATCH: &str = internal::PKG_VERSION_PATCH;
#[doc = r#"The pre-release version."#]
#[allow(dead_code)]
pub const PKG_VERSION_PRE: &str = internal::PKG_VERSION_PRE;
#[doc = r#"A colon-separated list of authors."#]
#[allow(dead_code)]
pub const PKG_AUTHORS: &str = internal::PKG_AUTHORS;
#[doc = r#"The name of the package."#]
#[allow(dead_code)]
pub const PKG_NAME: &str = internal::PKG_NAME;
#[doc = r#"The description."#]
#[allow(dead_code)]
pub const PKG_DESCRIPTION: &str = internal::PKG_DESCRIPTION;
#[doc = r#"The homepage."#]
#[allow(dead_code)]
pub const PKG_HOMEPAGE: &str = internal::PKG_HOMEPAGE;
#[doc = r#"The license."#]
#[allow(dead_code)]
pub const PKG_LICENSE: &str = internal::PKG_LICENSE;
#[doc = r#"The source repository as advertised in Cargo.toml."#]
#[allow(dead_code)]
pub const PKG_REPOSITORY: &str = internal::PKG_REPOSITORY;
#[doc = r#"The target triple that was being compiled for."#]
#[allow(dead_code)]
pub const TARGET: &str = internal::TARGET;
#[doc = r#"The host triple of the rust compiler."#]
#[allow(dead_code)]
pub const HOST: &str = internal::HOST;
#[doc = r#"`release` for release builds, `debug` for other builds."#]
#[allow(dead_code)]
pub const PROFILE: &str = internal::PROFILE;
#[doc = r#"The compiler that cargo resolved to use."#]
#[allow(dead_code)]
pub const RUSTC: &str = internal::RUSTC;
#[doc = r#"The documentation generator that cargo resolved to use."#]
#[allow(dead_code)]
pub const RUSTDOC: &str = internal::RUSTDOC;
#[doc = r#"Value of OPT_LEVEL for the profile used during compilation."#]
#[allow(dead_code)]
pub const OPT_LEVEL: &str = internal::OPT_LEVEL;
#[doc = r#"The parallelism that was specified during compilation."#]
#[allow(dead_code)]
pub const NUM_JOBS: u32 = internal::NUM_JOBS;
#[doc = r#"Value of DEBUG for the profile used during compilation."#]
#[allow(dead_code)]
pub const DEBUG: bool = internal::DEBUG;
#[doc = r#"The features that were enabled during compilation."#]
#[allow(dead_code)]
pub const FEATURES: [&str; 0] = internal::FEATURES;
#[doc = r#"The features as a comma-separated string."#]
#[allow(dead_code)]
pub const FEATURES_STR: &str = internal::FEATURES_STR;
#[doc = r#"The output of `C:\Users\lolig\.cargo\bin\rustc.exe -V`"#]
#[allow(dead_code)]
pub const RUSTC_VERSION: &str = internal::RUSTC_VERSION;
#[doc = r#"The output of `rustdoc -V`"#]
#[allow(dead_code)]
pub const RUSTDOC_VERSION: &str = internal::RUSTDOC_VERSION;
#[doc = r#"An array of effective dependencies as documented by `Cargo.lock`."#]
#[allow(dead_code)]
pub const DEPENDENCIES: [(&str, &str); 216] = internal::DEPENDENCIES;
#[doc = r#"The effective dependencies as a comma-separated string."#]
#[allow(dead_code)]
pub const DEPENDENCIES_STR: &str = internal::DEPENDENCIES_STR;
#[doc = r#"The target architecture, given by `CARGO_CFG_TARGET_ARCH`."#]
#[allow(dead_code)]
pub const CFG_TARGET_ARCH: &str = internal::CFG_TARGET_ARCH;
#[doc = r#"The endianness, given by `CARGO_CFG_TARGET_ENDIAN`."#]
#[allow(dead_code)]
pub const CFG_ENDIAN: &str = internal::CFG_ENDIAN;
#[doc = r#"The toolchain-environment, given by `CARGO_CFG_TARGET_ENV`."#]
#[allow(dead_code)]
pub const CFG_ENV: &str = internal::CFG_ENV;
#[doc = r#"The OS-family, given by `CARGO_CFG_TARGET_FAMILY`."#]
#[allow(dead_code)]
pub const CFG_FAMILY: &str = internal::CFG_FAMILY;
#[doc = r#"The operating system, given by `CARGO_CFG_TARGET_OS`."#]
#[allow(dead_code)]
pub const CFG_OS: &str = internal::CFG_OS;
#[doc = r#"The pointer width, given by `CARGO_CFG_TARGET_POINTER_WIDTH`."#]
#[allow(dead_code)]
pub const CFG_POINTER_WIDTH: &str = internal::CFG_POINTER_WIDTH;
