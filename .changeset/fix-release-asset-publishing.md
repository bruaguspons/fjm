---
"fjm": patch
---

Fix `.ci/install.sh` always failing to download because release binaries were never published. The release workflow tagged private packages incorrectly (skipped by `changeset tag` without `privatePackages.tag`), and even after tagging, the tag push used the default `GITHUB_TOKEN`, whose events don't trigger further workflow runs — so the job that builds and uploads binaries never ran. The tag-and-release step now runs in the same workflow as the binary builds, so publishing a version reliably produces a GitHub Release with all platform binaries attached.
