
# 🧬 ssz-rs

[![CI](https://github.com/Soubhik-10/ssz-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/Soubhik-10/ssz-rs/actions)
[![CI](https://github.com/Soubhik-10/ssz-rs/actions/workflows/lint.yml/badge.svg)](https://github.com/Soubhik-10/ssz-rs/actions)

A minimal, readable implementation of Ethereum's [SSZ] in Rust. Built for experimentation, testing, and learning.

---

## ✨ Features

- SSZ serialization/deserialization for primitive and composite types  
- Full `hash_tree_root` Merkleization support  
- Supports `BitList`, `BitVector`, `List`, `Vector`, `Option`, `Union`, and containers  
- Minimal dependencies
- `no-std` support

---

## ✅ Test Coverage

**Note:**  
All implementations have been manually tested by
[@Rimeeeeee](https://github.com/Rimeeeeee) and [@Soubhik-10](https://github.com/Soubhik-10)
against [`@chainsafe/ssz`](https://github.com/ChainSafe/ssz/tree/master/packages/ssz) fixtures.

---

## 👥 Contributors

[![Contributors](https://contrib.rocks/image?repo=Soubhik-10/ssz-rs)](https://github.com/Soubhik-10/ssz-rs/graphs/contributors)

👉 See the [full contributor list](https://github.com/Soubhik-10/ssz-rs/graphs/contributors).

---

## ⚠️ Warning

> ⚠️ **This project is intended for experimentation and learning purposes.**  
> It is **not production-ready** (Maybe it is … you use it **at your own risk**).

---

## 🧪 Getting Started

Add to `Cargo.toml`:

```toml
r_ssz = { git = "https://github.com/Soubhik-10/r_ssz" }


