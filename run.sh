#!/bin/bash
set -ex

sleep 1.5
# cargo fmt
# cargo clippy #  -- -D warnings
cargo build


LOCKFILE='./.flock'
if [ -f "$LOCKFILE" ]; then
    OLDPID=`cat $LOCKFILE`
    kill $OLDPID || true
    rm -f $LOCKFILE
fi


cargo run &
NEWPID="$!"
echo $NEWPID > $LOCKFILE

wait
kill $NEWPID
rm -f $LOCKFILE