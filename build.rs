// FILE: build.rs (Corrected Version)

use std::{
    env,
    fs,
    io::{self, Cursor},
    path::PathBuf,
};

const PY_VER:  &str = "3.11.7";
const REL_TAG: &str = "20240107";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if env::var("TALEA_USE_SYSTEM_PYTHON").is_ok() {
        println!("cargo:warning=TALEA_USE_SYSTEM_PYTHON set – using system python.");
        return Ok(());
    }

    // FIX: Determine the final, stable output directory based on the build profile.
    let profile = env::var("PROFILE")?; // "debug" or "release"
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let python_root = manifest_dir.join("target").join(profile).join("python-embedded");

    if python_root.exists() {
        println!("cargo:warning=Found existing embedded Python → skip download.");
        emit_rpath(&python_root);
        return Ok(());
    }

    let target  = env::var("TARGET")?;
    let triple  = target.as_str();

    let (arch, os) = match triple {
        t if t.contains("apple-darwin") => (
            if t.contains("aarch64") { "aarch64" } else { "x86_64" }, "apple-darwin",
        ),
        t if t.contains("linux") => (
            if t.contains("aarch64") { "aarch64" } else { "x86_64" }, "unknown-linux-gnu",
        ),
        _ => panic!("Unsupported compilation target for bundling: {triple}"),
    };

    let ext = "tar.zst"; // We will use zstd for both macOS and Linux now
    let filename = format!("cpython-{ver}+{tag}-{arch}-{os}-install_only.{ext}", ver = PY_VER, tag = REL_TAG, arch = arch, os = os);
    let url = format!("https://github.com/indygreg/python-build-standalone/releases/download/{tag}/{file}", tag = REL_TAG, file = filename);

    println!("cargo:warning=Downloading embedded python from:\n    {url}");

    let bytes = match reqwest::blocking::get(&url) {
        Ok(resp) if resp.status().is_success() => resp.bytes()?,
        Ok(resp) => {
            println!("cargo:warning=Server returned {}, falling back to system Python.", resp.status());
            return Ok(());
        }
        Err(e) => {
            println!("cargo:warning=Download failed: {e}, falling back.");
            return Ok(());
        }
    };

    println!("cargo:warning=Unpacking to {:?}", python_root);
    unpack_zst(&bytes, &python_root)?;

    println!("cargo:warning=Embedded Python ready at {:?}", python_root);
    emit_rpath(&python_root);
    Ok(())
}

fn unpack_zst(data: &[u8], dst: &PathBuf) -> io::Result<()> {
    let cursor = Cursor::new(data);
    let decoder = zstd::stream::read::Decoder::new(cursor)?;
    let mut archive = tar::Archive::new(decoder);
    
    // The archives have a nested 'python' directory. We need to extract its contents.
    let temp_unpack_dir = dst.with_extension("tmp-unpack");
    if temp_unpack_dir.exists() { fs::remove_dir_all(&temp_unpack_dir)?; }
    archive.unpack(&temp_unpack_dir)?;

    let source_dir = temp_unpack_dir.join("python");
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(source_dir)? {
        let entry = entry?;
        let from = entry.path();
        let to = dst.join(entry.file_name());
        fs::rename(from, to)?;
    }
    fs::remove_dir_all(temp_unpack_dir)?;
    Ok(())
}

fn emit_rpath(python_root: &PathBuf) {
    if env::var("TARGET").map(|t| t.contains("apple-darwin")).unwrap_or(false) {
        let lib = python_root.join("lib");
        println!("cargo:rustc-link-arg=-Wl,-rpath,{r}", r = lib.to_string_lossy());
    }
}