---
"fjm": minor
---

Add `--lts` and `--latest` flags to `fjm use`, symmetric to the ones already supported by `fjm install`.

```sh-session
$ fjm use --lts --install-if-missing
$ fjm use --latest --install-if-missing
```

Previously the only way to activate the latest LTS was to pass the alias string directly (`fjm use lts-latest`).
