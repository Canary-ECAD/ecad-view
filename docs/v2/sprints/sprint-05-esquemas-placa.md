# Sprint 5: Esquemas y vista de placa

**Objetivo:** desde cualquier cable o dispositivo, ver dónde está en el esquema
y en la placa, sin visor 3D.

## Entregables

- **Esquemas:** páginas SVG con zoom y desplazamiento, árbol de páginas,
  navegación por referencias cruzadas y puntos de interrupción (como la v1, ya
  en Leptos). El cable o dispositivo seleccionado se resalta.
- **Vista frontal de placa:** proyección 2D de cada placa o armario generada en
  el servidor a partir del E3D (`ecad-front`), con piezas coloreadas por estado
  (pendiente, hecho, comprobado).
  - Al seleccionar un cable se resaltan sus dos bornes y el recorrido.
  - Al pulsar una pieza se ve su designación, artículo y cables que llegan.
- Pantalla dividida: lista de cableado a la izquierda, esquema o placa a la derecha.

## Tareas

- [ ] Portar el saneado de SVG de la v1 (scripts de EPLAN, recursos relativos).
- [ ] Portar el índice de dispositivos y la navegación `jumpToFunction`.
- [ ] Proyección ortográfica E3D → SVG por placa: siluetas, cantos, orden de
      profundidad, texto de designación.
- [ ] Recorrido del cable: usar la geometría de `connection3did` si existe, si no
      unir los dos bornes en línea recta.
- [ ] Caché de vistas generadas por revisión.

## Criterios de aceptación

- Seleccionar un cable en la lista resalta sus extremos en la vista de placa en
  menos de 300 ms.
- La vista de placa del ejemplo es reconocible al compararla con EPLAN Pro Panel.
- No hay ningún visor 3D en la aplicación.

## Dependencias

Sprints 2 y 4.
