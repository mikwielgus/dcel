<!--
SPDX-FileCopyrightText: 2026 dcel contributors

SPDX-License-Identifier: MIT OR Apache-2.0
-->

# dcel

Dynamic doubly-connected edge list (DCEL) in Rust.

A [DCEL](https://en.wikipedia.org/wiki/Doubly_connected_edge_list) is a
topological data structure that represents an embedding of a planar graph in the
plane. DCEL partitions a surface into easily and quickly traversable vertices,
edges, and faces, while making only minimal assumptions about their shapes. You
can freely choose wherever the vertices should have two or more dimensions, or
wheverer the edges are line segments or curved arcs.

In mathematics, particularly in [topological graph
theory](https://en.wikipedia.org/wiki/Topological_graph_theory) and [algebraic
graph theory](https://en.wikipedia.org/wiki/Algebraic_graph_theory), planar
graph embeddings in the plane are represented using concepts such as cellularly
embedded graph, [ribbon graph](https://ncatlab.org/nlab/show/ribbon+graph), band
decomposition, ram graph, arrow presentation, signed rotation system.

## Usage

### Adding dependency

First, add `dcel` as a dependency to your `Cargo.toml`

```toml
[dependencies]
dcel = "0.8"
```

## Documentation

See the [documentation](https://docs.rs/dcel/latest/dcel) for more information
about `dcel`'s usage.

## Packaging

`dcel` is published as a [crate](https://crates.io/crates/dcel) on the
[Crates.io](https://crates.io/) registry.

## Contributing

We welcome issues and pull requests from anyone to our canonical
[repository](https://codeberg.org/topola/dcel) on Codeberg.

## Licence

### Outbound licence

`dcel` is dual-licensed as under either of

- [MIT license](./LICENSES/MIT.txt),
- [Apache License, Version 2.0](./LICENSES/Apache-2.0.txt),

at your option.

### Inbound licence

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this work by you will be dual-licensed as described above,
without any additional terms or conditions.
