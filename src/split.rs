// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, Push};

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, Vertex, VertexId};

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Item = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Item = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Item = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn split_face_by_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        split_face: FaceId,
    ) {
        self.split_face_by_edge_chain_with_all_weights(
            from,
            to,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            split_face,
            FW::default(),
        )
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
    pub fn split_face_by_edge_chain_with_all_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        split_face: FaceId,
        new_face_weight: FW,
    ) {
        let new_face = self.add_unwired_face(new_face_weight);
        let (new_edges, last_vertex, last_edge_weight) = self.add_unwired_dangling_edge_chain(
            from,
            vertex_weights,
            edge_weights,
            split_face,
            new_face,
        );
        self.add_unwired_edge(
            last_vertex,
            to,
            split_face,
            new_face,
            last_edge_weight.0,
            last_edge_weight.1,
        );

        let mut edges = vec![];
        edges.push(self.vertex_prev_edge(from));
        edges.extend(new_edges);
        edges.push(self.vertex_next_edge(to));

        self.wire_edge_chain(new_face, &edges);
    }

    fn add_unwired_dangling_edge_chain(
        &mut self,
        from: VertexId,
        dangling_vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face: FaceId,
        twin_face: FaceId,
    ) -> (Vec<EdgeId>, VertexId, (HEW, HEW)) {
        let mut edge_weights = edge_weights.into_iter();
        let mut edges = vec![];
        let mut last_vertex = from;

        for (vertex_weight, edge_weight) in dangling_vertex_weights
            .into_iter()
            .zip(edge_weights.by_ref())
        {
            let (new_edge, new_vertex) = self.add_unwired_dangling_edge(
                last_vertex,
                vertex_weight,
                edge_weight,
                face,
                twin_face,
            );

            edges.push(new_edge);
            last_vertex = new_vertex;
        }

        (edges, last_vertex, edge_weights.next().unwrap())
    }

    fn add_unwired_dangling_edge(
        &mut self,
        from: VertexId,
        dangling_vertex_weight: VW,
        edge_weight: (HEW, HEW),
        face: FaceId,
        twin_face: FaceId,
    ) -> (EdgeId, VertexId) {
        let dangling_vertex = self.add_unwired_vertex(dangling_vertex_weight);
        let dangling_edge = self.add_unwired_edge(
            from,
            dangling_vertex,
            face,
            twin_face,
            edge_weight.0,
            edge_weight.1,
        );

        (dangling_edge, dangling_vertex)
    }
}
