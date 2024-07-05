fn main() {
    println!("cargo:rustc-link-arg-bin=risosir=--script=src/kernel/kernel.ld");
}
