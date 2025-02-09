# External Function Interface for Cait Sith (MPC) 
Go Wrapper for the Rust Library [cait-sith](https://docs.rs/cait-sith/latest/cait_sith/).
This repository is related to MPC research efforts at Chainsafe Systems.

> [!WARNING]
> This library is under active development. Use at your own risk 
> and only if knowledgeable about cait-sith, Rust, MPC and Go!
> NEVER RE-USE TRIPLES and NEVER RE-USE PRESIGNS!

> [!WARNING]
> Last updated: 09.02.2025.
> Implemented tests for generating a multi party signature
> Generate keys for parties, have parties generate triples,
> run presign, run sign
>
> Some work tbd to make this production ready:
> split the code up so that it can be run by parties independently,
> rather than simulate an interaction.


# Try the EXT FFI

```bash
go run main.go
```

To compile changes to `external.rs` or other files in the crate:

```bash
cd cait-sith && cargo build --release
```
