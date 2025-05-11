# alpha

```rust
// examples/main.a
fn fib(n) {
    let result = if n <= 1 {
        n;
    } else {
        fib(n-1) + fib(n-2);
    };
    printf("fib(%d) = %d\n", n, result);
    result;
}

fib(8);
```

```shell
# interpret
cargo run -- -f examples/main.a -i

# compile and run
cargo run -- -f examples/main.a -r

# debug
cargo run -- -f examples/main.a -d
```
