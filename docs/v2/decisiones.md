# Decisiones técnicas (v2)

Decisiones tomadas en el [sprint 0](sprints/sprint-00-fundaciones.md), con el
porqué. Cuando una cambie, se edita aquí con la fecha y el motivo.

## Toolchain

- **Rust 1.100.0**, fijado en `rust-toolchain.toml` con `rustfmt`, `clippy` y
  el target `wasm32-unknown-unknown`. Edición 2024, `resolver = "3"`.
  Se fija la versión exacta para que CI y todos los puestos compilen igual;
  se sube a propósito, no por accidente. `cargo-leptos` 0.3 ya exige ≥ 1.97.
- En Windows se usa el host `x86_64-pc-windows-gnu` (sin Visual Studio).
  Basta con `rustup set default-host x86_64-pc-windows-gnu` y un `gcc` de
  MinGW en el `PATH` (lo necesita `rusqlite` con SQLite embebido). En CI, el
  runner de Windows usa MSVC, así que se prueban los dos.
- `unsafe_code = "forbid"` en todo el workspace.

## Workspace

```
crates/
  ecad-core     .epdz (7z), SVG, y en sprints 1-2 manifest.db, AML y E3D
  ecad-front    proyección frontal E3D -> SVG (vacío hasta el sprint 5)
  ecad-server   Axum + Leptos SSR; binario único
  ecad-ui       componentes Leptos (SSR + hidratación en WASM)
  ecad-cli      binario `ecad` (de momento, `ecad ls <archivo.epdz>`)
```

Las versiones de las dependencias se declaran una sola vez en
`[workspace.dependencies]` del `Cargo.toml` raíz.

## Dependencias base

| Crate | Versión | Para qué | Notas |
| --- | --- | --- | --- |
| `sevenz-rust2` | 0.25 | Abrir el `.epdz` | Sin `default-features` (no hace falta comprimir ni AES); con `bzip2` y `ppmd` por si algún export los usa |
| `rusqlite` | 0.43, `bundled` | `manifest.db` y la base del servidor | SQLite embebido: no depende de la versión del sistema |
| `quick-xml` | 0.42 | SVG y AML | Desde 0.42 la API es de `&str`, no de bytes |
| `axum` | 0.8 | Servidor HTTP | |
| `tokio` | 1 | Runtime | |
| `tower-http` | 0.6 | Compresión gzip, ficheros | |
| `leptos` (+ `leptos_axum`, `leptos_router`, `leptos_meta`) | 0.8 | UI | 0.9 está en RC; se sube cuando sea estable |
| `serde`, `thiserror`, `tracing` | | Comunes | |

**`sevenz-rust2` abre el ejemplo.** `samples/ejemplo.epdz` (6,5 MB, 22
entradas) se descomprime entero en memoria en ~0,4 s en release. Se lee todo
de una vez porque el 7z es sólido: leer una entrada suelta obliga a
descomprimir las anteriores del bloque. Lo comprueba
`crates/ecad-core/tests/sample.rs`, que también abre `manifest.db` con
`rusqlite` y cuenta las páginas.

## Interfaz: Leptos con SSR desde Axum

**Decisión: Leptos 0.8**, renderizado en el servidor e hidratado en el
navegador. No hizo falta probar Dioxus: Leptos no dio problemas con el SVG.

Prueba de concepto (`cargo leptos watch`, ver abajo): la página se renderiza en
el servidor con la lista de páginas del ejemplo, el WASM se hidrata y el visor
pide el SVG de la página a `/datos/paginas/{n}`.

- **El SVG no va dentro del HTML.** Se pide aparte y se incrusta con
  `innerHTML`. Así el HTML no lleva el SVG dos veces (marcado + datos
  serializados de la hidratación), el navegador lo guarda en caché por URL y
  el service worker del sprint 9 lo puede guardar igual.
- **Zoom cambiando el `viewBox`**, no con `transform` de CSS: las líneas se
  redibujan nítidas a cualquier aumento. El `viewBox` siempre tiene la
  proporción del contenedor, así que pasar de píxeles a milímetros es lineal.
  Rueda (centrada en el cursor), arrastrar, botones `−` / `+` / `Ajustar`,
  doble clic y teclas `+`, `-`, `0`.
- **Saneado en el servidor** (`ecad_core::svg::sanitize_with`): fuera
  `<script>`, `<foreignObject>`, atributos `on*` y enlaces `javascript:`; las
  imágenes relativas (`EPLAN.png`, logos) se reescriben a `/datos/recursos/`.
  Los saltos de referencia cruzada se reimplementarán en el sprint 5.

Medido a 1920×1080, servidor en release en local:

| | |
| --- | --- |
| WASM (gzip) | 226 KB (547 KB sin comprimir) |
| HTML de la página | 1,5 KB (gzip) |
| SVG de la página 1 | 5 KB gzip / 38 KB |
| Página visible tras abrir la URL | < 50 ms |
| Cambio de página (página 7, 2 020 elementos SVG) | 28 ms |
| Arranque del servidor (abrir y sanear el `.epdz`) | 0,4 s |

Muy por debajo del objetivo de 1 s. Las páginas del ejemplo son pequeñas
(≤ 120 KB); **pendiente** repetirlo con un proyecto real grande antes del
sprint 5.

## Estilos

**CSS propio con tokens** (`crates/ecad-ui/style/main.css`), sin framework.
Custom properties en `:root` para color, tipografía, espaciado, radio y el
tamaño mínimo táctil (`--target: 40px`); tema claro por defecto y oscuro con
`prefers-color-scheme`. La hoja del esquema se queda blanca en los dos temas
porque los colores de EPLAN lo asumen. `cargo-leptos` copia el fichero a
`/pkg/ecad.css`.

## Código de la v1 en esta rama

`apps/` y `packages/` **se quedan como referencia** hasta el sprint 10 (sobre
todo `packages/e3d-core/src/{epdz,manifest,aml,reader}.ts` y
`docs/formats.md`). No se tocan salvo para arreglar algo bloqueante; el CI de
Node (`ci.yml`) sigue igual y el de Rust va aparte (`rust.yml`).

Se han quitado `.impeccable/` y `DESIGN.md`, que eran del skill de diseño
retirado, y `PRODUCT.md` está reescrito para el taller.

## CI

`.github/workflows/rust.yml`:

- `lint` (Linux): `cargo fmt --check`, `cargo clippy -D warnings` del
  workspace y de `ecad-ui` en WASM.
- `test` (Linux y Windows): `cargo build` y `cargo test` del workspace.
- `wasm` (Linux): `ecad-ui` con `--features hydrate` para
  `wasm32-unknown-unknown`, perfil `wasm-release`.

Todo con `--locked`: `Cargo.lock` se versiona.

## Cómo arrancar

```bash
cargo install cargo-leptos --locked
cargo leptos watch                # desarrollo, http://127.0.0.1:3000
cargo leptos build --release      # binario en target/release/ecad-server
```

`ECAD_EPDZ=ruta/al/archivo.epdz` elige el proyecto (por defecto,
`samples/ejemplo.epdz`). Para ejecutar el binario de release sin
`cargo-leptos`, hacen falta `LEPTOS_OUTPUT_NAME=ecad`,
`LEPTOS_SITE_ROOT=target/site` y `LEPTOS_SITE_ADDR=127.0.0.1:3000`. La primera
vez, `cargo-leptos` descarga `wasm-bindgen` y `wasm-opt`.

## Notas

- En un puesto Windows con el host GNU, el ejecutable de pruebas vacío de
  `ecad-cli` fue bloqueado ("Acceso denegado", probablemente el antivirus).
  Como el binario no tiene tests, lleva `test = false`.
- El orden de las páginas es provisional (por título); el orden real sale de
  `manifest.db` en el sprint 1.
