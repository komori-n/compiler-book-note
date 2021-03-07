#!/bin/bash

#!/bin/bash
assert() {
    EXPECTED="$1"
    INPUT="$2"

    cargo run "$INPUT" > tmp.s
    cc -o tmp tmp.s
    ./tmp
    ACTUAL="$?"

    if [ "$ACTUAL" = "$EXPECTED" ]; then
        echo "$INPUT => $ACTUAL"
    else
        echo "$INPUT => $EXPECTED expected, but got $ACTUAL"
        exit 1
    fi
}

assert 0 0
assert 42 42
assert 21 "5+20-4"
assert 41 " 12 + 34 - 5 "
assert 1 "33>4"
assert 1 "0<2==1"
assert 0 "26==4"

echo OK