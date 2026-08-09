#!/bin/bash

DIRECTORY="$(dirname "$0")"

function setup_binary() {
  TEMP_DIR="/tmp/fjm-$(date '+%s')"
  mkdir "$TEMP_DIR"
  cp ./target/release/fjm "$TEMP_DIR/fjm"
  export PATH=$TEMP_DIR:$PATH
  export FJM_DIR=$TEMP_DIR/.fjm

  # First run of the binary might be slower due to anti-virus software
  echo "Using $(which fjm)"
  echo "  with version $(fjm --version)"
}

setup_binary

RECORDING_PATH=$DIRECTORY/screen_recording

(rm -rf "$RECORDING_PATH" &> /dev/null || true)

asciinema rec \
  --command "$DIRECTORY/recorded_screen_script.sh" \
  --cols 70 \
  --rows 17 \
  "$RECORDING_PATH"
sed "s@$TEMP_DIR@~@g" "$RECORDING_PATH" | \
  svg-term \
    --window \
    --out "docs/fjm.svg" \
    --height=17 \
    --width=70
