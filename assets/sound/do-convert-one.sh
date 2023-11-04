set -e
set -o pipefail
ORIGINAL="./original/$1"
OUTPUT="./ogg/$1.ogg"
if [ -f $OUTPUT ]; then exit 0; fi
mkdir -p "$(dirname $OUTPUT)"
echo "$ORIGINAL --> $OUTPUT"
ffmpeg -i "$ORIGINAL" -c:a libvorbis -b:a 64k "$OUTPUT"