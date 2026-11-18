# Sprint 8: Seguimiento

**Objetivo:** el jefe de taller y calidad ven el avance y la trazabilidad de
cada orden sin preguntar a nadie.

## Entregables

- **Panel de órdenes:** todas las órdenes abiertas con avance de material,
  montaje, cableado y comprobación, fecha de inicio, última modificación y
  tiempo de trabajo.
- **Detalle de una orden:** avance por tipo de tarea y por armario, operarios
  que han trabajado en ella con lo que ha hecho cada uno, consumo de material.
- **Comentarios:** en cualquier cable, pieza o página; la oficina los responde y
  los cierra. Contador de comentarios abiertos por orden.
- **Historial:** registro de eventos filtrable (quién, qué, cuándo).
- **Informe de cierre:** PDF de la orden terminada con avance, comprobaciones,
  incidencias y revisiones aplicadas.

## Tareas

- [ ] Consultas de agregación sobre estados y eventos.
- [ ] Cálculo de tiempo de trabajo a partir de los eventos (sesiones activas).
- [ ] Componentes de gráfico simples (barras de avance; sin tartas).
- [ ] Comentarios con notificación a oficina.
- [ ] Generación del informe PDF en el servidor.

## Criterios de aceptación

- El panel muestra el mismo avance que la lista de cableado de cada orden.
- Cada estado del informe de cierre tiene su usuario y hora.

## Dependencias

Sprints 4 y 6.
