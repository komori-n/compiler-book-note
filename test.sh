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

# NUM
assert 0 '0;'
assert 42 '42;'

# PAREN_EXPR
assert 42 '(((42)));'

# PRIMARY(omit)
# ( PRIMARY = NUM | PAREN_EXPR )

# UNARY
assert 42 '+42;'
assert 3 '5+(-2);'

# MUL
assert 12 '3*4;'
assert 2 '8/4;'

# ADD
assert 5 '3+2;'
assert 1 '3-2;'

# RELATIONAL
assert 1 '1 < 2;'
assert 0 '1 < 0;'
assert 1 '1 <= 1;'
assert 0 '1 <= 0;'
assert 1 '2 > 1;'
assert 0 '0 > 1;'
assert 1 '1 >= 1;'
assert 0 '0 >= 1;'

# EQUALITY
assert 1 '3 == 3;'
assert 0 '3 == 4;'
assert 0 '3 != 3;'
assert 1 '3 != 4;'

# ASSIGN
assert 3 'a=3; a;'
assert 13 'a=3; b=10; c=a+b; c;'

# STMT
assert 3 '1; 2; return 3; 4; return 5;'
assert 3 'if (1 < 2) return 3; return 5;'
assert 5 'if (1 > 2) return 3; return 5;'
assert 3 'if (1 < 2) 3; else 5;'
assert 5 'if (1 > 2) 3; else 5;'
assert 1 'b = 0; a=2; while (a = a - 1) b = b + 1; b;'
assert 2 'b = 0; a=2; while (b) a = a + 1; a;'
assert 4 'b=1; for (a=0;a<3;a=a+1) b=b+1; b;'
assert 4 'a=0;b=1; for (;a<3;a=a+1) b=b+1; b;'
assert 3 'b=1; for (a=0;;a=a+1) if (b==3) return b; else b=b+1;'
assert 3 'for (a=0;a<3;) a=a+1; a;'

echo OK