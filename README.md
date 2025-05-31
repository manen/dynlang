# `dynlang`

###### don't get caught up on the name it's just as unserious as it sounds

only interpreted for now because that's easier

## currently implemented

- variables (int, float, string, array, object)
- functions with input & output args
- conditional execution & conditional expressions
- boolean logic (<, >, ==, ||, &&)
- closures, context switching
- unconditional loops (with `break`), for loops
- [iterators, iterator helper functions](/iter.dl)

and:

- a flexible [builtin system](/interpret/src/val.rs#L189) that lets you call external rust functions from anywhere in the code
- a couple of [basic builtins](/cli/src/std_builtins.rs) already

so on paper you can build anything you want really

## story

i kinda got bored and developing this was pretty fun i've made attempts to build programming languages in the past but somehow in a couple days i far surpassed any demo language i've built

it basically pretty much works which is really suprising \
but obviously there are many things that aren't implemented just yet and this language is in no way intended for real world production usage
but whatever is implemented is pretty cool

## see

[`fib_cond.dl`](/fib_cond.dl) is a recursive implementation of fibonacci

[`iter.dl`](/iter.dl) is an extensible iterator system (it'll feel familiar if you've used rust's)

[`fib_iter.dl`](/fib_iter.dl) is a fibonacci implementation on top of the iterator system, with a slow iterator using [the recursive fibonacci implementation](/fib_cond.dl), and a fast iterator that uses closures and two internal variables to calculate the next result

---

try out for yourself:

- clone this repository

```sh
cargo run -p cli
```

it'll open a repl (which will feel familiar if you ever used the python or node repl) and you can try the language out like that

or use:

```sh
cargo run -p cli path_to_file.dl
```

to run a standalone file
