fn main() {
    let status = std::process::Command::new("nasm")
        .args(&["-f", "elf64", "-o", "boot.o", "src/boot.asm"])
        .status()
        .expect("Failed to run nasm");

    if !status.success() {
        panic!("nasm failed");
    }

    // attach header
    println!("cargo:rustc-link-arg=boot.o");
    // change linker
    println!("cargo:rustc-link-arg=-Tlinker.ld");
}