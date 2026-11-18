# Sprint 2: AML y E3D

**Objetivo:** leer el AutomationML (piezas, montaje, fabricante) y el E3D, y
tener el conversor E3D → STL.

## Entregables

- Lector AML en streaming: artículos, posiciones (`Frame`), superficies de
  montaje, dimensiones, taladros, puntos de fijación, conectores, textos
  multilingües. Enlazado con las funciones del manifest.
- Lector E3D v1–v4 en Rust (`ecad_core::e3d`).
- `ecad-cli e3d2stl <entrada.e3d|.epdz> [--pieza <id>] [-o salida.stl]`:
  STL binario por pieza o por espacio de instalación, en milímetros.
- Proyección frontal mínima en `ecad-front`: siluetas de las piezas de una placa
  vistas de frente, exportadas a SVG (se completa en el sprint 5).

## Tareas

- [ ] Portar `aml.ts` a `quick-xml` sin cargar el fichero entero (31 MB).
- [ ] Extraer `Part number`, fabricante, `GTIN`, peso, descripción, enlace a ficha.
- [ ] Extraer posición y superficie de montaje; calcular la placa a la que
      pertenece cada pieza.
- [ ] Extraer taladros (`Hole, diameter`) y puntos de fijación.
- [ ] Portar `reader.ts` (E3D) y comparar resultados con la v1 en los 7 E3D del ejemplo.
- [ ] Revisar el código de lectura E3D que aporta el usuario como referencia
      (solo para entender el formato; no se copia código de terceros).
- [ ] Escritor STL binario aplicando las transformaciones de cada pieza; normales
      recalculadas; eje Z hacia arriba.
- [ ] Tests: el STL generado abre en un visor estándar y sus dimensiones coinciden
      con las del AML.

## Criterios de aceptación

- Leer el AML del ejemplo usa menos de 200 MB de memoria y tarda menos de 5 s.
- Cada pieza con artículo en el AML queda enlazada a su dispositivo del manifest.
- `e3d2stl` genera un STL válido de cada espacio de instalación del ejemplo.

## Dependencias

Sprint 1.
