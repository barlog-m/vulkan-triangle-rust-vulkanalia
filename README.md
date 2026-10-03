# Rust empty template project

### make SDL_Init fail
```
SDL_VIDEO_DRIVER=bogus cargo run
```

### cargo test

```
cargo test -- --color always --nocapture
```

### cargo watch

```
cargo watch -x test
cargo watch -x check -x test
```
