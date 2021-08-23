# WASM Rust

- Windows WSL used

- install rust & wasm : https://antweiss.com/blog/extending-envoy-with-wasm-and-rust/

- examples : https://github.com/proxy-wasm/proxy-wasm-rust-sdk/tree/master/examples

## Set up wasm filter

`cargo new --lib pekerustallround1`

`cd pekerustallround1`

`code .`  (Visual Studio Code)

change **Cargo.toml**

```
[lib]
crate-type = ["cdylib"]
path = "src/lib.rs"

[dependencies]
proxy-wasm = "0.1.3"
log = "0.4.8"
serde_json = "1.0"

```

change **src/lib.rs** to implement your code


## LOCAL Testing (vs Istio envoy proxy)

- compile to wasm : `cargo build --target wasm32-unknown-unknown --release`

- `docker-compose up`

    - `curl -v -H "info: my little secret" http://localhost:8080/anything`

    - "secret" will be replaced with "open" in response, transponder configurations will be shown

    - json from config is parsed and value of "extraUrlsToMask" is shown in header info "Peke-Extra-Url-Mask" response

    - json config parsing is also used to get intermediate svc definition

    - call to intermediate service is done and value of result is upstreamed in header "Peke-First-Byte-Check"

- `docker-compose down`

## ISTIO Testing

- install httpbin : `kubectl apply -f httpbin.yaml`
- install peke-echo : `kubectl apply -f peke-echo.yaml`

- build and tag image : `wasme build precompiled ./target/wasm32-unknown-unknown/release/pekerustallround1.wasm -t webassemblyhub.io/peke/pekerustallround:v0.1`
- verify image : `wasme list`

- push image :
    - `wasme login -u Peke`
    - `wasme push webassemblyhub.io/peke/pekerustallround:v0.1`

- install wasme operator/cache 0.0.32
(if startup hangs, remove all FilterDeployments and reinstall wasme operator)

- install filterDeployment for peke-echo : `kubectl apply -f peke-echo-v1-pekerustallround.yaml`

- if filter doens't get pulled (see logs of wasme operator) : delete (and restart) wasme operator pod




