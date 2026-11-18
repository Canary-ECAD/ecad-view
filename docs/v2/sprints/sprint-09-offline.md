# Sprint 9: Trabajo sin red

**Objetivo:** en obra o en una zona sin cobertura, un puesto descarga una orden,
trabaja sin conexión y sincroniza al volver.

## Entregables

- Botón **"Llevar sin conexión"** en una orden: descarga los datos de la
  revisión, páginas, vistas de placa y estados actuales al puesto.
- La UI funciona igual sin red: lista de cableado, ficha, esquemas, placa,
  cambios de estado y comentarios.
- **Cola de cambios** local que se envía al reconectar.
- **Conflictos:** si dos personas cambian el mismo extremo, gana el último cambio
  y queda registrado; si llegó una revisión nueva mientras tanto, los cambios se
  aplican sobre ella siguiendo las reglas del sprint 7.
- Indicador claro de "sin conexión" y de cambios pendientes de subir.

## Tareas

- [ ] Service worker que sirve la UI y los recursos de las órdenes descargadas.
- [ ] Almacenamiento local en IndexedDB.
- [ ] Protocolo de sincronización con identificadores de evento para no duplicar.
- [ ] Caducidad: avisar si la orden descargada tiene una revisión más nueva.
- [ ] Probar cortes de red a mitad de sincronización.

## Criterios de aceptación

- Con la red cortada, se marcan 20 extremos; al reconectar aparecen en el
  servidor con su usuario y hora originales.
- Ningún cambio se pierde ni se duplica al cortar la red durante la sincronización.

## Dependencias

Sprints 4, 5 y 7.
