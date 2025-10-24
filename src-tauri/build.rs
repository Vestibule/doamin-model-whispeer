fn main() {
    // Set MACOSX_DEPLOYMENT_TARGET to 10.15 to avoid compilation errors on older macOS versions
    // related to unavailable `std::filesystem` features in whisper.cpp.
    // See https://github.com/ggerganov/whisper.cpp/issues/2347
    println!("cargo:rustc-env=MACOSX_DEPLOYMENT_TARGET=10.15");
    tauri_build::build()
}
