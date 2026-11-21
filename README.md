# ORBITA — Sistema Helios

Proyecto final de Gráficas por Computadora: una simulación espacial interactiva hecha en Rust. El programa usa un **renderer de software propio**: transforma y proyecta la geometría, calcula iluminación y profundidad, y dibuja cada cuadro en un framebuffer desde la CPU.

## Video de demostración

> **Pendiente:** graba un recorrido de la cámara y guarda el archivo como `media/recorrido.mp4`. Al subirlo al repositorio, este enlace quedará disponible:

[▶ Ver el recorrido por el sistema Helios](media/recorrido.mp4)

El video puede mostrar el despegue, el vuelo entre planetas, un cambio de altura y un acercamiento a una órbita. No necesita narración. Si el MP4 resulta demasiado pesado para GitHub, súbelo a YouTube o a GitHub Releases y reemplaza el enlace anterior por el enlace público.

GIF opcional: puedes guardar una animación corta como `media/recorrido.gif` y compartirla en Discord. Cuando el archivo exista, puedes mostrarlo aquí con `![Vista animada del sistema Helios](media/recorrido.gif)`.

## Características

- Una estrella y doce planetas de distintos tamaños, colores y texturas, alineados en el plano eclíptico.
- Planetas que se trasladan en órbitas circulares visibles y rotan sobre sus ejes.
- Campo de estrellas como fondo espacial.
- Nave 3D OBJ que acompaña a la cámara, con movimiento suave de vuelo.
- Cámara con controles de giro, cabeceo, altura y desplazamiento lateral para explorar el sistema en 3D.
- Colisiones que mantienen la nave fuera del interior de los planetas.
- Modo optimizado y modo normal, para priorizar rendimiento o calidad visual.
- Rasterización en CPU con proyección en perspectiva, z-buffer e iluminación difusa.

## Requisitos

- Rust y Cargo instalados (por medio de [rustup](https://rustup.rs/)).
- Una sesión gráfica compatible con `minifb`.
- Conexión a internet la primera vez que se compila, para descargar dependencias.

## Ejecutar

Desde la carpeta del proyecto:

```sh
cargo run --release
```

La primera compilación puede tardar un poco. Para generar el ejecutable optimizado sin iniciarlo, ejecuta `cargo build --release`; el resultado queda dentro de `target/` y no se incluye en Git.

## Controles

| Tecla | Acción |
| --- | --- |
| `R` | Acelerar hacia adelante |
| `F` | Frenar y, si se mantiene, retroceder |
| `D` / `G` | Desplazarse lateralmente |
| Flechas izquierda / derecha | Girar la nave |
| Flechas arriba / abajo | Inclinar el morro |
| `T` / `C` | Subir / bajar |
| `Tab` | Alternar modo optimizado y normal |
| `Esc` | Salir |

También se puede cambiar el modo de calidad con el botón que aparece en pantalla.

## Estructura

```text
.
├── assets/
│   ├── nave_ronald.obj
│   ├── sphere.obj
│   └── textures/
├── media/          # Video y GIF de demostración
├── src/main.rs     # Renderer, simulación, cámara y controles
├── Cargo.toml
├── Cargo.lock
└── README.md
```

Los archivos de compilación de Cargo se guardan en `target/`, que está excluida de Git. El repositorio conserva `Cargo.lock` para fijar las versiones de dependencias.

## Recursos y créditos

- `assets/nave_ronald.obj`: modelo de nave suministrado para el proyecto.
- Malla esférica y texturas: recursos del [repositorio Lab6graficasporcomputadora](https://github.com/ronald1777/Lab6graficasporcomputadora).
- Ventana y entrada de teclado: [minifb](https://crates.io/crates/minifb). Lectura de texturas PNG: [image](https://crates.io/crates/image).

## Autor

Ronald Catún.
