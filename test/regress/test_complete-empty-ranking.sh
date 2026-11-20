# Under -c, topic 2 is in the qrels but not in the run. It is scored as a
# ranking that retrieves nothing, whatever the measure's parameters. Its
# one relevant document is not retrieved, and -N is not given, so the
# number of documents in the collection is 0:
#   utility.p1,p2,p3,p4 = p3 * 1 + p4 * (0 - 1), which is 0 for the
#   default 1,-1,0,0 and 2 - 3 = -1 for 1,-1,2,3;
#   rbp_resid is 0 for any p, because nothing unjudged is retrieved;
#   num_ret is 0, and relstring is empty, also with a length (relstring.3);
#   a gain for level -1, the level of a document outside the qrels, gives
#   ndcg 0, because an empty ranking holds no such document.
# In qrels.first, topic 1 is the missing topic and comes first. Its empty
# ranking is the first ranking scored, so nothing is allocated yet: map
# is 0 for topic 1 and 1 for topic 2, whose one relevant document ranks
# first, and the mean is 0.5. Every all_trec measure must succeed on it.
. test/regress/lib.sh
d=test/regress/complete-empty-ranking

m="-c -q $d/qrels $d/results"
expect_output $d/out.utility -m utility $m
expect_output $d/out.utility-default -m utility.1,-1,0,0 $m
expect_output $d/out.utility-custom -m utility.1,-1,2,3 $m
expect_output $d/out.rbp_resid -m rbp_resid $m
expect_output $d/out.rbp_resid-p -m rbp_resid.p=0.5 $m
expect_output $d/out.num_ret -m num_ret $m
expect_output $d/out.relstring -m relstring $m
expect_output $d/out.relstring-length -m relstring.3 $m
expect_output $d/out.ndcg -m ndcg.-1=1 $m
expect_output $d/out.first -c -q -m map $d/qrels.first $d/results.first
expect_status 0 -c -q -m all_trec $d/qrels.first $d/results.first
