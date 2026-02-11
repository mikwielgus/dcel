// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, Remove, StableRemove};

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, Vertex, VertexId};

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + StableRemove<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + StableRemove<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + StableRemove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn remove_face(&mut self, face: FaceId) -> (Vec<EdgeId>, Vec<FaceId>) {
        let mut edges: Vec<EdgeId> = self.face_edges(face).collect();
        edges.extend(self.face_spokes(face));
        let interspokes: Vec<FaceId> = self.face_interspokes(face).collect();

        self.absorb_faces_over_edges_and_vertices(
            face,
            interspokes.clone(),
            edges.clone(),
            self.face_vertices(face).collect::<Vec<VertexId>>(),
        );

        (edges, interspokes)
    }

    pub fn remove_edge(&mut self, edge: EdgeId) -> FaceId {
        let absorbing_face = self.incident_face(edge.lesser());
        let face_to_absorb = self.opposite_face(edge.lesser());

        self.absorb_faces_over_edges_and_vertices(absorbing_face, [face_to_absorb], [edge], []);

        absorbing_face
    }

    pub fn remove_edge_chain(&mut self, edges: impl IntoIterator<Item = EdgeId>) -> FaceId {
        let edges: Vec<EdgeId> = edges.into_iter().collect();

        // XXX: Harden against empty iterators?
        let absorbing_face = self.incident_face(edges[0].lesser());
        let face_to_absorb = self.opposite_face(edges[0].lesser());

        self.absorb_faces_over_edges_and_vertices(absorbing_face, [face_to_absorb], edges, []);

        absorbing_face
    }
}

impl<VW, HEW, FW, VC: Remove<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    pub(crate) fn remove_orphaned_vertices(
        &mut self,
        vertices: impl IntoIterator<Item = VertexId>,
    ) {
        for vertex in vertices.into_iter() {
            self.remove_orphaned_vertex(vertex);
        }
    }

    pub(crate) fn remove_orphaned_vertex(&mut self, vertex: VertexId) {
        self.vertices.remove(&vertex.id());
    }
}

impl<VW, HEW, FW, VC, HEC: Remove<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub(crate) fn remove_orphaned_edges(&mut self, edges: impl IntoIterator<Item = EdgeId>) {
        for edge in edges.into_iter() {
            self.remove_orphaned_edge(edge);
        }
    }

    pub(crate) fn remove_orphaned_edge(&mut self, edge: EdgeId) {
        self.half_edges.remove(&edge.lesser().id());
        self.half_edges.remove(&edge.greater().id());
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Remove<usize>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    pub(crate) fn remove_orphaned_faces(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        for face in faces.into_iter() {
            self.remove_orphaned_face(face);
        }
    }

    pub(crate) fn remove_orphaned_face(&mut self, face: FaceId) {
        self.faces.remove(&face.id());
    }
}

// TODO: Tests.
