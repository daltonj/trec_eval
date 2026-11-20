# Finite underflow results, including zero, remain valid parameters.
. test/regress/lib.sh
D=test/regress/underflow-parameters
Q=$T/qrels
R=$T/run
printf '1 0 a 1\n' > "$Q"
printf '1 Q0 a 1 1 r\n' > "$R"

expect_output $D/out.set-F -m set_F.1e-310 $Q $R
expect_output $D/out.zero -m set_F.1e-400 $Q $R
for value in 1e-310 1e-400; do
    for m in iprec_at_recall 11pt_avg relstring; do
        expect_status 0 -m "$m.$value" $Q $R
    done
    for m in dcg ideal_dcg ndcg ndcg_rel Rndcg G rbp rbp_resid; do
        expect_status 0 -m "$m.1=$value" $Q $R
    done
    expect_status 0 -m "utility.$value,-$value,0,0" $Q $R
done
