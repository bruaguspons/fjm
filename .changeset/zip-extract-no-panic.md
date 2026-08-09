---
"fjm": patch
---

Fix `fjm install` panicking instead of returning a clean error when it can't create a temporary file while extracting a `.zip` JDK archive (e.g. full disk, no write permission on the temp dir). This path now surfaces `Error::IoError` like the rest of the extraction code instead of calling `.expect()`.
