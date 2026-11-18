# Sprint 7: Revisiones

**Objetivo:** cuando la oficina publica una revisión nueva de un proyecto que
ya está en fabricación, el taller sabe exactamente qué cambia, sin perder el
trabajo hecho.

## Entregables

- **Comparación de revisiones** (oficina y taller): cables añadidos, eliminados
  y modificados (qué campo cambia: sección, color, destino…); piezas añadidas,
  eliminadas o movidas; páginas cambiadas.
- **Aplicar la revisión a una orden en curso:**
  - Lo que no cambia conserva su estado.
  - Lo que cambia en algo hecho pasa a **revisar**.
  - Lo eliminado que estaba hecho pasa a **desmontar**, y al hacerlo a **retirado**.
  - Lo añadido entra como pendiente.
- La oficina previsualiza el impacto en las órdenes abiertas antes de publicar.
- Las revisiones anteriores quedan **obsoletas**, visibles en solo lectura.

## Tareas

- [ ] Definir claves estables para conexiones y piezas entre revisiones;
      probarlas con dos exportaciones reales del mismo proyecto.
- [ ] Algoritmo de comparación y su presentación.
- [ ] Migración de estados en una transacción, con evento por cada cambio.
- [ ] Filtro "solo cambios" en la lista de cableado y en la vista de placa.
- [ ] Notificar a los usuarios de la orden afectada.

## Criterios de aceptación

- Con dos revisiones de prueba (por ejemplo, cambiando la sección de 3 cables y
  quitando 1), la comparación lista exactamente esos 4 cambios.
- Tras aplicar la revisión, ningún extremo hecho y no afectado pierde su estado.

## Dependencias

Sprint 4. Hace falta un segundo `.epdz` del mismo proyecto con cambios conocidos.
