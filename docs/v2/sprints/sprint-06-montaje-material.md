# Sprint 6: Montaje y material

**Objetivo:** cubrir lo que pasa antes de cablear: preparar material, mecanizar
y montar las piezas.

## Entregables

- **Lista de material (picking):** artículos agrupados por referencia, con
  fabricante, descripción, cantidad, GTIN y enlace a la ficha técnica.
  Marcar como recogido; leer el código de barras con lector USB o cámara.
- **Lista de montaje:** piezas por placa y carril, en orden de montaje, con
  posición; estados pendiente → montado → comprobado. Se ven en la vista de placa.
- **Listas de corte:** carriles DIN y canaletas con su longitud.
- **Plano de mecanizado:** taladros por placa y puerta con diámetro y cota,
  exportable a PDF/SVG para el taller.
- **Exportar a STL** desde la interfaz (pieza o espacio de instalación), usando
  el conversor del sprint 2.
- Etiquetas de dispositivos y bornes (CSV).

## Tareas

- [ ] Agrupar el material por artículo y por espacio de instalación.
- [ ] Integrar la lectura de códigos de barras (teclado USB y cámara con
      `BarcodeDetector` donde exista).
- [ ] Calcular el orden de montaje (por carril, de izquierda a derecha).
- [ ] Generar el plano de mecanizado en SVG con cotas.
- [ ] Botón de descarga STL.

## Criterios de aceptación

- Con el ejemplo, la lista de material coincide en referencias y cantidades con
  la del AML.
- Leer el GTIN de una pieza la marca como recogida en su orden.
- El plano de mecanizado muestra todos los taladros que trae el AML.

## Dependencias

Sprints 2 y 5.
