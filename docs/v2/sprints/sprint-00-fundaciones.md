# Sprint 0: Fundaciones

**Objetivo:** dejar listo el esqueleto Rust y cerrar las decisiones técnicas que
condicionan todo lo demás.

## Entregables

- Workspace Cargo en la raíz con los crates `ecad-core`, `ecad-front`,
  `ecad-server`, `ecad-ui` y `ecad-cli` (vacíos pero compilando).
- CI en GitHub Actions: `cargo fmt --check`, `cargo clippy -D warnings`,
  `cargo test`, compilación WASM de `ecad-ui`.
- Prueba de concepto de UI: Leptos con SSR desde Axum, mostrando una página SVG
  de `samples/ejemplo.epdz` con zoom y desplazamiento, a 1920×1080.
- `docs/v2/decisiones.md` con las decisiones tomadas y por qué.

## Tareas

- [x] Crear el workspace y fijar la versión de Rust (`rust-toolchain.toml`).
- [x] Elegir dependencias base: `sevenz-rust2`, `rusqlite` (bundled), `quick-xml`,
      `axum`, `tokio`, `leptos`, `serde`, `thiserror`, `tracing`.
- [x] Comprobar que `sevenz-rust2` abre el `.epdz` de ejemplo (métodos de
      compresión que usa EPLAN).
- [x] Prueba de concepto Leptos + Axum con un SVG de esquema.
- [x] ~~Comparar rápidamente con Dioxus si Leptos da problemas con SVG grandes.~~
      No hizo falta: Leptos no dio problemas (ver [decisiones](../decisiones.md)).
- [x] Decidir el sistema de estilos (CSS propio con tokens, sin framework).
- [x] Configurar CI.
- [x] Quitar `.impeccable/` y `DESIGN.md` (restos del skill de diseño eliminado)
      y reescribir `PRODUCT.md` para el taller.
- [x] Decidir qué pasa con `apps/` y `packages/` de la v1 en esta rama
      (propuesta: se quedan como referencia hasta el sprint 10).

## Criterios de aceptación

- `cargo build --workspace` y `cargo test --workspace` pasan en Windows y Linux.
- La prueba de concepto abre una página del ejemplo en menos de 1 s en un portátil normal.
- Las decisiones están escritas.

## Estado

Hecho. Decisiones y medidas en [decisiones.md](../decisiones.md). Windows
probado en local; Linux, en el CI (`.github/workflows/rust.yml`). Pendiente
repetir la medida de rendimiento con un proyecto real grande antes del sprint 5.

## Dependencias

Ninguna.
