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
    pub fn split_face_by_edge(&mut self, from: VertexId, to: VertexId, split_face: FaceId) {
        self.split_face_by_edge_chain(from, to, [], split_face);
    }

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
        );
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
    pub fn split_face_by_edge_chain_with_all_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_to_split: FaceId,
        new_face_weight: FW,
    ) {
        let from_incoming = self.vertex_inner_incoming_half_edge(from, face_to_split);
        let from_outgoing = self.vertex_inner_outgoing_half_edge(from, face_to_split);
        let to_incoming = self.vertex_inner_incoming_half_edge(to, face_to_split);
        let to_outgoing = self.vertex_inner_outgoing_half_edge(to, face_to_split);

        let new_face = self.add_unwired_face(new_face_weight);
        let (mut new_edges, last_vertex, last_edge_weight) = self.add_unwired_dangling_edge_chain(
            from,
            vertex_weights,
            edge_weights,
            face_to_split,
            new_face,
        );
        new_edges.push(self.add_unwired_edge(
            last_vertex,
            to,
            face_to_split,
            new_face,
            last_edge_weight.0,
            last_edge_weight.1,
        ));

        let mut face_to_split_edges = vec![];
        face_to_split_edges.push(self.full_edge(from_incoming));
        face_to_split_edges.extend(new_edges.iter().copied());
        face_to_split_edges.push(self.full_edge(to_outgoing));

        // TODO: No need to run the whole loop here actually.
        let mut curr_half_edge = self.next_half_edge(to_outgoing);
        while curr_half_edge != from_incoming {
            face_to_split_edges.push(self.full_edge(curr_half_edge));
            curr_half_edge = self.next_half_edge(curr_half_edge);
        }

        let mut new_face_edges = vec![];
        new_face_edges.push(self.full_edge(to_incoming));
        new_face_edges.extend(new_edges.iter().rev().map(|edge| self.reverse_edge(*edge)));
        new_face_edges.push(self.full_edge(from_outgoing));

        // TODO: No need to run the whole loop here actually.
        let mut curr_half_edge = self.next_half_edge(from_outgoing);
        while curr_half_edge != to_incoming {
            new_face_edges.push(self.full_edge(curr_half_edge));
            curr_half_edge = self.next_half_edge(curr_half_edge);
        }

        self.wire_inner_half_edge_chain(face_to_split, &face_to_split_edges);
        self.wire_inner_half_edge_chain(new_face, &new_face_edges);
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
