// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::Get;

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns an immutable reference to the collection holding the data of
    /// the vertices.
    #[inline]
    pub fn vertices(&self) -> &VC {
        &self.vertices
    }

    /// Returns an immutable reference to the collection holding the data of
    /// the half-edges.
    #[inline]
    pub fn half_edges(&self) -> &HEC {
        &self.half_edges
    }

    /// Returns an immutable reference to the collection holding the data of
    /// the faces.
    #[inline]
    pub fn faces(&self) -> &FC {
        &self.faces
    }

    /// Dissolve the DCEL, ceding ownership and returning the datas of its
    /// vertices, half-edges, and faces.
    #[inline]
    pub fn dissolve(self) -> (VC, HEC, FC) {
        (self.vertices, self.half_edges, self.faces)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the vertex's representative half-edge.
    ///
    /// This is always an outgoing half-edge, never an incoming one. That is,
    /// the vertex is always the source (origin) vertex of its representative
    /// half-edge, never the target vertex.
    ///
    /// The representative half-edge of a vertex is one of its two outgoing
    /// half-edges that is stored in the vertex datum to make it possible to use
    /// the vertex id to access the vertex's surroundings.
    ///
    /// In DCEL terminology, the representative half-edge of a vertex is
    /// usually just called "the incident half-edge", but we prefer to call it
    /// differently to distinguish it from the other incident half-edges of the
    /// same vertex.
    #[inline]
    pub fn vertex_representative(&self, vertex: VertexId) -> HalfEdgeId {
        self.vertices.get(&vertex.id()).unwrap().representative
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Find the half-spoke that is also incident to a given face.
    #[inline]
    pub(crate) fn find_incident_half_spoke(&self, vertex: VertexId, face: FaceId) -> HalfEdgeId {
        self.vertex_half_spokes(vertex)
            .find(|&half_edge| self.incident_face(half_edge) == face)
            .unwrap()
    }

    /// Find the half-spoke that is also incident to a given face and return the
    /// half-edge that is previous in the circulation around the same incident
    /// face.
    #[inline]
    pub(crate) fn find_prev_incident_half_spoke(
        &self,
        vertex: VertexId,
        face: FaceId,
    ) -> HalfEdgeId {
        let outgoing = self.find_incident_half_spoke(vertex, face);
        self.prev(outgoing)
    }

    /// Find the half-edge from `source` to `target`, if there is one.
    ///
    /// This is done by iterating over all the half-spokes of `source`, looking
    /// for one whose target is `target`.
    #[inline]
    pub fn find_half_edge_from_to(&self, source: VertexId, target: VertexId) -> Option<HalfEdgeId> {
        self.vertex_half_spokes(source)
            .find(|&half_edge| self.source(self.twin(half_edge)) == target)
    }

    /// Find the edge between two vertices, if there is one.
    ///
    /// This is done by iterating over all the half-spokes of `source`, looking
    /// for one whose target is `target`, and upgrading it to a full edge if
    /// found.
    #[inline]
    pub fn find_edge_between(&self, source: VertexId, target: VertexId) -> Option<EdgeId> {
        Some(self.full_edge(self.find_half_edge_from_to(source, target)?))
    }

    /// Find the common face between two vertices, if there is one.
    ///
    /// This is done by iterating over the interspokes of both vertices, looking
    /// for a shared one.
    #[inline]
    pub fn find_vertices_common_face(
        &self,
        vertex1: VertexId,
        vertex2: VertexId,
    ) -> Option<FaceId> {
        let interspokes1: Vec<FaceId> = self.vertex_interspokes(vertex1).collect();

        self.vertex_interspokes(vertex2)
            .find(|interspoke| interspokes1.contains(interspoke))
    }

    /// Check if the vertex lies on the DCEL's boundary.
    ///
    /// This is determined by iterating over the vertex spokes and checking if
    /// any of them borders the unbounded face.
    #[inline]
    pub fn check_if_boundary_vertex(&self, vertex: VertexId) -> bool {
        self.vertex_spokes(vertex).any(|spoke| {
            self.incident_face(spoke.lesser()) == self.unbounded_face()
                || self.incident_face(spoke.greater()) == self.unbounded_face()
        })
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the weight of the vertex.
    #[inline]
    pub fn vertex_weight(&self, vertex: VertexId) -> &VW {
        &self.vertices.get(&vertex.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the source (origin) vertex of the half-edge.
    ///
    /// In DCEL terminology, this vertex is usually called the "origin".
    /// However, here we have opted to use terms from graph theory, where the
    /// terms "source vertex" (for this vertex) and "target vertex" (for the
    /// vertex on the other end) are used instead.
    #[inline]
    pub fn source(&self, half_edge: HalfEdgeId) -> VertexId {
        self.half_edges.get(&half_edge.id()).unwrap().source
    }

    /// Returns the target vertex of the half-edge.
    ///
    /// This is the same as the source vertex of the next half-edge.
    #[inline]
    pub fn target(&self, half_edge: HalfEdgeId) -> VertexId {
        self.source(self.next(half_edge))
    }

    /// Returns the twin (opposite) of the half-edge.
    #[inline]
    pub fn twin(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().twin
    }

    /// Returns the edge the half-edge is half of.
    #[inline]
    pub fn full_edge(&self, half_edge: HalfEdgeId) -> EdgeId {
        EdgeId::new(half_edge, self.twin(half_edge))
    }

    /// Returns the indicent face of the half-edge.
    #[inline]
    pub fn incident_face(&self, half_edge: HalfEdgeId) -> FaceId {
        self.half_edges.get(&half_edge.id()).unwrap().face
    }

    /// Returns the opposite face of the half-edge.
    ///
    /// This is the same as the incident face of the twin half-edge.
    #[inline]
    pub fn opposite_face(&self, half_edge: HalfEdgeId) -> FaceId {
        self.incident_face(self.twin(half_edge))
    }

    /// Returns the endpoint vertices of the edge.
    #[inline]
    pub fn edge_endpoints(&self, edge: EdgeId) -> (VertexId, VertexId) {
        (self.source(edge.lesser()), self.source(edge.greater()))
    }

    /// Returns the pair of faces adjacent to the edge: the incident face and
    /// the opposite face.
    #[inline]
    pub fn edge_faces(&self, edge: EdgeId) -> (FaceId, FaceId) {
        (
            self.incident_face(edge.lesser()),
            self.opposite_face(edge.greater()),
        )
    }

    /// Returns the previous half-edge in the circulation around the same
    /// incident face.
    #[inline]
    pub fn prev(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().prev
    }

    /// Returns the next half-edge in the circulation around the same incident
    /// face.
    #[inline]
    pub fn next(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.half_edges.get(&half_edge.id()).unwrap().next
    }

    /// Returns the half-edge that follows in the rotation around the same
    /// source vertex.
    ///
    /// This is the same as the twin of the previous half-edge.
    #[inline]
    pub fn turn_back(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.twin(self.prev(half_edge))
    }

    /// Returns the half-edge that precedes in the rotation around the same
    /// source vertex.
    ///
    /// This is the same as the next half-edge of the twin half-edge.
    #[inline]
    pub fn turn(&self, half_edge: HalfEdgeId) -> HalfEdgeId {
        self.next(self.twin(half_edge))
    }

    /// Returns the weight of the half-edge.
    #[inline]
    pub fn half_edge_weight(&self, half_edge: HalfEdgeId) -> &HEW {
        &self.half_edges.get(&half_edge.id()).unwrap().weight
    }

    /// Returns the pair of half-edge weights of the edge, ascendingly sorted by
    /// half-edge indices.
    #[inline]
    pub fn edge_weights(&self, edge: EdgeId) -> (&HEW, &HEW) {
        (
            self.half_edge_weight(edge.lesser()),
            self.half_edge_weight(edge.greater()),
        )
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Value = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the face's representative half-edge.
    ///
    /// The representative half-edge of a face is one of incident half-edges
    /// that is stored in the face datum to make it possible to use the face id
    /// to access the face's surroundings.
    ///
    /// In DCEL terminology, the representative half-edge of a face is
    /// usually just called "the incident half-edge", but we prefer to call it
    /// differently to distinguish it from the other incident half-edges of the
    /// same face.
    #[inline]
    pub fn face_representative(&self, face: FaceId) -> Option<HalfEdgeId> {
        self.faces.get(&face.id()).unwrap().representative
    }

    /// Returns the weight of the face.
    #[inline]
    pub fn face_weight(&self, face: FaceId) -> &FW {
        &self.faces.get(&face.id()).unwrap().weight
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    /// Returns the unbounded face.
    ///
    /// The unbounded face is always the first element of the face list.
    #[inline]
    pub fn unbounded_face(&self) -> FaceId {
        FaceId(0)
    }
}
