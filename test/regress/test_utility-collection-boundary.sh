# -N LONG_MAX must not overflow when computing utility's fourth cell. The
# overflow wraps to the same value in a default build, so only a build with
# -fsanitize=undefined can fail this script.
. test/regress/lib.sh
D=test/regress/utility-collection-boundary
Q=test/regress/qrels.one
R=test/regress/results.one
expect_result 0 $D/out.portable /dev/null -N 2147483647 -m utility.0,0,0,1 $Q $R
need_long64
expect_result 0 $D/out.max /dev/null -N 9223372036854775807 \
    -m utility.0,0,0,1 $Q $R
