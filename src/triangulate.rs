// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, Push};

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, Vertex, VertexId};

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Partition a face into triangles by inserting a vertex inside and then
    /// adding edges between it and the original face's vertexes.
    ///
    /// The original face is reused for the first triangle. New faces are
    /// created for all the other triangles.
    ///
    /// Returns the new vertex id together with the face ids of all the new
    /// triangles.
    pub fn triangulate_around_vertex(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex_weight: VW,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        let perimeter_vertex_count = self.face_vertexes(perimeter_face).count();

        self.triangulate_around_vertex_with_all_weights(
            perimeter_face,
            inner_vertex_weight,
            std::iter::repeat_n(Default::default(), perimeter_vertex_count),
            std::iter::repeat_n(Default::default(), perimeter_vertex_count),
        )
    }

    pub fn fan_triangulate(
        &mut self,
        perimeter_face: FaceId,
        apex: VertexId,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        let perimeter_vertex_count = self.face_vertexes(perimeter_face).count();

        self.fan_triangulate_with_all_weights(
            perimeter_face,
            apex,
            std::iter::repeat_n(Default::default(), perimeter_vertex_count),
            std::iter::repeat_n(Default::default(), perimeter_vertex_count),
        )
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn triangulate_around_vertex_with_all_weights(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex_weight: VW,
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        let inner_vertex = self.add_unwired_vertex(inner_vertex_weight);
        self.fan_triangulate_with_all_weights(
            perimeter_face,
            inner_vertex,
            inner_edge_weights,
            triangle_face_weights,
        )
    }

    pub fn fan_triangulate_with_all_weights(
        &mut self,
        perimeter_face: FaceId,
        apex: VertexId,
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> (Vec<FaceId>, Vec<EdgeId>) {
        let new_faces = self.add_unwired_triangulation_faces(perimeter_face, triangle_face_weights);

        let mut triangle_faces = vec![perimeter_face];
        triangle_faces.extend(new_faces.clone());

        let new_edges = self.add_unwired_triangulation_edges(
            perimeter_face,
            apex,
            &triangle_faces,
            inner_edge_weights,
        );
        self.wire_triangulation_faces_edges_vertexes(perimeter_face, &new_edges);

        (new_faces, new_edges)
    }

    fn add_unwired_triangulation_faces(
        &mut self,
        first_face: FaceId,
        triangle_face_weights: impl IntoIterator<Item = FW>,
    ) -> Vec<FaceId> {
        let mut new_faces = vec![];

        let mut face_weights_iter = triangle_face_weights.into_iter();
        self.faces.insert(
            first_face.id(),
            Face {
                incident_half_edge: self.faces.get(&first_face.id()).unwrap().incident_half_edge,
                weight: face_weights_iter.next().unwrap(),
            },
        );

        for face_weight in face_weights_iter {
            new_faces.push(self.add_unwired_face(face_weight));
        }

        new_faces
    }

    fn add_unwired_triangulation_edges(
        &mut self,
        perimeter_face: FaceId,
        inner_vertex: VertexId,
        triangle_faces: &[FaceId],
        inner_edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let mut inner_edge_weights = inner_edge_weights.into_iter();
        let mut face_vertexes_walker = self.face_vertexes(perimeter_face).walker();
        let mut edges = vec![];

        let mut i: usize = 0;

        while let Some(perimeter_vertex) = face_vertexes_walker.next(self) {
            let (weight, twin_weight) = inner_edge_weights.next().unwrap();

            let prev_face = if i == 0 {
                // Wrap to avoid negative index.
                triangle_faces[triangle_faces.len() - 1]
            } else {
                triangle_faces[i - 1]
            };

            let (forward, backward) = self.add_unwired_edge(
                perimeter_vertex,
                inner_vertex,
                prev_face,
                triangle_faces[i],
                weight,
                twin_weight,
            );
            edges.push(self.full_edge(forward));

            i += 1;
        }

        edges
    }

    fn wire_triangulation_faces_edges_vertexes(
        &mut self,
        perimeter_face: FaceId,
        inner_edges: &[EdgeId],
    ) {
        let inner_edges_circular_tuple_windows = inner_edges
            .iter()
            .zip(inner_edges.iter().skip(1).chain(inner_edges.iter().take(1)));
        let mut perimeter_half_edges_walker = self.face_half_edges(perimeter_face).walker();

        for (inner_edge, next_inner_edge) in inner_edges_circular_tuple_windows {
            let perimeter_half_edge = perimeter_half_edges_walker.next(self).unwrap();
            let triangle_face = self.face_in_front(inner_edge.greater());

            self.wire_inner_half_edge_chain(
                triangle_face,
                &[
                    perimeter_half_edge,
                    next_inner_edge.lesser(),
                    inner_edge.greater(),
                ],
            );
        }
    }
}

// TODO: Tests.
