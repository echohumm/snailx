use {
    rustc_version::version,
    std::{
        env::current_dir,
        fs::{read_to_string, write}
    }
};

// as jank as this may seem, because rust is stupid, we need this to make it compile on older
// versions of rustc. cfg_attr/rustversion do not work because the attribute potentially being
// configured out is evaluated before the configuring out, so it will still error for unrecognized
// attrs (like unsafe).
fn main() {
    let v = version().expect("failed to get rustc version for build");
    let cwd = current_dir().expect("failed to get current directory");
    let raw_src = cwd.join("src");
    let src_path = cwd.join("src/direct.rs.src");
    let dst_path = cwd.join("src/direct.rs");

    // Let Cargo know when to rerun the build script for the Rust source generation
    println!("cargo:rerun-if-changed={}", raw_src.display());
    println!("cargo:rerun-if-env-changed=RUSTC");

    let src_contents = read_to_string(&src_path).expect("failed to read source file");

    let generated = if v.minor > 81 {
        src_contents
    } else {
        src_contents.as_str().replace(
            "#[unsafe(link_section = \".init_array.00099\")]",
            "#[link_section = \".init_array.00099\"]"
        )
    };

    // Only write if the destination differs to avoid unnecessary rebuilds and editor churn
    let need_write = match read_to_string(&dst_path) {
        Ok(existing) => existing != generated,
        Err(_) => true
    };

    if need_write {
        write(&dst_path, generated.as_bytes()).expect("failed to write destination file");
    }
}
