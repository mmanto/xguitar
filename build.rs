fn main() {
    let target_arch = std::env::var("CARGO_CFG_TARGET_ARCH").unwrap_or_default();
    if target_arch == "wasm32" {
        // sfizz es una librería C++ nativa — no se linkea en el build WASM.
        return;
    }

    // El .pc de sfizz en algunas distros (ej. Arch, paquete `sfizz-lib`)
    // declara mal el nombre de la librería (`Libs: -llibsfizz` en vez de
    // `-lsfizz`, aunque el archivo real es `libsfizz.so`) — por eso solo
    // usamos pkg-config para ubicar el libdir y linkeamos "sfizz" a mano en
    // vez de confiar en el nombre que trae el .pc.
    match pkg_config::Config::new()
        .cargo_metadata(false)
        .probe("sfizz")
    {
        Ok(lib) => {
            for path in &lib.link_paths {
                println!("cargo:rustc-link-search=native={}", path.display());
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=pkg-config no encontró sfizz ({e}); intentando linkear con el \
                 path de librerías por defecto del sistema."
            );
        }
    }
    println!("cargo:rustc-link-lib=dylib=sfizz");
}
