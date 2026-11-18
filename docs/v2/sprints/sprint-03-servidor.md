# Sprint 3: Servidor

**Objetivo:** un servidor propio donde la oficina sube proyectos y publica
revisiones, y el taller puede consultarlas.

## Entregables

- `ecad-server`: binario único con Axum y SQLite (`ecad.db`) y carpeta de datos.
- Usuarios locales con roles **oficina**, **taller** y **supervisor**; inicio de
  sesión con contraseña (argon2) y sesión por cookie. Primer administrador
  creado desde la línea de comandos.
- Subida de `.epdz`: se procesa una vez con `ecad-core` y se guarda el
  resultado (JSON/SQLite por revisión, SVG de páginas, vistas de placa).
- Proyectos y revisiones: crear proyecto, subir revisión, publicar, marcar
  obsoleta.
- Órdenes de fabricación: una por espacio de instalación (armario) a fabricar.
- API JSON documentada en `docs/v2/api.md`.
- Pantalla de oficina en Leptos: lista de proyectos, subida, publicación.

## Tareas

- [ ] Esquema SQLite según [modelo-datos.md](../modelo-datos.md) con migraciones.
- [ ] Autenticación, roles y protección CSRF.
- [ ] Subida con límite de tamaño y procesado en segundo plano con indicador de progreso.
- [ ] Almacenamiento de revisiones por proyecto (el `.epdz` original se conserva).
- [ ] Registro de eventos inmutable (quién subió, publicó, etc.).
- [ ] Configuración por fichero (`ecad.toml`): puerto, carpeta de datos, HTTPS.
- [ ] HTTPS con certificado propio de la empresa o detrás de un proxy.
- [ ] Copias de seguridad: comando `ecad-server backup`.

## Criterios de aceptación

- Un usuario de oficina sube `samples/ejemplo.epdz`, lo publica y crea una orden.
- Un usuario de taller la ve y no puede subir ni publicar.
- El servidor no hace ninguna petición a Internet.

## Dependencias

Sprints 1 y 2.
