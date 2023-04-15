#! /bin/bash

stdin=$(cat)

#length=$(echo "$stdin" | wc -c)
length=$((${#stdin}+1))

cat <<EOF
Content-Length: $length

$stdin
EOF
