# DEPLOYMENT.md — Guía de Distribución

Proceso para distribuir m-guitar a usuarios finales en cada plataforma.

---

## Entornos

| Entorno | Rama | Distribución |
|---|---|---|
| Development | cualquiera | `cargo run --release` |
| Release | `main` (tag) | Binario + paquete por plataforma |

m-guitar es una app de escritorio. No hay "deploy a servidor".
"Deploy" significa compilar para distribución y empaquetar.

---

## Build de producción

```bash
# 1. Asegurar que todo compila limpiamente
cargo clean
cargo build --release
cargo test
cargo clippy -- -D warnings

# 2. El binario está en target/release/m-guitar
ls -lh target/release/m-guitar

# 3. (Opcional) Strip para reducir tamaño
strip target/release/m-guitar
```

---

## Empaquetado por plataforma

### Linux — AppImage

```bash
# Usando cargo-appimage (requiere appimagetool)
cargo install cargo-appimage
cargo appimage
# Output: target/appimage/m-guitar.AppImage
```

### Linux — .deb / .rpm

```bash
cargo install cargo-deb
cargo deb
# Output: target/debian/m-guitar_0.1.0_amd64.deb
```

### macOS — .app bundle

```bash
cargo install cargo-bundle
cargo bundle --release
# Output: target/release/bundle/osx/m-guitar.app
```

Firmar con `codesign` para distribución fuera de la App Store.

### macOS — .dmg

```bash
# Crear DMG a partir del .app
hdiutil create -volname "m-guitar" -srcfolder \
  target/release/bundle/osx/m-guitar.app \
  -ov -format UDZO m-guitar.dmg
```

### Windows — .msi

```bash
# Instalar WiX Toolset, luego:
cargo install cargo-wix
cargo wix
# Output: target/wix/m-guitar.msi
```

---

## Versionado

```bash
# Taggear release
git tag -a v0.1.0 -m "v0.1.0 — Visor de glifos inicial"
git push origin v0.1.0
```

El tag dispara el build de release si hay CI/CD configurado.

---

## Rollback

No aplica para app de escritorio. Si una versión tiene bugs, el usuario descarga la versión anterior.

Para desarrollo:
```bash
git checkout v0.0.0  # tag anterior
cargo build --release
```

---

## Health check post-build

```bash
# Verificar tamaño
ls -lh target/release/m-guitar

# Verificar que arranca (modo headless no disponible para GUI)
# Test manual: ejecutar y verificar que la ventana aparece
cargo run --release

# Verificar dependencias dinámicas (Linux)
ldd target/release/m-guitar
```
