# evm_inspector

# Set the build profile to be release

```
RUSTFLAGS='-C target-cpu=native' cargo build --profile maxperf --target x86_64-unknown-linux-gnu
```

note: we have to add the -Z build-std later
