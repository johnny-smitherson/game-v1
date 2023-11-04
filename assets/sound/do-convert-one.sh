set -e
set -o pipefail
ORIGINAL="./original/$1"
OUTPUT_TMP="./ogg/$1.1.wav"
OUTPUT="./ogg/$1.ogg"
DIRNAME="$(dirname $OUTPUT)"
BASENAME="$(basename $OUTPUT_TMP)"
NORM_BASE="$BASENAME.norm.wav"
OUTPUT_NORM="$DIRNAME/$NORM_BASE"

if [ -f "$OUTPUT" ]; then 
    if [  "$(wc -c $OUTPUT | cut -f1 -d' ')" == "0" ]; then
        echo "del empty: $OUTPUT"
        rm -f "$OUTPUT"
    else
        exit 0
    fi
fi

mkdir -p "$(dirname $OUTPUT)"
echo "$ORIGINAL --> $OUTPUT_TMP"
rm -f "$OUTPUT_NORM" "$OUTPUT_TMP" 
ffmpeg -i "$ORIGINAL"  -acodec pcm_s16le -ar 44100 "$OUTPUT_TMP" || ( rm -f "$OUTPUT_NORM" "$OUTPUT_TMP" "$OUTPUT" && exit 255 )
if [ "$(wc -c $OUTPUT_TMP | cut -f1 -d' ')" == "0" ]; then
    echo "del empty: $OUTPUT_TMP"
    rm -f "$OUTPUT_TMP"
fi

if [ -f "$OUTPUT_TMP" ]; then
    echo "$BASENAME --> $NORM_BASE"
    ( set -ex    ; cd $DIRNAME ; ffmpeg-normalize "$BASENAME" -o   "$NORM_BASE" --normalization-type peak  --target-level 0 --force || exit 255 )
    rm -f "$OUTPUT_TMP"
fi

ffmpeg -i "$OUTPUT_NORM" -c:a libvorbis "$OUTPUT" || ( rm -f "$OUTPUT_NORM" "$OUTPUT_TMP" "$OUTPUT" && exit 255 )
rm -f "$OUTPUT_NORM"
echo "DONE $OUTPUT"