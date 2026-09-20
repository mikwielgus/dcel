// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use stable_vec::StableVec;

use crate::{Dcel, Face, HalfEdge, Vertex};

/// A [Dcel] whose vertex, half-edge, and face collections are `StableVec`.
pub type StableDcel<VW, HEW = (), FW = ()> =
    Dcel<VW, HEW, FW, StableVec<Vertex<VW>>, StableVec<HalfEdge<HEW>>, StableVec<Face<FW>>>;

/// A [StableDcel] whose collections record undoredoable deltas.
#[cfg(feature = "undoredo")]
pub type RecordingStableDcel<VW, HEW = (), FW = ()> = Dcel<
    VW,
    HEW,
    FW,
    undoredo::Recorder<StableVec<Vertex<VW>>>,
    undoredo::Recorder<StableVec<HalfEdge<HEW>>>,
    undoredo::Recorder<StableVec<Face<FW>>>,
>;
