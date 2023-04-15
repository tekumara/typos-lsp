#! /bin/bash

stdin=$(cat)

length=$((${#stdin}+1))

cat <<EOF
Content-Length: $length

$stdin
EOF
