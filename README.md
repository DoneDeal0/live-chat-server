# LIVE CHAT SERVER

Live chat server in rust for a hackaton project.

## INSTALLATION

You need to install Rust and Cargo:

```bash
curl https://sh.rustup.rs -sSf | sh
```

## TEST

```bash
cargo test
```

## RUN

```bash
cargo run
```

## BUILD

```bash
cargo build
```

<hr/>

### CAVEATS

- Some events can be broadcasted back to the sender. You may need to filter your own data on the frontend based on the `user_id`.

- This project is not secure. It doesn't allow requests from all origins and doesn't authenticate requests with a cookie. It was just for a hackathon. I may improve it later.
