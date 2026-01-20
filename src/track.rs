// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::hash::Hash;

use maplike::Get;

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

pub struct HalfEdgesCounter {
    map: BTreeMap<usize, usize>,
}

impl HalfEdgesCounter {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn visit_face_edges<
        VW,
        HEW,
        FW,
        VC,
        HEC: Get<usize, Value = HalfEdge<HEW>>,
        FC: Get<usize, Value = Face<FW>>,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
        face: FaceId,
    ) {
        for edge in dcel.face_edges(face) {
            self.visit_edge(edge);
        }
    }

    pub fn visit_face_half_edges<
        VW,
        HEW,
        FW,
        VC,
        HEC: Get<usize, Value = HalfEdge<HEW>>,
        FC: Get<usize, Value = Face<FW>>,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
        face: FaceId,
    ) {
        for half_edge in dcel.face_half_edges(face) {
            self.visit_half_edge(half_edge);
        }
    }

    pub fn visit_half_edge(&mut self, half_edge: HalfEdgeId) {
        *self.map.entry(half_edge.id()).or_default() += 1;
    }

    pub fn visit_edge(&mut self, edge: EdgeId) {
        self.visit_half_edge(edge.forward());
        self.visit_half_edge(edge.backward());
    }

    pub fn visited_half_edges(&self) -> impl Iterator<Item = HalfEdgeId> {
        self.map.keys().map(|key| HalfEdgeId(*key))
    }

    pub fn visited_half_edge_counts(&self) -> impl Iterator<Item = (HalfEdgeId, usize)> {
        self.map
            .iter()
            .map(|(key, value)| (HalfEdgeId(*key), *value))
    }

    pub fn visited_edges<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> impl Iterator<Item = EdgeId> {
        let mut deduplicating_set = BTreeSet::new();

        self.visited_half_edges().filter_map(move |half_edge| {
            if deduplicating_set.contains(&half_edge.id()) {
                return None;
            }

            deduplicating_set.insert(half_edge.id());
            Some(dcel.full_edge(half_edge))
        })
    }

    pub fn inner_edges<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> impl Iterator<Item = EdgeId> {
        self.visited_edges::<VW, HEW, FW, VC, HEC, FC>(dcel)
            .filter(|edge| self.is_inner_edge(*edge))
    }

    pub fn outer_edges<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> impl Iterator<Item = EdgeId> {
        self.visited_edges::<VW, HEW, FW, VC, HEC, FC>(dcel)
            .filter(|edge| self.is_outer_edge(*edge))
    }

    pub fn edge_visit_count(&self, edge: EdgeId) -> usize {
        self.half_edge_visit_count(edge.forward()) + self.half_edge_visit_count(edge.backward())
    }

    pub fn half_edge_visit_count(&self, half_edge: HalfEdgeId) -> usize {
        self.map.get(&half_edge.id()).copied().unwrap_or_default()
    }

    pub fn is_inner_edge(&self, edge: EdgeId) -> bool {
        self.half_edge_visit_count(edge.forward()) <= 0
            || self.half_edge_visit_count(edge.backward()) <= 0
    }

    pub fn is_outer_edge(&self, edge: EdgeId) -> bool {
        self.half_edge_visit_count(edge.forward()) >= 1
            && self.half_edge_visit_count(edge.backward()) >= 1
    }
}

pub struct VertexesCounter {
    map: BTreeMap<usize, usize>,
}

impl VertexesCounter {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn visit_face_vertexes<
        VW,
        HEW,
        FW,
        VC: Get<usize, Value = Vertex<VW>>,
        HEC: Get<usize, Value = HalfEdge<HEW>>,
        FC: Get<usize, Value = Face<FW>>,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
        face: FaceId,
    ) {
        for vertex in dcel.face_vertexes(face) {
            self.visit_vertex(vertex);
        }
    }

    pub fn visit_vertex(&mut self, vertex: VertexId) {
        *self.map.entry(vertex.id()).or_insert(0) += 1;
    }

    pub fn visited_vertexes(&self) -> impl Iterator<Item = VertexId> {
        self.map.keys().map(|&id| VertexId(id))
    }
}

pub struct VertexTracker<VW> {
    map: HashMap<VW, VertexId>,
}

impl<VW: Eq + Hash> VertexTracker<VW> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn visit_vertex(&mut self, weight: VW, vertex: VertexId) {
        self.map.insert(weight, vertex);
    }

    pub fn vertex(&mut self, weight: VW) -> Option<VertexId> {
        self.map.get(&weight).cloned()
    }
}

pub struct EdgesTracker {
    map: BTreeMap<(usize, usize), usize>,
}

impl EdgesTracker {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }

    pub fn visit_vertexes_edge(&mut self, from: VertexId, to: VertexId, half_edge: HalfEdgeId) {
        self.map.insert((from.id(), to.id()), half_edge.id());
    }

    pub fn vertexes_half_edge(&self, from: VertexId, to: VertexId) -> Option<HalfEdgeId> {
        self.map
            .get(&(from.id(), to.id()))
            .map(|id| HalfEdgeId(*id))
    }
}
