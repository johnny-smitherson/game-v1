BASE_DIR=assets/debug_diagrams/spawn_tank
mkdir -p $BASE_DIR


dot_to_png () {
    cat $1.dot | dot -Tsvg > $1.svg
    convert $1.svg $1.png
    rm -f $1.svg $1.dot
}


for i in \
        "update" \
        "pre_update" \
        "post_update" \
        "first" \
        "last" \
        "startup" \
        "pre_startup" \
        "post_startup" \
        "asset_events" \
        ; do
    cargo run --bin debug_print_schedule $i assets/debug_diagrams/schedule_$i.dot
    dot_to_png assets/debug_diagrams/schedule_$i
done

cargo run --bin debug_print_render_graph assets/debug_diagrams/render.dot
dot_to_png assets/debug_diagrams/render