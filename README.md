# `dynlang`

###### don't get caught up on the name it's just as unserious as it sounds

only interpreted for now because that's easier

## currently implemented

- strings
- numbers (i32 & f32)
- functions with input & output args
- variables
- context (variables defined at a smaller scope will cease to exist once that scope is left)[*](#closures)
- boolean logic (<, >, ==, ||, &&)
- conditional execution & conditional expressions

## theoretically supported but not implemented

- arrays (it's in the language and a value can be an array there's just no way to define or index one)

## should be supported and implemented

- finally decouple what the interpreter thinks is a function and `langlib::Function`[*](#closures) \
this would allow us to define builtin functions and shit which is pretty useful (and we could start doing some more interesting stuff)

- order of operations (you can manually order your operations with parens in the meanwhile but ehhh still)

## see

[fib_cond.dl](/fib_cond.dl) is a recursive implementation of a fibonacci generator

[iter.dl](/iter.dl) is an extensible iterator system \
that's the most impressive for now

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

### closures

closures don't really work if they capture variables because we only pass the function as a value and we don't package any other information with it (like the variables it's using from outer scopes)

okay the way to this probably is to introduce a new Value variant that's just a function and the context it needs but i kinda need to rethink how contexts should even work cause right now [functions have access to variables they shouldn't even know about](/contexts-are-broken.dl)
