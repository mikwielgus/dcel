// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::{Dcel, Face, HalfEdge, Vertex};

pub type StableDcel<VW, HEW = (), FW = ()> = Dcel<
    VW,
    HEW,
    FW,
    stable_vec::StableVec<Vertex<VW>>,
    stable_vec::StableVec<HalfEdge<HEW>>,
    stable_vec::StableVec<Face<FW>>,
>;
