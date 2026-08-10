---
"fjm": patch
---

Fix `fjm install --tool maven` failing with a 404 when resolving `--latest` or a major/minor version (e.g. `v4.0.0-rc-6` instead of `4.0.0-rc-6`). `install_maven` built the Maven Central download URL from `Version::v_str()`, which always prefixes versions with `v` for internal install-directory naming; that prefix was correctly stripped for exact-version installs but leaked through untrimmed on the `--latest` and major/minor resolution paths.
