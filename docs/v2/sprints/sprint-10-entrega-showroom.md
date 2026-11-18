# Sprint 10: Entrega y showroom

**Objetivo:** que una empresa pueda instalar la v2 sola, y que la web pública
enseñe el producto sin exponer ningún proyecto.

## Entregables

- **Instalación:**
  - Windows: instalador que registra `ecad-server` como servicio.
  - Linux: paquete o binario con unidad systemd.
  - Imagen Docker.
  - Guía de instalación, actualización y copias de seguridad.
- **Web pública (showroom):** página estática con vídeo de demostración,
  capturas y descripción de funciones. Sin visor ni subida de archivos.
  Sustituye a la demo actual en Cloudflare.
- **Retirar la demo actual:** el despliegue de Cloudflare no está en el repo; lo
  elimina el propietario desde el panel de Cloudflare.
- **Vídeo de demostración** grabado con el proyecto de ejemplo: subir revisión,
  cablear, comprobar, comparar revisiones, panel de seguimiento.
- **Sustitución de la v1:** `apps/` y `packages/` pasan a una rama `v1` o se
  archivan; README y wiki se reescriben para la v2.

## Tareas

- [ ] Pipeline de release: binarios firmados para Windows y Linux, imagen Docker.
- [ ] Migraciones de base de datos probadas de una versión a la siguiente.
- [ ] Guion y grabación del vídeo.
- [ ] Página showroom (HTML estático, sin dependencias externas en tiempo de ejecución).
- [ ] Revisar licencia y nombre del producto (decisiones abiertas del README).
- [ ] Reescribir README, wiki y capturas.

## Criterios de aceptación

- Una persona sin conocimientos de Rust instala el servidor en Windows siguiendo
  la guía y abre el ejemplo en menos de 15 minutos.
- La web pública no permite abrir ni subir proyectos.

## Dependencias

Todos los sprints anteriores.
