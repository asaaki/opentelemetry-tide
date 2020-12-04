use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn generate_build_vars(output_path: &Path) {
    let profile = env::var("PROFILE").unwrap();
    let mut f = File::create(&output_path.join("build_vars.rs")).expect("Could not create user build_vars.rs file");
    f.write_all(format!("static PROFILE: &str = \"{}\";", profile).as_bytes())
        .expect("Unable to write user agent");
}

fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not specified");
    let out_path = Path::new(&out_dir);
    generate_build_vars(&out_path);
}
