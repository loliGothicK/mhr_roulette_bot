fn main() {
    let mut opts = built::Options::default();
    opts.set_dependencies(true);

    let src = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = std::path::Path::new(&std::env::var("OUT_DIR").unwrap()).join("built.rs");
    built::write_built_file_with_opts(&opts, src.as_ref(), &dst)
        .expect("Failed to acquire build-time information");
}

/// The time this crate was built
#[cfg(feature = "chrono")]
fn built_time() -> built::chrono::DateTime<built::chrono::Local> {
    built::util::strptime(built_info::BUILT_TIME_UTC).with_timezone(&built::chrono::offset::Local)
}
