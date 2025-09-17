# hilang
A small programming language

```
$ cat program.hi
(
    "30" -> int -> <"x">.store
    -> "1" -> int -> <"i">.store
    -> [<(
        <<"i">.load, <"x">.load>.le
        -> [ (
            <<"i">.load, ("15" -> int)>.mod -> <"t">.store
            -> <<"t">.load, ("0" -> int)>.eq
            -> "FizzBuzz" -> output
        ) | [ (
            <<"i">.load, ("3" -> int)>.mod -> <"t">.store
            -> <<"t">.load, ("0" -> int)>.eq
            -> "Fizz" -> output
        ) | [ (
            <<"i">.load, ("5" -> int)>.mod -> <"t">.store
            -> <<"t">.load, ("0" -> int)>.eq
            -> "Buzz" -> output
        ) | (
            <"i">.load -> output
        ) ] ] ]
        -> <<"i">.load, ("1" -> int)>.add -> <"i">.store
    )>.loop | pass]
)
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
