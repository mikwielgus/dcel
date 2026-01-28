// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::Get;

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertexes(&self) -> &VC {
        &self.vertexes
    }

    #[inline]
    pub fn half_edges(&self) -> &HEC {
        &self.half_edges
    }

    #[inline]
    pub fn faces(&self) -> &FC {
        &self.faces
    }

    #[inline]
    pub fn dissolve(self) -> (VC, HEC, FC) {
        (self.vertexes, self.half_edges, self.faces)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn outgoing_next_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.vertexes
            .get(&vertex.id())
            .unwrap()
            .outgoing_next_half_edge
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn incoming_next_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.twin(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    pub fn vertex_next_edge(&self, vertex: VertexId) -> EdgeId {
        EdgeId::new(
            self.outgoing_next_half_edge(vertex),
            self.incoming_next_half_edge(vertex),
        )
    }

    #[inline]
    pub fn incoming_prev_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.prev_half_edge(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    pub fn outgoing_prev_half_edge(&self, vertex: VertexId) -> HalfEdgeId {
        self.twin(self.incoming_prev_half_edge(vertex))
    }

    #[inline]
    pub fn vertex_prev_edge(&self, vertex: VertexId) -> EdgeId {
        EdgeId::new(
            self.incoming_prev_half_edge(vertex),
            self.outgoing_prev_half_edge(vertex),
        )
    }

    #[inline]
    pub fn vertex_inner_outgoing_half_edge(&self, vertex: VertexId, face: FaceId) -> HalfEdgeId {
        self.vertex_half_spokes(vertex)
            .find(|&half_edge| self.face_in_front(half_edge) == face)
            .unwrap()
    }

    #[inline]
    pub fn vertex_inner_incoming_half_edge(&self, vertex: VertexId, face: FaceId) -> HalfEdgeId {
        let outgoing = self.vertex_inner_outgoing_half_edge(vertex, face);
        self.prev_half_edge(outgoing)
    }

    #[inline]
    pub fn vertexes_common_face(&self, vertex1: VertexId, vertex2: VertexId) -> Option<FaceId> {
        let interspokes1: Vec<FaceId> = self.vertex_interspokes(vertex1).collect();

        self.vertex_interspokes(vertex2)
            .find(|interspoke| interspokes1.contains(interspoke))
    }

    #[inline]
    pub fn is_boundary_vertex(&self, vertex: VertexId) -> bool {
        self.vertex_spokes(vertex).any(|spoke| {
            self.face_in_front(spoke.forward()) == self.unbounded_face()
                || self.face_in_front(spoke.backward()) == self.unbounded_face()
        })
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertex_weight(&self, vertex: VertexId) -> &VW {
        &self.vertexes.get(&vertex.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn origin(&self, half_edge: HalfEdgeId) -> VertexId {
        self.half_edges.get(&half_edge.id()).unwrap().origin
    }

    #[inline]
    pub fn endpoints(&self, edge: EdgeId) -> (VertexId, VertexId) {
        (self.origin(edge.forward()), self.origin(edge.backward()))
    }

    #[inline]
    pub fn twin(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().twin
    }

    #[inline]
    pub fn full_edge(&self, half_edge: HalfEdgeId) -> EdgeId {
        EdgeId::new(half_edge, self.twin(half_edge))
    }

    #[inline]
    pub fn reverse_edge(&self, edge: EdgeId) -> EdgeId {
        EdgeId::new(edge.backward(), edge.forward())
    }

    #[inline]
    pub fn face_in_front(&self, half_edge: HalfEdgeId) -> FaceId {
        self.half_edges.get(&half_edge.id()).unwrap().face
    }

    #[inline]
    pub fn face_behind(&self, half_edge: HalfEdgeId) -> FaceId {
        self.face_in_front(self.twin(half_edge))
    }

    #[inline]
    pub fn edge_faces(&self, edge: EdgeId) -> (FaceId, FaceId) {
        (
            self.face_in_front(edge.forward()),
            self.face_behind(edge.backward()),
        )
    }

    #[inline]
    pub fn prev_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().prev
    }

    #[inline]
    pub fn next_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().next
    }

    #[inline]
    pub fn prev_edge(&self, edge: EdgeId) -> EdgeId {
        let next_forward_half_edge = self.half_edges.get(&edge.forward().id()).unwrap().prev;
        let next_backward_half_edge = self
            .half_edges
            .get(&next_forward_half_edge.id())
            .unwrap()
            .twin;

        EdgeId::new(next_forward_half_edge, next_backward_half_edge)
    }

    #[inline]
    pub fn next_edge(&self, edge: EdgeId) -> EdgeId {
        let next_forward_half_edge = self.half_edges.get(&edge.forward().id()).unwrap().next;
        let next_backward_half_edge = self
            .half_edges
            .get(&next_forward_half_edge.id())
            .unwrap()
            .twin;

        EdgeId::new(next_forward_half_edge, next_backward_half_edge)
    }

    #[inline]
    pub fn turn_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.next_half_edge(self.twin(half_edge))
    }

    #[inline]
    pub fn turn_back_half_edge(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.twin(self.prev_half_edge(half_edge))
    }

    #[inline]
    pub fn turn_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.turn_half_edge(edge.forward()))
    }

    #[inline]
    pub fn turn_back_edge(&self, edge: EdgeId) -> EdgeId {
        self.full_edge(self.turn_back_half_edge(edge.forward()))
    }

    #[inline]
    pub fn half_edge_weight(&self, half_edge: HalfEdgeId) -> &HEW {
        &self.half_edges.get(&half_edge.id()).unwrap().weight
    }

    #[inline]
    pub fn edge_weights(&self, edge: EdgeId) -> (&HEW, &HEW) {
        (
            self.half_edge_weight(edge.forward()),
            self.half_edge_weight(edge.backward()),
        )
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Value = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn incident_half_edge(&self, face: FaceId) -> Option<HalfEdgeId> {
        self.faces.get(&face.id()).unwrap().incident_half_edge
    }

    #[inline]
    pub fn face_weight(&self, face: FaceId) -> &FW {
        &self.faces.get(&face.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the id of the unbounded face.
    ///
    /// The unbounded face is always the first element of the face list.
    #[inline]
    pub fn unbounded_face(&self) -> FaceId {
        FaceId(0)
    }
}
