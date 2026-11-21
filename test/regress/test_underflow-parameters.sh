# Finite underflow results, including zero, remain valid parameters.
# Rounding to zero warns; a subnormal value does not.
. test/regress/lib.sh
D=test/regress/underflow-parameters
Q=$T/qrels
R=$T/run
printf '1 0 a 1\n' > "$Q"
printf '1 Q0 a 1 1 r\n' > "$R"

expect_result 0 $D/out.set-F /dev/null -m set_F.1e-310 "$Q" "$R"
echo "Warning, value \`1e-400' out of range, read as \`0' (parameter of measure 'set_F')." > "$T/err"
expect_result 0 $D/out.zero "$T/err" -m set_F.1e-400 "$Q" "$R"
