# fga-rs

wip(breaking changes at any time)

## what is fga-rs

fga-rs is a Fine Grained Authorization written in rust, such as openfga, permify, spicedb and so on.

fga-rs is a permission server for real time, enterprise application permissions inspired by Google Zanzibar.

## why fga-rs

1. written in rust
2. support superadmin
3. support fast condition eval(wip)

## Getting Started

### required

1. [rust](https://www.rust-lang.org/)
2. [postgres](https://postgresql.org/)

### init db(wip)

```shell
psql -d fga-rs -a -f init.sql
```

### start

```shell
cargo run -p cli -- server -d postgres://postgres@127.0.0.1:5432/fga-rs
```

## TODO

- [x] schema: design(such as model.authz), parser(use lalrpop)
- [x] schema: support permission: union(+ |), intersection(&), exclude(-), priority
- [ ] support condition(expr eval)
- [x] remote checker
- [x] grpc
- [x] refactor check request
- [x] expand tuple
- [x] opentelemetry
- [ ] config file use toml
- [ ] migration database
- [ ] unit test
- [ ] e2e test
- [ ] playground
- [ ] vscode-extension
- [ ] lsp
- [ ] sdk (priority: rust, java, go, js, python, ruby...)

## Credit

1. [openfga](https://github.com/openfga/openfga)
2. [permify](https://github.com/Permify/permify)
3. [spicedb](https://github.com/authzed/spicedb)
