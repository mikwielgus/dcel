// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, Push};

use crate::{
    Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId,
    track::{EdgesTracker, VertexTracker},
};

impl<
    VW: Clone + Eq + Ord,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Insert a mesh made of overlapping polygons.
    pub fn insert_mesh(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        self.insert_mesh_in_face(face_polygons)
    }

    /// Insert a mesh made of overlapping polygons.
    pub fn insert_mesh_in_face(
        &mut self,
        face_polygons: impl IntoIterator<Item = impl IntoIterator<Item = VW>>,
    ) -> Vec<FaceId> {
        let mut vertex_weights_counter = VertexTracker::new();
        let mut edges_tracker = EdgesTracker::new();
        let mut faces = vec![];

        for face_polygon in face_polygons {
            faces.push(self.insert_adjoined_polygon(
                &mut vertex_weights_counter,
                &mut edges_tracker,
                face_polygon,
            ));
        }

        faces
    }

    fn insert_adjoined_polygon(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> FaceId {
        self.insert_adjoined_polygon_in_face(
            vertex_weights_counter,
            edges_tracker,
            self.unbounded_face(),
            vertex_weights,
        )
    }

    fn insert_adjoined_polygon_in_face(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> FaceId {
        self.insert_adjoined_polygon_in_face_with_all_weights(
            vertex_weights_counter,
            edges_tracker,
            outer_face,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            FW::default(),
        )
    }
}

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Insert a polygon.
    pub fn insert_polygon(&mut self, vertex_weights: impl IntoIterator<Item = VW>) -> FaceId {
        self.insert_polygon_in_face(self.unbounded_face(), vertex_weights)
    }

    /// Insert a polygon in a given face.
    pub fn insert_polygon_in_face(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> FaceId {
        self.insert_polygon_in_face_with_all_weights(
            outer_face,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            FW::default(),
        )
    }
}

impl<
    VW: Clone + Eq + Ord,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn insert_adjoined_polygon_with_all_weights(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        self.insert_adjoined_polygon_in_face_with_all_weights(
            vertex_weights_counter,
            edges_tracker,
            self.unbounded_face(),
            vertex_weights,
            edge_weights,
            face_weight,
        )
    }

    fn insert_adjoined_polygon_in_face_with_all_weights(
        &mut self,
        vertex_weights_tracker: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        let new_face = self.add_unwired_face(face_weight);
        let vertices =
            self.add_deduplicated_unwired_polygon_vertices(vertex_weights_tracker, vertex_weights);
        let edges = self.add_adjoined_unwired_polygon_edges(
            edges_tracker,
            &vertices,
            new_face,
            outer_face,
            edge_weights,
        );

        self.wire_outer_half_edge_chain_adjoiningly(
            outer_face,
            &edges
                .iter()
                .map(|(_, backward)| *backward)
                .collect::<Vec<_>>(),
        );
        self.wire_inner_half_edge_chain(
            new_face,
            &edges
                .iter()
                .map(|(forward, _)| *forward)
                .collect::<Vec<_>>(),
        );

        new_face
    }

    fn add_deduplicated_unwired_polygon_vertices(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> Vec<VertexId> {
        vertex_weights
            .into_iter()
            .map(|weight| {
                vertex_weights_counter
                    .vertex(weight.clone())
                    .unwrap_or_else(|| {
                        let vertex = self.add_unwired_vertex(weight.clone());
                        vertex_weights_counter.visit_vertex(weight, vertex);
                        vertex
                    })
            })
            .collect()
    }

    fn add_adjoined_unwired_polygon_edges(
        &mut self,
        edges_tracker: &mut EdgesTracker,
        vertices: &[VertexId],
        new_face: FaceId,
        outer_face: FaceId,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<(HalfEdgeId, HalfEdgeId)> {
        let vertices_circular_tuple_windows = vertices
            .iter()
            .zip(vertices.iter().skip(1).chain(vertices.iter().take(1)));

        let mut edges: Vec<(HalfEdgeId, HalfEdgeId)> = vec![];

        for ((&from_vertex, &to_vertex), (forward_half_edge_weight, backward_half_edge_weight)) in
            vertices_circular_tuple_windows.zip(edge_weights)
        {
            let edge = if let Some(existing_half_edge) =
                edges_tracker.vertices_half_edge(to_vertex, from_vertex)
            {
                // Reuse already existing shared edge.
                let forward = self.twin(existing_half_edge);
                let reused_edge = (forward, existing_half_edge);

                // Make the forward half-edge point to the new face.
                self.half_edges.insert(
                    forward.id(),
                    HalfEdge {
                        face: new_face,
                        ..self.half_edges.get(&forward.id()).unwrap().clone()
                    },
                );

                reused_edge
            } else {
                // There is no preexisting shared edge. Add a new one.
                let (forward, backward) = self.add_unwired_edge(
                    from_vertex,
                    to_vertex,
                    new_face,
                    outer_face,
                    forward_half_edge_weight,
                    backward_half_edge_weight,
                );
                (forward, backward)
            };

            edges_tracker.visit_vertices_edge(from_vertex, to_vertex, edge.0);
            edges.push(edge);
        }

        edges
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
    /// Insert a polygon with all its vertex and edge weights, and its face
    /// weight, specified.
    pub fn insert_polygon_with_all_weights(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        self.insert_polygon_in_face_with_all_weights(
            self.unbounded_face(),
            vertex_weights,
            edge_weights,
            face_weight,
        )
    }

    /// Insert a polygon into a given face with all its vertex and edge weights,
    /// and its face weight, specified.
    pub fn insert_polygon_in_face_with_all_weights(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) -> FaceId {
        let new_face = self.add_unwired_face(face_weight);
        let vertices = self.add_unwired_polygon_vertices(vertex_weights);
        let edges = self.add_unwired_polygon_edges(&vertices, new_face, outer_face, edge_weights);

        self.wire_outer_half_edge_chain_circularly(
            &edges.iter().map(|edge| edge.greater()).collect::<Vec<_>>(),
        );
        self.wire_inner_half_edge_chain(
            new_face,
            &edges.iter().map(|edge| edge.lesser()).collect::<Vec<_>>(),
        );

        new_face
    }

    fn add_unwired_polygon_vertices(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> Vec<VertexId> {
        vertex_weights
            .into_iter()
            .map(|vertex_weight| self.add_unwired_vertex(vertex_weight))
            .collect()
    }

    fn add_unwired_polygon_edges(
        &mut self,
        vertices: &[VertexId],
        new_face: FaceId,
        outer_face: FaceId,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let vertices_circular_tuple_windows = vertices
            .iter()
            .zip(vertices.iter().skip(1).chain(vertices.iter().take(1)));

        let mut edges = vec![];

        for ((from_vertex, to_vertex), (forward_half_edge_weight, backward_half_edge_weight)) in
            vertices_circular_tuple_windows.zip(edge_weights)
        {
            let (forward, _) = self.add_unwired_edge(
                *from_vertex,
                *to_vertex,
                new_face,
                outer_face,
                forward_half_edge_weight,
                backward_half_edge_weight,
            );
            edges.push(self.full_edge(forward));
        }

        edges
    }
}

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    /// Insert an edge between two vertices, splitting a face in two.
    ///
    /// Returns the ids of the edge's half-edges and the id of the new face
    /// created by the split.
    pub fn insert_edge(
        &mut self,
        from: VertexId,
        to: VertexId,
    ) -> ((HalfEdgeId, HalfEdgeId), FaceId) {
        self.split_face_by_edge(from, to, self.find_vertices_common_face(from, to).unwrap())
    }

    /// Insert an edge chain between two vertices, splitting a face in two.
    ///
    /// Returns the ids of the edge's half-edges and the id of the new face
    /// created by the split.
    pub fn insert_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) -> (Vec<(HalfEdgeId, HalfEdgeId)>, FaceId) {
        self.split_face_by_edge_chain(
            from,
            to,
            vertex_weights,
            self.find_vertices_common_face(from, to).unwrap(),
        )
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn insert_edge_chain_with_edge_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> (Vec<(HalfEdgeId, HalfEdgeId)>, FaceId) {
        self.split_face_by_edge_chain_with_all_weights(
            from,
            to,
            vertex_weights,
            edge_weights,
            self.find_vertices_common_face(from, to).unwrap(),
            FW::default(),
        )
    }
}

#[cfg(test)]
mod test {
    use crate::{HalfEdgeId, assert_face_boundary, init_dcel_with_3x3_hex_mesh};

    use super::*;

    #[test]
    fn test_insert_polygon() {
        let mut dcel = Dcel::<(i32, i32)>::new();
        let face = dcel.insert_polygon([(0, 0), (10, 0), (10, 10), (0, 10)]);

        assert_eq!(dcel.faces().len(), 2);

        assert_face_boundary!(dcel, 0, 0);
        assert_face_boundary!(dcel, face.id(), 4);
    }

    #[test]
    fn test_insert_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(Dcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertices: Vec<VertexId> = dcel.face_vertices(face_to_split).collect();

        // Split face 5 in two with a single edge.
        let (_new_edge, _new_face) =
            dcel.insert_edge(face_to_split_vertices[0], face_to_split_vertices[3]);

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(dcel.faces().len(), 11);

        // The original hexagon is now split into two quads.
        assert_face_boundary!(dcel, 0, 0);
        assert_face_boundary!(dcel, 1, 6);
        assert_face_boundary!(dcel, 2, 6);
        assert_face_boundary!(dcel, 3, 6);
        assert_face_boundary!(dcel, 4, 6);
        // First quad face.
        assert_face_boundary!(dcel, 5, 4);
        assert_face_boundary!(dcel, 6, 6);
        assert_face_boundary!(dcel, 7, 6);
        assert_face_boundary!(dcel, 8, 6);
        assert_face_boundary!(dcel, 9, 6);
        // Second quad face.
        assert_face_boundary!(dcel, 10, 4);
    }

    #[test]
    fn test_insert_adjoined_squares() {
        let two_adjoined_squares: Vec<Vec<(i64, i64)>> = vec![
            // Left square (CCW).
            vec![(0, 0), (0, 1), (-1, 1), (-1, 0)],
            // Right square sharing edge with left square (CCW).
            vec![(0, 0), (1, 0), (1, 1), (0, 1)],
        ];

        let mut dcel: Dcel<(i64, i64)> = Dcel::new();
        dcel.insert_mesh(two_adjoined_squares);

        assert_eq!(dcel.vertices().len(), 6);
        assert_eq!(dcel.half_edges().len(), 14);
        assert_eq!(dcel.faces().len(), 3);

        assert_eq!(
            dcel.face_vertices(FaceId::new(0))
                .collect::<Vec<VertexId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_half_edges(FaceId::new(0))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_edges(FaceId::new(0))
                .collect::<Vec<EdgeId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_half_edges(FaceId::new(1))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
        assert_eq!(
            dcel.face_edges(FaceId::new(1))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
        assert_eq!(
            dcel.face_half_edges(FaceId::new(1))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
        assert_eq!(
            dcel.face_edges(FaceId::new(2))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_insert_noisy_4x4_grid_mesh() {
        // A 2D grid mesh of 4x4 faces with slightly noisified (i64, i64)
        // coordinates.
        let mesh: Vec<Vec<(i64, i64)>> = vec![
            // Row 1.
            vec![(2, 3), (48, -2), (53, 54), (-4, 48)],
            vec![(48, -2), (102, 4), (98, 51), (53, 54)],
            vec![(102, 4), (155, -3), (147, 55), (98, 51)],
            vec![(155, -3), (201, 2), (197, 49), (147, 55)],
            // Row 2.
            vec![(-4, 48), (53, 54), (47, 103), (3, 98)],
            vec![(53, 54), (98, 51), (104, 97), (47, 103)],
            vec![(98, 51), (147, 55), (149, 102), (104, 97)],
            vec![(147, 55), (197, 49), (203, 99), (149, 102)],
            // Row 3.
            vec![(3, 98), (47, 103), (51, 155), (-2, 147)],
            vec![(47, 103), (104, 97), (97, 149), (51, 155)],
            vec![(104, 97), (149, 102), (155, 148), (97, 149)],
            vec![(149, 102), (203, 99), (198, 152), (155, 148)],
            // Row 4.
            vec![(-2, 147), (51, 155), (49, 204), (5, 197)],
            vec![(51, 155), (97, 149), (102, 198), (49, 204)],
            vec![(97, 149), (155, 148), (146, 201), (102, 198)],
            vec![(155, 148), (198, 152), (204, 199), (146, 201)],
        ];

        let mut dcel: Dcel<(i64, i64)> = Dcel::new();
        dcel.insert_mesh(mesh);

        for (i, _face) in dcel.faces().iter().enumerate() {
            if FaceId::new(i) == dcel.unbounded_face() {
                assert_eq!(
                    dcel.face_vertices(FaceId::new(i))
                        .collect::<Vec<VertexId>>()
                        .len(),
                    0
                );
                assert_eq!(
                    dcel.face_half_edges(FaceId::new(i))
                        .collect::<Vec<HalfEdgeId>>()
                        .len(),
                    0
                );
                assert_eq!(
                    dcel.face_edges(FaceId::new(i))
                        .collect::<Vec<EdgeId>>()
                        .len(),
                    0
                );
                continue;
            }

            assert_eq!(
                dcel.face_vertices(FaceId::new(i))
                    .collect::<Vec<VertexId>>()
                    .len(),
                4
            );
            assert_eq!(
                dcel.face_half_edges(FaceId::new(i))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                4
            );
            assert_eq!(
                dcel.face_edges(FaceId::new(i))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                4
            );
        }

        assert_eq!(dcel.vertices().len(), 25);
        assert_eq!(dcel.half_edges().len(), 80);
        assert_eq!(dcel.faces().len(), 17);
    }
}
