# nvmd-command

English | [简体中文](./README.zh-CN.md)

`nvmd-command` (nombre del binario: `nvmd`) es una CLI ligera para la gestión de versiones de Node.js construida con Rust.

También es el entorno de ejecución de comandos utilizado por [nvm-desktop](https://github.com/1111mp/nvm-desktop):

- Gestiona versiones de Node.js desde la terminal (`install`, `use`, `list`, `uninstall`, etc.)
- Enruta los comandos shim (`node`, `npm`, `npx`, `corepack`, ...) hacia la versión correcta de Node
- Soporta tanto la versión predeterminada global como la selección de versión a nivel de proyecto

---

## Características

- Ejecutable nativo único (Rust)
- Flujo de trabajo de instalación/cambio de múltiples versiones de Node.js
- Soporte para archivos de versión a nivel de proyecto (predeterminado: `.nvmdrc`)
- Consulta de la versión actual y rutas de instalación
- Ecosistema y disposición de datos compartidos con nvm-desktop

---

## Compilar desde el código fuente

> Se recomienda a los usuarios finales instalar `nvmd` desde la distribución de nvm-desktop.

### 1) Prerrequisitos

Instalar el toolchain de Rust (stable): <https://www.rust-lang.org/tools/install>

### 2) Compilación

```bash
# compilación de depuración (debug)
cargo build

# compilación de lanzamiento (release)
cargo build --release
```

Binarios resultantes:

- Linux / macOS: `target/release/nvmd`
- Windows: `target/release/nvmd.exe`

---

## Inicio rápido

```bash
# instalar una versión de Node.js
nvmd install 20.11.1

# establecer la versión predeterminada global
nvmd use 20.11.1

# mostrar la versión activa
nvmd current

# listar versiones instaladas
nvmd ls
```

Versión a nivel de proyecto (escribe el archivo de versión en el directorio actual, predeterminado `.nvmdrc`):

```bash
nvmd use 18.20.3 --project
```

---

## Referencia de comandos

Usa `nvmd --help` para obtener ayuda completa.

| Comando | Descripción |
|---|---|
| `nvmd current` | Mostrar la versión activa de Node.js |
| `nvmd install <version>` | Instalar una versión específica |
| `nvmd list` / `nvmd ls` | Listar versiones instaladas |
| `nvmd list --group` | Listar grupos de proyectos |
| `nvmd uninstall <version>` | Desinstalar una versión específica |
| `nvmd use <version>` | Establecer la versión predeterminada global |
| `nvmd use <version> --project` | Establecer la versión para el proyecto actual |
| `nvmd which <version>` | Mostrar la ruta de instalación de una versión (Unix: `.../bin`) |

> La entrada de versión soporta tanto `v20.11.1` como `20.11.1`.

### Prioridad de resolución de versiones

Al resolver qué versión de Node.js ejecutar, `nvmd` utiliza este orden (de mayor a menor prioridad):

1. Variable de entorno `NVMD_NODE_VERSION`
2. Archivo de versión del proyecto (predeterminado: `.nvmdrc`, buscado desde el directorio actual hacia arriba en los directorios padres)
3. Archivo predeterminado global (`$NVMD_HOME/default`)

`NVMD_NODE_VERSION` tiene la prioridad más alta y anula la configuración del proyecto/global para el entorno del proceso actual.

---

## Cómo funciona (basado en shims)

`nvmd-command` utiliza shims en lugar de hooks de shell:

1. Ejecutas `node`, `npm` u otros comandos relacionados.
2. Un shim reenvía la solicitud a `nvmd`.
3. `nvmd` resuelve la versión desde el archivo de versión del proyecto (predeterminado `.nvmdrc`) o el predeterminado global (`$NVMD_HOME/default`).
4. `nvmd` inicia un proceso hijo con un `PATH` ajustado que apunta al directorio de Node objetivo.

Esto mantiene el cambio de versiones rápido, fiable y agnóstico al shell utilizado.

---

## Disposición de directorios y archivos de datos

Raíz predeterminada: `$HOME/.nvmd` (se puede sobrescribir con `NVMD_HOME`).

```text
$NVMD_HOME/
├─ bin/            # shims y punto de entrada ejecutable
├─ versions/       # versiones de Node.js instaladas
├─ default         # versión de Node predeterminada global
├─ setting.json    # configuraciones
├─ projects.json   # mapeo de proyecto a versión
├─ groups.json     # información de grupos de proyectos
└─ packages.json   # metadatos de shims de paquetes globales
```

---

## Configuración (`setting.json`)

`$NVMD_HOME/setting.json` soporta:

```json
{
  "directory": "/custom/path/to/versions",
  "mirror": "https://nodejs.org/dist",
  "node_version_file": ".nvmdrc"
}
```

- `directory`: Directorio de instalación de versiones de Node.js (predeterminado: `$NVMD_HOME/versions`)
- `mirror`: Mirror de descarga de Node.js (predeterminado: `https://nodejs.org/dist`)
- `node_version_file`: Nombre del archivo de versión del proyecto (predeterminado: `.nvmdrc`)

---

## FAQ

### `nvmd use <version>` dice "not installed"

Instálala primero:

```bash
nvmd install <version>
```

### El cambio de proyecto no funciona

Verifica que:

- El directorio actual (o los directorios padres) contenga el archivo de versión (predeterminado `.nvmdrc`)
- La versión en el archivo esté instalada
- El `PATH` de tu shell priorice los shims en `$NVMD_HOME/bin`

### Cambiar el mirror de descarga

Actualiza `mirror` en `setting.json` y ejecuta `nvmd install <version>` nuevamente.

---

## Integración con nvm-desktop

`nvmd-command` trabaja estrechamente con [nvm-desktop](https://github.com/1111mp/nvm-desktop):

- nvm-desktop proporciona flujos de trabajo GUI e integración del ecosistema
- nvmd-command proporciona el despacho de CLI/runtime de alto rendimiento

Para detalles de empaquetado/integración:
<https://github.com/1111mp/nvm-desktop#develop-and-build>
