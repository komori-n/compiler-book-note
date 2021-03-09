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

assert 0 '0;'
assert 42 '42;'
assert 21 "5+20-4;"
assert 41 " 12 + 34 - 5 ;"
assert 1 "33>4;"
assert 1 "0<2==1;"
assert 0 "26==4;"
assert 42 "a=42; a;"
assert 51 "a=33-4; b=26-4; a+b;"
assert 51 "hoge=33-4; fuga=26-4; hoge+fuga;"
assert 51 "a=33-4; b=26-4; return a+b; return 20;"
assert 5 "if(3>2)return 5; a=10; return a;"
assert 5 "if(3>2)return 5; else return 10;"
assert 10 "if(3<2)return 5; else return 10;"
assert 4 "a=0;while(a<4)a=a+2; return a;"
assert 5 "j=0;for(i=0;i<5;i=i+1)j=j+1;return j;"

echo OK