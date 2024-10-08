use std::path::Path;

fn main() {
    let local_path = Path::new(env!("CARGO_MANIFEST_DIR"));
    println!(
        "cargo:rustc-link-arg=-T{}",
        local_path.join("user.ld").display()
    )
}
