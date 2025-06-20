# 🔐 r_ssz

[![CI](https://github.com/Soubhik-10/r_ssz/actions/workflows/rust.yml/badge.svg)](https://github.com/Soubhik-10/r_ssz/actions)
[![CI](https://github.com/Soubhik-10/r_ssz/actions/workflows/lint.yml/badge.svg)](https://github.com/Soubhik-10/r_ssz/actions)
[![CI](https://github.com/Soubhik-10/r_ssz/actions/workflows/nostd.yml/badge.svg)](https://github.com/Soubhik-10/r_ssz/actions)
[![CI](https://github.com/Soubhik-10/r_ssz/actions/workflows/miri.yml/badge.svg)](https://github.com/Soubhik-10/r_ssz/actions)

A minimal, readable implementation of Ethereum's [SSZ] in Rust. Built for experimentation, testing, and learning.

---

## ✨ Features

- SSZ serialization/deserialization for primitive and composite types
- Full `hash_tree_root` Merkleization support
- Supports `BitList`, `BitVector`, `List`, `Vector`, `Option`, `Union` and `Container`
- Minimal dependencies
- `no-std` support

---

## ✅ Test Coverage

**Note:**  
All implementations have been tested by
[@Rimeeeeee](https://github.com/Rimeeeeee) and [@Soubhik-10](https://github.com/Soubhik-10)
using [`@chainsafe/ssz`](https://github.com/ChainSafe/ssz/tree/master/packages/ssz) package.

---

## 👥 Contributors

[![Contributors](https://contrib.rocks/image?repo=Soubhik-10/r_ssz&cache-bust=20250612)](https://github.com/Soubhik-10/r_ssz/graphs/contributors)

👉 See the [full contributor list](https://github.com/Soubhik-10/r_ssz/graphs/contributors).

---

## ⚠️ Warning

> ⚠️ **This project is intended for experimentation and learning purposes.**  
> It is **not production-ready** (Maybe it is … you use it **at your own risk**).

---

## 🧪 Getting Started

Add to `Cargo.toml`:

```toml
r_ssz = "0.1.2"


```

### Experimental features

| EIP                                                  | Status                 |
| ---------------------------------------------------- | ---------------------- |
| [EIP-7495](https://eips.ethereum.org/EIPS/eip-7495>) | Tested and Implemented |
| [EIP-7916](https://eips.ethereum.org/EIPS/eip-7916>) | Tested and Implemented |
| [EIP-7688](https://eips.ethereum.org/EIPS/eip-7688>) | In Progress            |

---

### For now we are posponing this release as these are extremely experimental and need to be more tested. Even the EIPs are not finalized and by the look of how things are going Pureth and these EIPs are not going to be implemented anytime sooner than Amsterdam EL fork. We will try to release these along with other related EIPs and the completed version of this crate sometimes arounf the Fusaka release. Thanks for reading this. Have a great day :)
