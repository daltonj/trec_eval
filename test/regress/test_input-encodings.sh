# BOM, CRLF and a final CR preserve values in every reader. Appending LF
# makes a final CR valid; a CR within a line must still be refused.
. test/regress/lib.sh
Q=test/regress/qrels
R=test/regress/results
for source in qrels prefs results zscores; do
    f=test/regress/$source
    { printf '\357\273\277'; [ "$source" != qrels ] || echo '# comment'; cat $f; } > "$T/$source.bom"
    awk '{ printf "%s\r\n", $0 }' $f > "$T/$source.crlf"
    printf '%s\r' "$(cat $f)" > "$T/$source.lastcr"
done
for format in qrels qrels_jg qrels_prefs prefs results zscores; do
    case $format in
    qrels|qrels_jg|qrels_prefs) source=qrels; set -- -R $format ;;
    prefs) source=prefs; set -- -R prefs ;;
    results) source=results; set -- ;;
    zscores) source=zscores; set -- -m map -Z ;;
    esac
    case $format in
    results) save_output "$T/plain" $Q $R ;;
    zscores) save_output "$T/plain" "$@" test/regress/$source $Q $R ;;
    *) save_output "$T/plain" "$@" test/regress/$source $R ;;
    esac
    for ending in bom crlf lastcr; do
        case $format in
        results) expect_output "$T/plain" $Q "$T/$source.$ending" ;;
        zscores) expect_output "$T/plain" "$@" "$T/$source.$ending" $Q $R ;;
        *) expect_output "$T/plain" "$@" "$T/$source.$ending" $R ;;
        esac
    done
done
printf '1 0 d1 1\n1 0 d2 0\r1 0 d3 1\n' > "$T/cr"
expect_message 2 "CR character without LF in line 2 of qrels file '$T/cr'" "$T/cr" $R
