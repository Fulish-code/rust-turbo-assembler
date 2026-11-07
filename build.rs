//! En Windows incrusta el ícono en el .exe (Explorador, barra de tareas, accesos directos).
fn main() {
    println!("cargo:rerun-if-changed=assets/rust-tasm.ico");
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/rust-tasm.ico");
        res.set("ProductName", "Rust Turbo Assembler");
        res.set("FileDescription", "Rust Turbo Assembler");
        if let Err(e) = res.compile() {
            println!("cargo:warning=No se pudo incrustar el ícono: {e}");
        }
    }
}
