use std::env;
use std::fs;
use std::path::PathBuf;

const PRIVATE: &str = "\
#[doc(hidden)]
pub mod __private$$ {
    #[doc(hidden)]
    pub use crate::export::*;
}
";

fn main() {
    println!("cargo:rerun-if-changed=tests/regression");

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let patch_version = env::var("CARGO_PKG_VERSION_PATCH").unwrap();

    let mod_private = PRIVATE.replace("$$", &patch_version);
    fs::write(out_dir.join("private.rs"), mod_private).unwrap();

    let mut mod_place = fs::read_to_string(manifest_dir.join("src").join("place.rs")).unwrap();
    mod_place = mod_place.replace("__private", &format!("__private{patch_version}"));
    fs::write(out_dir.join("place.rs"), mod_place).unwrap();
}
