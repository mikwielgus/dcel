// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, StableRemove};

use crate::{
    Dcel, EdgeId, Face, FaceId, HalfEdge, Vertex, VertexId,
    track::{HalfEdgesCounter, VertexesCounter},
};

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + StableRemove<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + StableRemove<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + StableRemove<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn merge_faces_around_vertex(&mut self, inner_vertex: VertexId) {
        let absorbing_face = self.face_in_front(self.outgoing_next_half_edge(inner_vertex));
        self.absorb_faces_around_vertex(absorbing_face, inner_vertex);
    }

    pub fn absorb_faces_around_vertex(&mut self, absorbing_face: FaceId, inner_vertex: VertexId) {
        let initial_half_edge = self.outgoing_next_half_edge(inner_vertex);
        let initial_edge = self.full_edge(initial_half_edge);

        let inner_edges: Vec<EdgeId> = self.spokes(initial_edge).collect();
        let perimeter_edges: Vec<EdgeId> =
            self.vertex_rim_edges(inner_vertex).collect::<Vec<EdgeId>>();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            self.interspokes(initial_half_edge)
                .filter(|face| face.id() != absorbing_face.id())
                .collect::<Vec<FaceId>>(),
            inner_edges,
            [inner_vertex],
            &perimeter_edges,
        );
    }

    pub fn merge_faces_over_edges_and_vertexes(
        &mut self,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces_over_edges_and_vertexes(
            absorbing_face,
            faces.filter(|face| face.id() != absorbing_face.id()),
            edges,
            vertexes,
        );
    }

    pub fn absorb_faces_over_edges_and_vertexes(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
        edges: impl IntoIterator<Item = EdgeId>,
        vertexes: impl IntoIterator<Item = VertexId>,
    ) {
        let edges: Vec<EdgeId> = edges.into_iter().collect();

        // Find an initial edge that is not in the excluded list. Otherwise, the
        // circulator could end up starting from an excluded edge, which would
        // result in an infinite loop, as the termination condition depends on
        // returning to the initial edge.
        let initial_edge = self
            .face_edges(absorbing_face)
            .find(|edge| !edges.contains(edge))
            .unwrap();

        let perimeter_edges: Vec<EdgeId> = self
            .circulate_edges_with_excludes(initial_edge, edges.clone())
            .collect();

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces,
            edges,
            vertexes,
            &perimeter_edges,
        );
    }

    pub fn merge_faces(&mut self, faces: impl IntoIterator<Item = FaceId>) {
        let mut faces = faces.into_iter();
        let absorbing_face = faces.next().unwrap();

        self.absorb_faces(
            absorbing_face,
            faces.filter(|face| face.id() != absorbing_face.id()),
        );
    }

    pub fn absorb_faces(
        &mut self,
        absorbing_face: FaceId,
        faces: impl IntoIterator<Item = FaceId>,
    ) {
        let mut half_edges_counter = HalfEdgesCounter::new();
        let mut vertex_weights_counter = VertexesCounter::new();
        let faces: Vec<FaceId> = faces.into_iter().collect();

        // To detect the merged edges and vertexes correctly, the absorbing face
        // has to be visited in addition to the absorbed faces.
        half_edges_counter.visit_face_half_edges(self, absorbing_face);
        vertex_weights_counter.visit_face_vertexes(self, absorbing_face);

        for &face in &faces {
            half_edges_counter.visit_face_half_edges(self, face);
            vertex_weights_counter.visit_face_vertexes(self, face);
        }

        self.absorb_faces_over_edges_and_vertexes_in_perimeter(
            absorbing_face,
            faces,
            half_edges_counter
                .inner_edges(self)
                .collect::<Vec<EdgeId>>(),
            vertex_weights_counter
                .visited_vertexes()
                .filter(|&vertex| {
                    self.spokes_reverse(self.vertex_next_edge(vertex))
                        .all(|edge| half_edges_counter.is_inner_edge(edge))
                })
                // PERF: Needless collect?
                .collect::<Vec<VertexId>>(),
            &half_edges_counter
                .outer_edges(self)
                .collect::<Vec<EdgeId>>(),
        );
    }

    pub(crate) fn absorb_faces_over_edges_and_vertexes_in_perimeter(
        &mut self,
        absorbing_face: FaceId,
        faces_to_absorb: impl IntoIterator<Item = FaceId>,
        edges_to_remove: impl IntoIterator<Item = EdgeId>,
        vertexes_to_remove: impl IntoIterator<Item = VertexId>,
        perimeter_edges: &[EdgeId],
    ) {
        self.remove_faces(faces_to_absorb);
        self.remove_edges(edges_to_remove);
        self.remove_vertexes(vertexes_to_remove);

        self.wire_inner_half_edge_chain(absorbing_face, perimeter_edges);
    }
}

#[cfg(all(test, feature = "stable-vec"))]
mod test {
    use crate::{
        EdgeId, FaceId, HalfEdgeId, StableDcel, VertexId, assert_face_boundary,
        init_dcel_with_3x3_hex_mesh,
    };

    #[test]
    fn test_merge_faces_around_vertex() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.merge_faces_around_vertex(VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(dcel.faces().num_elements(), 8);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        // Face 2 does not exist.
        assert_face_boundary!(&dcel, 3, 6);
        // Face 4 does not exist.
        assert_face_boundary!(&dcel, 5, 12);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn test_absorb_faces_around_vertex() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.absorb_faces_around_vertex(FaceId::new(2), VertexId::new(8));

        // There are now eight faces in total: one unbounded and seven bounded.
        assert_eq!(dcel.faces().num_elements(), 8);

        // Among the remaining faces, one is now a dodecagon, and the remaining
        // six are hexagons.
        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 12);
        assert_face_boundary!(&dcel, 3, 6);
        // Face 4 does not exist.
        // Face 5 does not exist.
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn test_merge_faces_over_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.merge_faces_over_edges_and_vertexes(
            [FaceId::new(5), FaceId::new(6)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(dcel.faces().num_elements(), 9);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        assert_face_boundary!(&dcel, 5, 10);
        // Face 6 does not exist.
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn test_absorb_face_into_face_over_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.absorb_faces_over_edges_and_vertexes(
            FaceId::new(6),
            [FaceId::new(5)],
            [EdgeId::new(HalfEdgeId::new(44), HalfEdgeId::new(45))],
            [],
        );

        assert_eq!(dcel.faces().num_elements(), 9);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        // Face 5 does not exist.
        assert_face_boundary!(&dcel, 6, 10);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn merge_two_faces() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.merge_faces([FaceId::new(7), FaceId::new(8)]);

        assert_eq!(dcel.faces().num_elements(), 9);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        assert_face_boundary!(&dcel, 5, 6);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 10);
        // Face 8 does not exist.
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn absorb_face_into_face() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        dcel.absorb_faces(FaceId::new(8), [FaceId::new(7)]);

        assert_eq!(dcel.faces().num_elements(), 9);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        assert_face_boundary!(&dcel, 5, 6);
        assert_face_boundary!(&dcel, 6, 6);
        // Face 7 does not exist.
        assert_face_boundary!(&dcel, 8, 10);
        assert_face_boundary!(&dcel, 9, 6);
    }
}
