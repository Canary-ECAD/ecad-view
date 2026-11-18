# ECAD View v2: alcance

Reescritura completa en Rust de Covaga ECAD Viewer, pensada para el **taller de
fabricación de cuadros eléctricos**. Esta rama (`rework/rust`) crece en paralelo
a la versión actual; cuando la v2 cubra lo necesario, la reemplaza.

## Qué cambia respecto a la v1

| | v1 (actual) | v2 |
| --- | --- | --- |
| Uso principal | Consultar un `.epdz` con un móvil delante del armario | Fabricar el armario: cablear, montar, comprobar |
| Dónde se procesa | En el dispositivo, cada vez que se abre | En el servidor, una vez por revisión |
| Instalación | PWA pública / Tauri / Capacitor | Servidor propio de la empresa (sin nube) + puestos de taller |
| Pantalla | Móvil y tablet primero | Monitor de 22" y portátil de 15,6", horizontal, 1920×1080 |
| 3D | Visor three.js del E3D | Sin visor. Vista frontal 2D de la placa y conversor E3D → STL |
| Estado | Solo lectura | Avance por cable y por extremo, por operario, con revisiones |
| Código | TypeScript + React | Rust de punta a punta |

## Usuarios

1. **Montador / cableador** (principal). Delante del armario, en un monitor de
   22" en el puesto o en un monitor portátil en obra. Ratón, a veces táctil,
   a veces con guantes. Quiere saber qué cable toca ahora, cómo es, de dónde a
   dónde va y marcarlo como hecho sin pelearse con la herramienta.
2. **Oficina técnica.** Sube el `.epdz` exportado de EPLAN, publica la revisión
   y responde a los comentarios del taller.
3. **Jefe de taller / calidad.** Ve el avance por proyecto, armario y operario,
   y quién ha hecho y comprobado cada cosa.

## Principios

1. **Propio y cerrado.** Todo corre en la red de la empresa. Ningún dato sale a
   servicios externos; sin telemetría, sin CDN en tiempo de ejecución.
2. **Online primero, offline posible.** El trabajo normal es contra el servidor;
   un puesto puede descargarse un proyecto, trabajar sin red y sincronizar.
3. **El dato de fabricación manda.** Todo lo que el `.epdz` trae y sirve para
   fabricar se muestra: punteras, dirección de salida, recorrido, par de apriete,
   taladros, referencias. Ver [modelo-datos.md](modelo-datos.md).
4. **Densidad ordenada.** Pantallas de 1080p llenas de información útil, con
   jerarquía clara y contraste alto para luz de nave. Objetivos táctiles
   ≥ 40 px para monitores portátiles táctiles.
5. **Trazabilidad.** Cada cambio de estado queda registrado con usuario y hora.
   Una revisión nueva nunca borra trabajo hecho: lo marca para revisar.

## Arquitectura

```
crates/
  ecad-core/     Lectura de formatos: .epdz (7z), manifest.db, AML, E3D.
                 Modelo de proyecto. E3D -> STL. Sin red ni UI.
  ecad-front/    Proyección frontal 2D (E3D -> SVG) de placas y armarios.
  ecad-server/   Servidor Axum: usuarios, proyectos, revisiones, estados,
                 API, ficheros estáticos de la UI. Base de datos SQLite.
  ecad-ui/       Interfaz web en Leptos (Rust -> WASM), servida por el server.
  ecad-cli/      Herramientas: inspeccionar un .epdz, convertir E3D -> STL.
```

- **Un solo binario** `ecad-server` con la UI embebida y una base SQLite.
  Se instala como servicio de Windows o con Docker.
- **UI en Leptos** porque los esquemas son SVG y el navegador los dibuja mejor
  que cualquier alternativa; se confirma en el sprint 0.
- **Offline:** service worker + IndexedDB en el puesto, cola de cambios que se
  sube al reconectar.
- La v1 sirve de referencia para los formatos: `packages/e3d-core/src/reader.ts`,
  `manifest.ts`, `aml.ts` y `docs/formats.md`.

## Fuera de alcance

- Visor 3D interactivo.
- Edición del proyecto (la fuente sigue siendo EPLAN).
- Móviles. La v1 sigue existiendo para quien la necesite.
- Servicios en la nube y cuentas externas.

## Sprints

Cada sprint deja algo utilizable y probado con `samples/ejemplo.epdz`.

| # | Sprint | Resultado |
| --- | --- | --- |
| 0 | [Fundaciones](sprints/sprint-00-fundaciones.md) | Workspace Rust, CI, decisión de UI |
| 1 | [Lectura del proyecto](sprints/sprint-01-lectura-proyecto.md) | `ecad-core` lee `.epdz` y `manifest.db` |
| 2 | [AML y E3D](sprints/sprint-02-aml-e3d.md) | Piezas, montaje, E3D y conversor STL |
| 3 | [Servidor](sprints/sprint-03-servidor.md) | Usuarios, subida y publicación de revisiones |
| 4 | [Lista de cableado](sprints/sprint-04-cableado.md) | El taller cablea y marca estados |
| 5 | [Esquemas y vista de placa](sprints/sprint-05-esquemas-placa.md) | Localizar cada cable en esquema y placa |
| 6 | [Montaje y material](sprints/sprint-06-montaje-material.md) | Lista de montaje, picking, taladros, etiquetas |
| 7 | [Revisiones](sprints/sprint-07-revisiones.md) | Comparar revisiones y gestionar desmontajes |
| 8 | [Seguimiento](sprints/sprint-08-seguimiento.md) | Panel de avance, comentarios, trazabilidad |
| 9 | [Trabajo sin red](sprints/sprint-09-offline.md) | Descargar proyecto y sincronizar |
| 10 | [Entrega y showroom](sprints/sprint-10-entrega-showroom.md) | Instaladores, web con vídeo, retirar la demo |

## Decisiones abiertas

- Autenticación: usuarios locales en SQLite (por defecto) o integración con el
  directorio activo de la empresa (LDAP) más adelante.
- Nombre del producto v2 (¿se mantiene "Covaga ECAD Viewer"?).
- Licencia: la v1 es MIT. Si la v2 va a venderse o instalarse en clientes,
  decidirlo antes del sprint 10.
