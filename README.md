You can view, follow and analyze my work for this and [chair23](https://github.com/vnikonov63/chair23) projects at this [Trello Board](https://trello.com/invite/b/68f1e17cc8973c8fdf027799/ATTIcd69c353aabd81e0c64072ee9248f52970CEABD0/chair22-23)

You can try out my demo on this [website](https://chair22-web.onrender.com/)

## Quick Start
- You can start the server with `cargo run -p server`
- You can start the frontend with `cd client && npm run`
- You can interact with the compiler in the command line with `cargo run -p cli`
- You can run the tests for the compiler with `cd cli && cargo test`

## Features 

## Concrete Syntax

This is the concrete syntax for the input language of my compiler:
```
<expr> :=
  | <number>
  | true
  | false
  | input
  | <identifier>
  | (let (<binding>+) <expr>)
  | (<op1> <expr>)
  | (<op2> <expr> <expr>)
  | (set! <name> <expr>)
  | (if <expr> <expr> <expr>)
  | (block <expr>+)
  | (loop <expr>)
  | (break <expr>)

<op1> := add1 | sub1 | isnum | isbool
<op2> := + | - | * | < | > | >= | <= | =

<binding> := (<identifier> <expr>)
```

## Code You Can Run
**1. First Fibonacci Sequence Element with a Given Divisor**
```Racket
(let ((divisor 25) (index 0) (maximum 50) (prev 0) (curr 1) (temp1 0) (temp2 0))
    (if (<= divisor 0)
        -1
        (loop
            (if (<= index maximum)
                (block
                    (set! temp1 curr)
                    (loop
                        (if (> temp1 0)
                            (set! temp1 (- temp1 divisor))
                            (break temp1)
                        )
                    )
                    (if (= temp1 0)
                        (break curr)
                        (block
                            (set! temp2 curr)
                            (set! curr (+ curr prev))
                            (set! prev temp2)
                            (set! index (add1 index))
                        )
                    )
                )
                (break -1)
            )
        )
    )
)
```

