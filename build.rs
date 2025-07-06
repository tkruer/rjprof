use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");

    let java_home = env::var("JAVA_HOME").expect("Please set JAVA_HOME to your JDK installation");

    bindgen::Builder::default()
        .header("wrapper.h")
        // point to JNI & JVMTI headers
        .clang_arg(format!("-I{}/include", java_home))
        .clang_arg(format!("-I{}/include/darwin", java_home)) // macOS
        // generate everything
        .allowlist_type(".*")
        .allowlist_function(".*")
        .allowlist_var("JVMTI_ERROR_NONE")
        .allowlist_var("JNI_OK")
        .allowlist_var("JNI_ERR")
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(PathBuf::from("src/bindings.rs"))
        .expect("Couldn't write bindings!");
}
