use bindgen::Bindings;
use std::env;
use std::path::Path;
use std::path::PathBuf;

/// use xcrun to find the path to the MacOS SDK
fn macos_get_framework_sdk_path() -> String {
    let output = std::process::Command::new("xcrun")
        .args(["--show-sdk-path"])
        .output()
        .expect("failed to execute xcrun");

    let path = String::from_utf8(output.stdout).expect("failed to parse xcrun output");
    path.trim().to_string() + "/System/Library/Frameworks"
}

fn ensure_submodules_initialized() {
    // Check if submodules are initialized by looking for key files
    let ffglsdk_header = Path::new("FFGLSDK/Include/FFGL.h");
    let resolume_header = Path::new("ffgl-resolume/source/lib/ffgl/FFGL.h");
    
    if !ffglsdk_header.exists() || !resolume_header.exists() {
        println!("cargo:warning=Git submodules not initialized, attempting to initialize...");
        
        // Try to initialize submodules
        let output = std::process::Command::new("git")
            .args(["submodule", "update", "--init", "--recursive"])
            .output();
            
        match output {
            Ok(result) => {
                if !result.status.success() {
                    let stderr = String::from_utf8_lossy(&result.stderr);
                    println!("cargo:warning=Failed to initialize submodules: {}", stderr);
                    println!("cargo:warning=Please run 'git submodule update --init --recursive' manually");
                }
            }
            Err(e) => {
                println!("cargo:warning=Failed to run git command: {}", e);
                println!("cargo:warning=Please run 'git submodule update --init --recursive' manually");
            }
        }
    }
}

/// When cross-compiling to Windows GNU, find mingw-w64 include path (for windows.h).
fn mingw_include_path() -> Option<String> {
    if let Ok(p) = env::var("MINGW_INCLUDE_PATH") {
        return Some(p);
    }
    // Homebrew mingw-w64 (Apple Silicon and Intel)
    let homebrew_paths = [
        "/opt/homebrew/opt/mingw-w64/toolchain-x86_64/x86_64-w64-mingw32/include",
        "/usr/local/opt/mingw-w64/toolchain-x86_64/x86_64-w64-mingw32/include",
    ];
    for p in &homebrew_paths {
        if Path::new(p).exists() {
            return Some((*p).to_string());
        }
    }
    None
}

fn main() {
    println!("cargo:rerun-if-env-changed=BINDGEN_EXTRA_CLANG_ARGS");
    println!("cargo:rerun-if-env-changed=TARGET");
    println!("cargo:rerun-if-env-changed=MINGW_INCLUDE_PATH");
    ensure_submodules_initialized();

    let target = env::var("TARGET").unwrap_or_default();
    let (clang_args_ffgl, clang_args_ffgl2): (Vec<String>, Vec<String>) = if target.contains("windows") {
        let mut ffgl = vec![
            "-x".into(),
            "c++".into(),
            "-IFFGLSDK/Include".into(),
        ];
        let mut ffgl2: Vec<String> = vec![
            "-x".into(),
            "c++".into(),
            "-Iffgl-resolume/source/lib/ffgl".into(),
            "-Iffgl-resolume/deps/glew-2.1.0/include".into(),
        ];
        if let Some(inc) = mingw_include_path() {
            ffgl.push(format!("-I{inc}"));
            ffgl2.push(format!("-I{inc}"));
        }
        println!("cargo:rustc-link-lib=opengl32");
        (ffgl, ffgl2)
    } else if target.contains("darwin") || target.contains("macos") {
        let macos_framework_path = macos_get_framework_sdk_path();
        let ffgl = vec![
            "-x".into(),
            "c++".into(),
            "-IFFGLSDK/Include".into(),
            "-F".into(),
            macos_framework_path.clone(),
            "-framework".into(),
            "OpenGL".into(),
        ];
        let ffgl2 = vec![
            "-x".into(),
            "c++".into(),
            "-Iffgl-resolume/source/lib/ffgl".into(),
            "-F".into(),
            macos_framework_path,
            "-framework".into(),
            "OpenGL".into(),
        ];
        (ffgl, ffgl2)
    } else {
        (
            vec!["-x".into(), "c++".into(), "-IFFGLSDK/Include".into()],
            vec!["-x".into(), "c++".into(), "-Iffgl-resolume/source/lib/ffgl".into()],
        )
    };

    let clang_args_ffgl: Vec<&str> = clang_args_ffgl.iter().map(String::as_str).collect();
    let clang_args_ffgl2: Vec<&str> = clang_args_ffgl2.iter().map(String::as_str).collect();

    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("env variable OUT_DIR not found"));

    // Generate the bindings (raw_line suppresses clippy lints in generated code).
    build_to_out_file(
        bindgen::Builder::default()
            .clang_args(&clang_args_ffgl)
            .header("wrapper.h")
            .raw_line("#![allow(clippy::unnecessary_transmute)]")
            .generate()
            .unwrap(),
        &out_dir.join("ffgl1.rs"),
    );

    build_to_out_file(
        bindgen::Builder::default()
            .clang_args(&clang_args_ffgl2)
            .header("wrapper.h")
            .raw_line("#![allow(clippy::unnecessary_transmute)]")
            .generate()
            .unwrap(),
        &out_dir.join("ffgl2.rs"),
    );
}

fn build_to_out_file(bindings: Bindings, file: &Path) {
    // Write them to the crate root.
    bindings
        .write_to_file(file)
        .expect("could not write bindings");
}
