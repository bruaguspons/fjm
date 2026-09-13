---
"fjm": minor
---

Add Gradle as a fully managed tool, on par with Java and Maven: `fjm install --tool gradle <version>` (exact version, `--latest`, and major-minor input), `fjm ls-remote --tool gradle`, and `use`/`current`/`default`/`uninstall`/`alias`/`unalias` all support `--tool gradle`. Gradle versions are resolved against `https://services.gradle.org` (overridable via `--gradle-dist-mirror`/`FJM_GRADLE_DIST_MIRROR`), two-component version strings like `8.10` are coerced to `8.10.0` internally, and `.gradle-version` files are discovered using the same Local/Recursive strategy as `.java-version`/`.maven-version`. `fjm env` exports `GRADLE_HOME` when a Gradle version is active.
