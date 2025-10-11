# hilang
A small programming language

```
$ cat program.hi
"30" -> int -> \x;
"1" -> int -> \i;
(
    i =< x -> (
        i % ("15" -> int) == ("0" -> int) -> "FizzBuzz"
        | i % ("3" -> int) == ("0" -> int) -> "Fizz"
        | i % ("5" -> int) == ("0" -> int) -> "Buzz"
        | i
    ) -> output;
    i + ("1" -> int) -> i
).loop | pass
$ hilang program.hi
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
17
Fizz
19
Buzz
Fizz
22
23
Fizz
Buzz
26
Fizz
28
29
FizzBuzz
```
