// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::BTreeSet;

use maplike::{Get, Insert, Remove};

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Remove<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Remove<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Remove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces_around_vertex(&mut self, inner_vertex: VertexId) {
        let absorbing_face = self.face_in_front(
            self.vertexes
                .get(&inner_vertex.id())
                .unwrap()
                .outgoing_next_half_edge,
        );
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self
            .vertexes
            .get(&inner_vertex.id())
            .unwrap()
            .outgoing_next_half_edge;
        let initial_edge = self.full_edge(
            self.vertexes
                .get(&inner_vertex.id())
                .unwrap()
                .outgoing_next_half_edge,
        );
        let inner_edges: Vec<EdgeId> = self.cw_edges(initial_edge).collect();
        let perimeter_edges: Vec<EdgeId> = self
            .edges_with_excludes(initial_edge, inner_edges.clone())
            .collect();

        self.remove_faces(
            self.cw_faces(self.face_in_front(initial_half_edge))
                .collect::<Vec<FaceId>>(),
        );
        self.remove_edges(inner_edges);
        self.remove_vertex(inner_vertex);

        self.wire_face_edges_vertexes(absorbing_face, &perimeter_edges);
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Remove<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Remove<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Remove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces(absorbing_face, faces);
    }

    pub fn merge_faces_over_edges_and_vertexes(
        &mut self,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces_over_edges_and_vertexes(absorbing_face, faces, edges, vertexes);
    }

    pub fn absorb_faces(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
    ) {
        let mut visited_half_edges = BTreeSet::new();
        let mut visited_vertexes = BTreeSet::new();

        self.record_occurrences_in_face(
            &mut visited_half_edges,
            &mut visited_vertexes,
            absorbing_face,
        );

        for face in faces {
            self.record_occurrences_in_face(&mut visited_half_edges, &mut visited_vertexes, face);
        }

        self.remove_vertexes(
            visited_vertexes
                .iter()
                .filter(|&&vertex| {
                    self.cw_edges(self.vertex_next_edge(VertexId(vertex)))
                        .all(|edge| {
                            visited_half_edges.contains(&edge.forward().id())
                                && visited_half_edges.contains(&edge.backward().id())
                        })
                })
                .map(|vertex| VertexId(*vertex))
                // PERF: Needless collect?
                .collect::<Vec<VertexId>>(),
        );

        self.remove_edges(
            visited_half_edges
                .iter()
                .map(|&half_edge| self.full_edge(HalfEdgeId(half_edge)))
                .filter(|&edge| self.is_inner_edge(&visited_half_edges, edge))
                .collect::<Vec<EdgeId>>(),
        );

        self.wire_face_edges_vertexes(
            absorbing_face,
            &visited_half_edges
                .iter()
                .map(|&half_edge| self.full_edge(HalfEdgeId(half_edge)))
                .filter(|&edge| self.is_outer_edge(&visited_half_edges, edge))
                .collect::<Vec<_>>(),
        );
    }

    fn record_occurrences_in_face(
        &self,
        visited_half_edges: &mut BTreeSet<usize>,
        visited_vertexes: &mut BTreeSet<usize>,
        face: FaceId,
    ) {
        for edge in self.face_edges(face) {
            visited_half_edges.insert(edge.forward().id());
            visited_vertexes.insert(self.origin(edge.forward()).id());
        }
    }

    fn is_inner_edge(&self, visited_half_edges: &BTreeSet<usize>, edge: EdgeId) -> bool {
        visited_half_edges.contains(&edge.forward().id())
            && visited_half_edges.contains(&edge.backward().id())
    }

    fn is_outer_edge(&self, visited_half_edges: &BTreeSet<usize>, edge: EdgeId) -> bool {
        !visited_half_edges.contains(&edge.forward().id())
            || !visited_half_edges.contains(&edge.backward().id())
    }

    pub fn absorb_faces_over_edges_and_vertexes(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let edges: Vec<EdgeId> = edges.into_iter().collect();
        let perimeter_edges: Vec<EdgeId> = self
            .edges_with_excludes(
                self.full_edge(
                    self.faces
                        .get(&absorbing_face.id())
                        .unwrap()
                        .incident_half_edge
                        .unwrap(),
                ),
                edges.clone(),
            )
            .collect();

        self.remove_faces(faces);
        self.remove_edges(edges);
        self.remove_vertexes(vertexes);

        self.wire_face_edges_vertexes(absorbing_face, &perimeter_edges);
    }
}
