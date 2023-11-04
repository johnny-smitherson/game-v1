set -e
set -o pipefail
ORIGINAL="./original/$1"
BASENAME="$(basename $ORIGINAL)"
DIRNAME="$(dirname $ORIGINAL)"
OUTPUT="./ogg/$1.ogg"
if [ -f $OUTPUT ]; then exit 0; fi
mkdir -p "$(dirname $OUTPUT)"
echo "$ORIGINAL --> $OUTPUT"
(
    cd "$DIRNAME"
    ffmpeg-normalize "$BASENAME" -c:a libvorbis -b:a 64k -o "$BASENAME.ogg"
)
mv "$ORIGINAL.ogg" $OUTPUT