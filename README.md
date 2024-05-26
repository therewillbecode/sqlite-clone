
first install  `https://github.com/lu-zero/cargo-c`

then compile c bindings with 
```cargo cbuild```


export PKG_CONFIG_PATH=<path>/rust-wrapper/target/<arch>/debug
export PKG_CONFIG_PATH=./rust-wrapper/target/aarch64-apple-darwin/debug

export LD_LIBRARY_PATH=<path>/rust-wrapper/target/<arch>/debug

export LD_LIBRARY_PATH=./rust-wrapper/target/aarch64-apple-darwin/debug

