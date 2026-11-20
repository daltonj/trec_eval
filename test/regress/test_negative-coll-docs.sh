# -N sets the number of documents in the collection, which utility uses
# for its fourth coefficient. With -m utility.1,-1,0,1, each topic scores
# N - 2.
. test/regress/lib.sh
d=test/regress/negative-coll-docs
m=utility.1,-1,0,1

# A negative -N is refused like an unparsable one: exit 1, no output.
for n in "-N -1" -N-1; do
    expect_output_status 1 $d/out.empty $n -q -m $m $d/qrels $d/results
done
expect_message 1 "trec_eval: Invalid number of documents in collection '-1'" \
    -N -1 -q -m $m $d/qrels $d/results

# -N 10 gives 8 for each topic.
expect_output $d/out.N10 -N 10 -q -m $m $d/qrels $d/results
