#!/bin/bash

set -ueo pipefail

function join_by {
    sep="$1"
    shift
    while (($# > 1)) ; do
        printf '%s%s' "$1" "$sep"
        shift
    done
    if (($#)) ; then
        printf '%s\n' "$1"
    fi
}

function run {
    desc="$1"
    shift
    cmd=$(join_by ' && ' "$@")

    echo $desc
    echo
    echo '```bash'
    for x in "$@"; do
        if [ -z "$x" ]; then
            echo
        else
            echo '$ '$x
        fi
        set +e
        eval "$x"
        set -e
    done
    echo '```'
    echo
}

IN="in.bam"

run \
    "Basics:" \
    "samtools view $IN | head -n4 | samcut -H | column -t"

run \
    "Flags:" \
    "samtools view $IN | head -n4 | samcut -H flag read1 read2 paired dup flags | column -t"

run \
    "Expand all flags:" \
    "samtools view $IN | head -n4 | samcut -H flag flagss flags | column -t"

run \
    "Get stats about flag:" \
    'function count { sort | uniq -c | sort -nrk1,1; }' \
    'cols="dup read1 read2 paired"' \
    "(echo count \$cols; samtools view $IN | samcut \$cols | count | head -n5) | column -t"

run \
    "Get tags:" \
    "samtools view $IN | head -n5 | samcut -H qname SM MD MQ MC XA | column -t"

run \
    "Histogram of flags and tags:" \
    "samtools view $IN | samcut NM read1 proper_pair | count | column -t"
