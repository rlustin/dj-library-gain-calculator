#!/bin/bash

# Find what attributes are missing in models.rs, compared to what can be found
# in .nml

usage() {
  echo "$0 path/to/src/models.rs [nml files]+"
  exit 1
}

if [[ $# -lt 2 ]]
then
  usage $0
fi

MODELS_RS=$1
shift 1

tmp1=$(mktemp)
trap "rm $tmp1" 0

for i in "$@"
do
  egrep -o '[A-Z]+="' "$i" | sed 's/="//g' >> $tmp1
done

entries=$(cat $tmp1 | sort | uniq)

for i in $entries
do
  found=$(egrep -o $i $MODELS_RS);
  if [[ -z $found ]]
  then
    echo "$i missing in models.rs"
  fi
done
