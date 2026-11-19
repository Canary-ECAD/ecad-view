# Producto

ECAD View v2: la herramienta del **taller de fabricación de cuadros
eléctricos** para cablear, montar y comprobar un armario a partir del `.epdz`
que exporta EPLAN. Alcance completo y plan por sprints en
[docs/v2/README.md](docs/v2/README.md).

## Usuarios

1. **Montador / cableador** (principal). Delante del armario, con un monitor de
   22" en el puesto o un monitor portátil de 15,6" en obra. Ratón, a veces
   pantalla táctil, a veces con guantes. Luz de nave: fluorescentes, reflejos,
   rincones mal iluminados. Quiere saber qué cable toca ahora, cómo es
   (sección, color, punteras, pelado), de dónde a dónde va y por dónde pasa, y
   marcarlo como hecho sin pelearse con la herramienta.
2. **Oficina técnica.** Sube el `.epdz` exportado de EPLAN, publica cada
   revisión y responde a los comentarios del taller. No quiere tocar dos veces
   un dato que ya está en EPLAN.
3. **Jefe de taller / calidad.** Ve el avance por proyecto, armario y operario,
   y quién ha hecho y comprobado cada cosa. Necesita poder enseñar esa
   trazabilidad a un cliente o en una auditoría.

## Para qué sirve

Que el dato de fabricación que ya existe en el proyecto EPLAN llegue al puesto
de trabajo entero y legible, y que el trabajo hecho quede registrado:

- **Cablear:** lista de cableado con todo lo que trae el `.epdz` (extremos,
  punteras, dirección de salida, recorrido, puentes), localizable en esquema y
  en la vista frontal de la placa, con estado por cable y por extremo.
- **Montar:** lista de montaje, picking de material, taladros y etiquetas.
- **Revisar:** cuando llega una revisión nueva, qué cambia, qué hay que
  desmontar y qué trabajo hecho queda pendiente de revisar.
- **Seguir:** avance, comentarios y quién hizo qué y cuándo.

Éxito: un cableador termina un armario sin imprimir la lista de cableado ni
llamar a la oficina, y el jefe de taller sabe en todo momento qué falta.

## Personalidad

**Precisa, directa, de taller.** Habla el idioma del proyecto y del oficio
(cable, borne, puntera, placa, armario, revisión) y no el de la exportación
(`typeId`, `propid`, rutas del archivo). Responde en una frase y no adorna:
nombra la cosa, no la celebra. La confianza se gana con datos correctos y
rapidez, no con adjetivos.

## Anti-referencias

- **EPLAN y el CAD clásico.** Ribbons cargados, diálogos grises con biseles,
  densidad sin jerarquía.
- **SaaS genérico.** Rejillas de tarjetas redondeadas, degradados, cifras
  gigantes de adorno, ilustraciones de stock.
- **Interfaz de móvil estirada.** Mucho aire y poco dato en un monitor de
  1080p: aquí la pantalla está para llenarla de información útil.

## Principios de diseño

1. **El dato de fabricación manda.** Todo lo que el `.epdz` trae y sirve para
   fabricar se muestra; nada se esconde detrás de "más detalles" si el
   operario lo necesita para el siguiente cable.
2. **Densidad ordenada.** Pantallas de 1920×1080 en horizontal llenas de
   información, con jerarquía clara (peso, agrupación, alineación). La
   respuesta a "está muy cargado" es ordenar, no quitar.
3. **Se lee a un metro.** Contraste alto para luz de nave y tamaños que se leen
   de pie delante del armario. Objetivos táctiles de 40 px como mínimo para
   monitores portátiles táctiles y uso con guantes.
4. **El esquema es nítido.** Las páginas y la vista de placa se dibujan como
   vectores, con zoom sin pérdida y desplazamiento inmediato.
5. **Propio y trazable.** Todo corre en la red de la empresa, sin servicios
   externos ni telemetría. Cada cambio de estado queda registrado con usuario y
   hora, y una revisión nueva nunca borra trabajo hecho: lo marca para revisar.
6. **Honesta con lo desconocido.** Si un dato del archivo no se puede
   interpretar, se dice, en lugar de inventar o callarlo.

## Accesibilidad

- Texto de cuerpo con contraste mínimo de 4,5:1 en los dos temas (claro por
  defecto en taller; oscuro disponible). La hoja del esquema sigue siendo
  blanca en ambos porque los colores de EPLAN lo asumen.
- Todo se puede manejar con teclado; el foco se ve siempre.
- El estado nunca se comunica solo con color (hecho / pendiente / revisar
  llevan también icono o texto).
- Interfaz en español primero; las etiquetas deben aguantar textos más largos
  de otros idiomas sin desbordar.
