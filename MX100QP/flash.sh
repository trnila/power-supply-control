#!/usr/bin/env bash
SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
ARDUINO_PATH="$SCRIPT_DIR/.arduino"
SERIAL_DEV="$1"

if [ -z "$SERIAL_DEV" ]; then
	echo "Usage: $0 /dev/ttyACM0"
	exit 1
fi

if [ ! -d "$ARDUINO_PATH" ]; then
	mkdir -p "$ARDUINO_PATH"
	curl -L https://downloads.arduino.cc/arduino-cli/arduino-cli_latest_Linux_64bit.tar.gz | tar -C "$ARDUINO_PATH" -xzf -

	"$ARDUINO_PATH/arduino-cli" core install arduino:avr
fi

"$ARDUINO_PATH/arduino-cli" compile --upload -p "$SERIAL_DEV" --fqbn arduino:avr:uno "$SCRIPT_DIR/MX100QP.ino"
