# A preference count is an unsigned short. 65535 judgment groups that
# agree on a pair are counted; one more is refused with exit status 4
# instead of wrapping the count to 0. Groups with one subgroup take the
# equivalence-class path, and groups with two take the preference arrays.
. test/regress/lib.sh
printf '1 Q0 a 1 4 r\n1 Q0 b 2 3 r\n1 Q0 c 3 2 r\n1 Q0 d 4 1 r\n' > "$T/run"
for form in ec pa; do
    for groups in 65535 65536; do
        awk -v n=$groups -v form=$form 'BEGIN {
            for (g = 0; g < n; g++) {
                printf "1 g%d s1 a 2\n1 g%d s1 b 1\n", g, g
                if (form == "pa")
                    printf "1 g%d s2 c 1\n1 g%d s2 d 0\n", g, g
            } }' > "$T/prefs"
        if [ $groups = 65535 ]; then
            expect_status 0 -R prefs -m prefs_pair "$T/prefs" "$T/run"
            grep -q 'prefs_pair.*1\.0000' "$T/te.output" ||
                te_fail "prefs_pair is not 1.0000"
        else
            expect_status 4 -R prefs -m prefs_pair "$T/prefs" "$T/run"
            grep -q 'more than 65535 judgment groups prefer one document' \
                "$T/te.error" || te_fail "no message for the count limit"
        fi
    done
done
