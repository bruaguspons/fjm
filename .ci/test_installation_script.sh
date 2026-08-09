#!/bin/bash

set -e

DIRECTORY="$(dirname "$0")"
SHELL_TO_RUN="$1"
PROFILE_FILE="$("$DIRECTORY/get_shell_profile.sh" "$SHELL_TO_RUN")"

ls -lah ~
echo "---"
echo "Profile is $PROFILE_FILE"
echo "---"
cat "$PROFILE_FILE"
echo "---"
echo "PATH=$PATH"
echo "---"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  fjm --version
"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  fjm install 21
  fjm ls | grep 21

  echo 'fjm ls worked.'
"

$SHELL_TO_RUN -c "
  . $PROFILE_FILE
  fjm use 21
  java --version | grep 21

  echo 'java --version worked.'
"
