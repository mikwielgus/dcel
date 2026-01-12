// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use std::hash::Hash;

use maplike::{Get, Insert, Push};

use crate::{
    Dcel, EdgeId, Face, FaceId, HalfEdge, Vertex, VertexId,
    track::{EdgesTracker, VertexTracker},
};

impl<
    VW: Clone + Eq + Hash,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_mesh(&mut self, face_polygons: impl IntoIterator<Item = Vec<VW>>) {
        self.insert_mesh_in_face(face_polygons);
    }

    pub fn insert_mesh_in_face(&mut self, face_polygons: impl IntoIterator<Item = Vec<VW>>) {
        let mut vertex_weights_counter = VertexTracker::new();
        let mut edges_tracker = EdgesTracker::new();

        for face_polygon in face_polygons {
            self.insert_adjoined_polygon(
                &mut vertex_weights_counter,
                &mut edges_tracker,
                face_polygon,
            );
        }
    }

    fn insert_adjoined_polygon(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) {
        self.insert_adjoined_polygon_in_face(
            vertex_weights_counter,
            edges_tracker,
            self.unbounded_face(),
            vertex_weights,
        );
    }

    fn insert_adjoined_polygon_in_face(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
    ) {
        self.insert_adjoined_polygon_in_face_with_all_weights(
            vertex_weights_counter,
            edges_tracker,
            outer_face,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            FW::default(),
        );
    }
}

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon(&mut self, vertex_weights: impl IntoIterator<Item = VW>) {
        self.insert_polygon_in_face(self.unbounded_face(), vertex_weights);
    }

    pub fn insert_polygon_in_face(
        &mut self,
        outer_face: FaceId,
        vertexes_weights: impl IntoIterator<Item = VW>,
    ) {
        self.insert_polygon_in_face_with_all_weights(
            outer_face,
            vertexes_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            FW::default(),
        );
    }
}

impl<
    VW: Clone + Eq + Hash,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    fn insert_adjoined_polygon_with_all_weights(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        self.insert_adjoined_polygon_in_face_with_all_weights(
            vertex_weights_counter,
            edges_tracker,
            self.unbounded_face(),
            vertex_weights,
            edge_weights,
            face_weight,
        );
    }

    fn insert_adjoined_polygon_in_face_with_all_weights(
        &mut self,
        vertex_weights_counter: &mut VertexTracker<VW>,
        edges_tracker: &mut EdgesTracker,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        let new_face = self.add_unwired_face(face_weight);
        let vertexes =
            self.add_deduplicated_unwired_polygon_vertexes(vertex_weights_counter, vertex_weights);
        let edges = self.add_adjoined_unwired_polygon_edges(
            edges_tracker,
            &vertexes,
            new_face,
            outer_face,
            edge_weights,
        );

        self.wire_face_edges_vertexes(new_face, &edges);
    }

    fn add_deduplicated_unwired_polygon_vertexes(
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
        vertexes: &[VertexId],
        new_face: FaceId,
        outer_face: FaceId,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let vertexes_circular_pair_windows = vertexes
            .iter()
            .zip(vertexes.iter().skip(1).chain(vertexes.iter().take(1)));

        let mut edges = vec![];

        for ((&from_vertex, &to_vertex), (forward_half_edge_weight, backward_half_edge_weight)) in
            vertexes_circular_pair_windows.zip(edge_weights)
        {
            let edge = edges_tracker
                // Note that vertexes are intentionally reversed here.
                .vertexes_half_edge(to_vertex, from_vertex)
                .map(|half_edge| self.full_edge(half_edge))
                .unwrap_or_else(|| {
                    self.add_unwired_edge(
                        from_vertex,
                        to_vertex,
                        new_face,
                        outer_face,
                        forward_half_edge_weight,
                        backward_half_edge_weight,
                    )
                });
            edges_tracker.visit_vertexes_edge(from_vertex, to_vertex, edge.forward());
            edges.push(edge);
        }

        edges
    }
}

impl<
    VW: Clone,
    HEW: Clone,
    FW: Clone,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn insert_polygon_with_all_weights(
        &mut self,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        self.insert_polygon_in_face_with_all_weights(
            self.unbounded_face(),
            vertex_weights,
            edge_weights,
            face_weight,
        );
    }

    pub fn insert_polygon_in_face_with_all_weights(
        &mut self,
        outer_face: FaceId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_weight: FW,
    ) {
        let new_face = self.add_unwired_face(face_weight);
        let vertexes = self.add_unwired_polygon_vertexes(vertex_weights);
        let edges = self.add_unwired_polygon_edges(&vertexes, new_face, outer_face, edge_weights);

        self.wire_face_edges_vertexes(new_face, &edges);
    }

    fn add_unwired_polygon_vertexes(
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
        vertexes: &[VertexId],
        new_face: FaceId,
        outer_face: FaceId,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
    ) -> Vec<EdgeId> {
        let vertexes_circular_pair_windows = vertexes
            .iter()
            .zip(vertexes.iter().skip(1).chain(vertexes.iter().take(1)));

        let mut edges = vec![];

        for ((from_vertex, to_vertex), (forward_half_edge_weight, backward_half_edge_weight)) in
            vertexes_circular_pair_windows.zip(edge_weights)
        {
            let edge = self.add_unwired_edge(
                *from_vertex,
                *to_vertex,
                new_face,
                outer_face,
                forward_half_edge_weight,
                backward_half_edge_weight,
            );
            edges.push(edge);
        }

        edges
    }
}

#[cfg(test)]
mod test {
    use super::*;

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

        assert_eq!(dcel.vertexes().len(), 25);
        assert_eq!(dcel.half_edges().len(), 80);
        assert_eq!(dcel.faces().len(), 17);
    }
}
