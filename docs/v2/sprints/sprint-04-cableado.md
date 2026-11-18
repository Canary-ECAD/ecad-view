# Sprint 4: Lista de cableado

**Objetivo:** el montador abre una orden, ve sus cables, los cablea y marca cada
extremo. Es la pantalla principal de la v2.

## Entregables

- **Lista de cableado** a 1920×1080: nº, origen, destino, sección, color,
  longitud, tratamiento de extremos, recorrido, cadena, estado.
  - Origen y destino partidos en dispositivo y borne; el prefijo común se atenúa.
  - Muestra de color del conductor junto al código.
  - Filas de cable multipolar agrupando sus hilos.
- **Ficha del cable** (panel lateral): todos los datos de fabricación de cada
  extremo (pelado, puntera, dirección de salida, par de apriete si está
  confirmado), potencial, tipo de conductor, comentarios.
- **Estados por extremo:** pendiente → hecho → comprobado. El estado del cable se
  deriva de los dos extremos. Deshacer el último cambio.
- **Búsqueda, filtros y orden:** búsqueda por designación; filtros por
  sección, color, tipo, recorrido, estado y regleta; orden por varias columnas.
- **Modos de trabajo:** por regleta o dispositivo (todos los cables que llegan a
  `-XD11`), por recorrido o libre.
- Varios operarios en la misma orden: los cambios de otros aparecen en vivo.
- Atajos de teclado y botones grandes para monitores táctiles.
- Etiqueta del cable: imprimir o exportar CSV para la impresora de etiquetas.

## Tareas

- [ ] Diseño visual de la pantalla (mockup a 1920×1080) y tokens de color/tipografía.
- [ ] Componente de tabla virtualizada (miles de filas sin perder fluidez).
- [ ] API de estados con registro de evento por cambio.
- [ ] Actualización en vivo (WebSocket o SSE).
- [ ] Persistencia de filtros y columnas por usuario.
- [ ] Modo alto contraste y modo oscuro.
- [ ] Exportación de etiquetas (CSV genérico; formatos de fabricante más adelante).

## Criterios de aceptación

- Con el ejemplo, el montador filtra por `-XD11`, marca los dos extremos de un
  cable y el cable aparece como hecho para otro operario en menos de 1 s.
- La pantalla se usa entera sin hacer zoom en un monitor de 15,6" a 1080p.
- Todos los datos de la ficha vienen del `.epdz`; ninguno está inventado.

## Dependencias

Sprint 3.
