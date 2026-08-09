@echo off
cd /d %*
if "%FJM_VERSION_FILE_STRATEGY%" == "recursive" (
  fjm use --silent-if-unchanged
) else (
  if exist .java-version (
    fjm use --silent-if-unchanged
  )
)
@echo on
