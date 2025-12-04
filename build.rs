use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    // Only build frontend assets if not in a release profile for Docker
    // (Docker builds will run npm separately for better layer caching)
    let skip_npm = env::var("SKIP_NPM_BUILD").is_ok();

    if skip_npm {
        println!("cargo:warning=Skipping npm build (SKIP_NPM_BUILD is set)");
        return;
    }

    println!("cargo:rerun-if-changed=package.json");
    println!("cargo:rerun-if-changed=package-lock.json");
    println!("cargo:rerun-if-changed=build.js");
    println!("cargo:rerun-if-changed=static/js/src/");
    println!("cargo:rerun-if-changed=static/css/");

    // Check if node_modules exists
    let node_modules_exists = Path::new("node_modules").exists();

    if !node_modules_exists {
        println!("cargo:warning=node_modules not found, running npm install...");

        let status = Command::new("npm")
            .arg("install")
            .status()
            .expect("Failed to run npm install. Make sure npm is installed.");

        if !status.success() {
            panic!("npm install failed");
        }
    }

    // Run npm build
    println!("cargo:info=Building frontend assets with npm...");

    let status = Command::new("npm")
        .arg("run")
        .arg("build")
        .status()
        .expect("Failed to run npm build. Make sure npm is installed.");

    if !status.success() {
        panic!("npm build failed");
    }

    println!("cargo:info=Frontend assets built successfully");
}
