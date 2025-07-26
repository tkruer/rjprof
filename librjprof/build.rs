use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let java_home = env::var("JAVA_HOME").expect("Please set JAVA_HOME to your JDK installation");

    // Determine platform-specific include directory
    let platform_include = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "win32"
    } else {
        panic!("Unsupported platform")
    };

    bindgen::Builder::default()
        .header("src/wrapper.h")
        // point to JNI & JVMTI headers
        .clang_arg(format!("-I{}/include", java_home))
        .clang_arg(format!("-I{}/include/{}", java_home, platform_include))
        // generate everything
        .allowlist_type(".*")
        .allowlist_function(".*")
        .allowlist_var("JVMTI_ERROR_NONE")
        .allowlist_var("JNI_OK")
        .allowlist_var("JNI_ERR")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from("src/bindings/gen_bindings.rs"))
        .expect("Couldn't write bindings!");
}
