# -M sets the number of documents evaluated per topic. Topic 1 retrieves
# three documents and topic 2 two.
. test/regress/lib.sh
d=test/regress/negative-max-docs
Q=test/regress/qrels
R=test/regress/results

# A negative -M is refused like an unparsable one: exit 1, no output.
expect_output_status 1 /dev/null -M -1 -m num_ret -q $Q $R
expect_message 1 "trec_eval: Invalid maximum number of documents per topic '-1'" \
    -M -1 -m num_ret -q $Q $R

# -M 0 evaluates no documents; -M 2 keeps two documents of each topic.
expect_output $d/out.M0 -M 0 -m num_ret -q $Q $R
expect_output $d/out.M2 -M 2 -m num_ret -q $Q $R
