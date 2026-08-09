#!/bin/bash

set -e

export PATH=$PATH_ADDITION:$PATH

GAL_PROMPT_PREFIX="\e[34m✡\e[m  "

function type() {
  printf $GAL_PROMPT_PREFIX
  echo -n " "
  echo $* | node .ci/type-letters.js
}

type 'eval "$(fjm env)"'
eval "$(fjm env)"

type 'fjm --version'
fjm --version

type 'cat .java-version'
cat .java-version

type 'fjm install 21'
fjm install 21

type 'fjm use 21'
fjm use 21

type 'java --version'
java --version

sleep 2
echo ""
