---
"fjm": minor
---

Add Maven version management alongside JDKs, via a new multi-tool activation model.

`fjm` now manages Java and Maven as independent, parallel tools:

- `fjm install --tool maven <version>`, `fjm use --tool maven <version>`, `fjm ls-remote --tool maven`, `fjm uninstall --tool maven`, `fjm alias`/`fjm unalias`/`fjm default` all accept `--tool maven` (defaulting to `--tool java` when omitted, for backward compatibility).
- A `.maven-version` file works alongside `.java-version`, resolved independently in the same directory.
- `fjm env` now exports `JAVA_HOME` (new, additive) and, when a Maven version is active, `MAVEN_HOME`, alongside the existing PATH/symlink behavior.
- A new `FJM_MAVEN_DIST_MIRROR` env var / `--maven-dist-mirror` flag lets you override the Maven Central mirror, symmetric to the existing `FJM_JDK_DIST_MIRROR`.
- Maven downloads are checksum-verified against Maven Central's `.sha512` sidecar files.

**Behavior change:** `fjm current` and `fjm list` (`fjm ls`), when run without `--tool`, now print every active/installed tool (e.g. `java: 17.0.2` / `maven: 3.9.9`) instead of a single bare JDK version line. Pass `--tool java` to get the previous single-line output back. This is the only breaking output-shape change in this release — JDK-only workflows that always passed an explicit tool, or that don't parse `current`/`list` output, are unaffected.

Gradle support is intentionally not included — the underlying `ToolKind`/`RemoteVersionIndex` abstraction is designed so it can be added later without reworking the activation model.
