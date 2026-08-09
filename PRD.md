# PRD — fjm (Fast Java Manager)

## 1. Resumen

`fjm` es un gestor de versiones de Java, escrito en Rust, inspirado en gestores de versiones ya probados del ecosistema Node y JVM (ver el README para el detalle de prior-art). Objetivo: instalación instantánea, un solo binario, y una experiencia tan simple como la de esos gestores, pero para el ecosistema Java (JDKs, y potencialmente otras herramientas del ecosistema como Maven/Gradle en el futuro — fuera de alcance para v1).

## 2. Problema

Hoy, para manejar múltiples versiones de Java en la misma máquina, la opción de referencia es `sdkman`. Su mecanismo de activación es una función de shell que se sourcea en `.bashrc`/`.zshrc`. Esto significa que **solo funciona en shells interactivas que hayan sourceado ese init**. Una shell no interactiva (cron, systemd service, `docker exec`, un runner de CI sin login shell) no tiene `java` disponible, salvo que se replique manualmente la lógica de activación.

### Investigación de prior art (2026-08-08)

Se revisó el código fuente de los proyectos de referencia disponibles en este workspace. El detalle de la comparativa y sus conclusiones vive en el README (sección "Why fjm?"), para no duplicar contenido.

**Conclusión:** el problema no es un detalle de implementación que `sdkman` resolvió mal — es un patrón heredado por toda una generación de gestores de versiones basados en shell functions / `eval`. Todos requieren que el shell haya sourceado algo al iniciar.

La alternativa que sí rompe esa dependencia es el modelo de **shims** (`rbenv`, `asdf`, `pyenv`): un binario fijo instalado una sola vez en un directorio que ya está en el `$PATH` del sistema (ej. `/usr/local/bin/java`), que resuelve la versión en tiempo de ejecución (leyendo `.java-version` hacia arriba desde el cwd) y hace `exec` al binario real. No depende de sourcing porque el propio shim ya está en el PATH desde la instalación.

### Decisión para esta etapa

Por decisión explícita para el MVP (2026-08-08): **implementar el mismo mecanismo que el proyecto de referencia analizado en el README** (binario Rust + `eval "$(fjm env)"` en el shell RC), para priorizar velocidad de desarrollo y reutilizar un diseño ya probado.

El modelo de shims queda **documentado como opción futura** (ver sección 8) pero **explícitamente fuera de alcance de esta etapa** — no se implementa ahora.

## 3. Objetivos (v1)

- Un solo binario estático, sin dependencias runtime, multiplataforma (macOS, Linux, Windows).
- Instalación y cambio de versión de JDK con un comando.
- Soporte de archivo de versión por proyecto (`.java-version`, análogo a `.node-version`/`.nvmrc`).
- Activación vía `eval "$(fjm env)"`, con soporte `--use-on-cd` para autoswitch al cambiar de directorio.
- Arranque instantáneo (mismo principio de diseño: Rust, sin runtime interpretado).

## 4. No objetivos (v1)

- Resolver el problema de shells no interactivas (queda para una v2 con shims — ver sección 8).
- Gestión de otras herramientas del ecosistema (Maven, Gradle, etc.).
- Reemplazar `sdkman` como gestor de "candidates" genérico — el foco es exclusivamente JDKs.

## 5. Usuarios objetivo

Desarrolladores Java/Kotlin que trabajan con múltiples proyectos que requieren distintas versiones de JDK, y que ya usan o conocen flujos equivalentes en el ecosistema Node.

## 6. Alcance funcional (MVP)

Calcado del set de comandos del proyecto de referencia analizado (ver README), adaptado a JDKs:

- `fjm list-remote` — listar versiones de JDK disponibles para instalar.
- `fjm install <version>` — instalar una versión.
- `fjm list` — listar versiones instaladas localmente.
- `fjm use <version>` — activar una versión en la shell actual.
- `fjm default <version>` — fijar versión por defecto.
- `fjm current` — mostrar versión activa.
- `fjm env [--use-on-cd] [--shell <shell>]` — imprimir el script de activación para `eval`.
- `fjm exec -- <cmd>` — ejecutar un comando con una versión específica sin cambiar la shell.
- `fjm uninstall <version>`
- `fjm completions --shell <shell>`
- `fjm alias` / `fjm unalias`

### Fuente de distribución de JDKs

Decisión (2026-08-08): se usa la [API de Adoptium/Temurin](https://api.adoptium.net) para listar y descargar builds de JDK. `fjm list-remote`/`fjm install` ya resuelven versiones y descargan releases reales contra esa API; `FJM_JDK_DIST_MIRROR`/`--jdk-dist-mirror` permiten apuntar a un mirror compatible.

## 7. Shells soportados (v1)

`bash`, `zsh`, `fish`, `powershell`. Windows CMD como soporte parcial.

## 8. Trabajo futuro — Shims (explícitamente fuera de alcance ahora)

Para eliminar la dependencia de sourcing y que `java` funcione en cualquier contexto (cron, CI, `docker exec`, servicios systemd), evaluar en una etapa posterior:

- Instalar shims fijos (`java`, `javac`, etc.) en un directorio del `$PATH` del sistema, gestionados por `fjm`.
- Cada shim resuelve la versión en tiempo de invocación (walk-up de `.java-version` desde el cwd) y hace `exec` al binario real.
- Trade-off conocido: overhead de un proceso extra por invocación (despreciable frente al arranque de la JVM), a cambio de funcionar sin sourcing.

Esta sección debe revisarse como propuesta formal (spec/design) cuando se decida abordarla — no implica compromiso de que se vaya a implementar.

## 9. Métricas de éxito

- Tiempo de arranque de `fjm env`/`fjm use` sub-50ms.
- Instalación en un solo comando, sin dependencias externas más allá de `curl`/`unzip`.

## 10. Preguntas abiertas

- Estrategia de resolución cuando hay `.java-version` y `.sdkmanrc` simultáneamente en el mismo repo (compatibilidad con proyectos que ya usan sdkman).
