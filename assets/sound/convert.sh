set -e
set -o pipefail
# rm -rf ogg/*
find original -type f | cut -d'/' -f2- | xargs -n1 -P16 bash do-convert-one.sh
echo done