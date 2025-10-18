use std::{
    env::{self, current_dir},
    ffi::OsString,
    fs::{read_to_string, write},
    process::{Command, exit}
};

#[cfg(not(any(unix, target_vendor = "apple")))]
compile_error!("snailx only supports Unix and macOS");

struct Version {
    major: usize,
    minor: usize
}

fn rust_version() -> Version {
    let rustc = env::var_os("RUSTC").unwrap_or_else(|| OsString::from("rustc"));
    let mut cmd = if let Some(wrapper) = env::var_os("RUSTC_WRAPPER").filter(|w| !w.is_empty()) {
        let mut cmd = Command::new(wrapper);
        cmd.arg(rustc);
        cmd
    } else {
        Command::new(rustc)
    };

    let out = cmd.arg("-V").output().expect("failed to execute rustc to get version");

    if !out.status.success() {
        eprintln!("rustc execution failed: {}", String::from_utf8_lossy(&out.stderr));
        exit(1);
    }

    let stdout = String::from_utf8(out.stdout).expect("rustc output was not valid UTF-8");

    // split into "rustc", version, and hash
    let mut parts = stdout
        .trim()
        .split(' ')
        // get ver
        .nth(1)
        .expect("rustc output did not contain version info")
        // cut off stability tag
        .split('-')
        .next()
        .expect("rustc version info did not contain a version")
        // split into major, minor, patch
        .splitn(3, '.');

    let major = parts
        .next()
        .expect("rustc version did not contain a major version")
        .parse()
        .expect("rustc version major version was not a number");
    let minor = parts
        .next()
        .expect("rustc version did not contain a minor version")
        .parse()
        .expect("rustc version minor version was not a number");

    Version { major, minor }
}

// as jank as this may seem, because rust is stupid, we need this to make it compile on older
// versions of rustc. cfg_attr/rustversion do not work because the attribute potentially being
// configured out is evaluated before the configuring out, so it will still error for unrecognized
// attrs (like unsafe).
fn main() {
    let v = rust_version();
    let cwd = current_dir().expect("failed to get current directory");
    let raw_src = cwd.join("src");
    let direct_src = raw_src.join("direct.rs.src");

    // Let Cargo know when to rerun the build script for the Rust source generation
    println!("cargo:rerun-if-changed={}", raw_src.display());
    println!("cargo:rerun-if-env-changed=RUSTC");
    println!("cargo:rerun-if-env-changed=RUSTC_WRAPPER");
    // println!("cargo:rerun-if-env-changed=WIN_BUFSIZE");

    let src_contents = read_to_string(direct_src).expect("failed to read source file");

    let generated = src_contents
        .as_str()
        .replace(
            "[Replace me with link section]",
            if v.minor > 81 || v.major > 1 {
                "#[unsafe(link_section = \".init_array.00098\")]"
            } else {
                "#[link_section = \".init_array.00098\"]"
            }
        );
        // .replace(
        //     "[Replace me with windows bufsize]",
        //     &env::var("WIN_BUFSIZE").unwrap_or_else(|_| String::from("1024"))
        // )

    let dst_path = raw_src.join("direct.rs");

    // only write if the destination differs to avoid unnecessary i/o
    if let Ok(existing) = read_to_string(&dst_path) {
        if existing == generated {
            exit(0);
        }
    }
    write(&dst_path, generated.as_bytes()).expect("failed to write destination file");
}
