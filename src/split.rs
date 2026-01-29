// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::{Get, Insert, Push};

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

impl<
    VW: Clone,
    HEW: Clone + Default,
    FW: Clone + Default,
    VC: Get<usize, Value = Vertex<VW>> + Insert<usize> + Push<usize>,
    HEC: Get<usize, Value = HalfEdge<HEW>> + Insert<usize> + Push<usize>,
    FC: Get<usize, Value = Face<FW>> + Insert<usize> + Push<usize>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn split_edge_by_vertex(&mut self, edge_to_split: EdgeId, vertex: VW) -> EdgeId {
        let (origin, _) = self.endpoints(edge_to_split);
        let forward = edge_to_split.forward();
        let backward = edge_to_split.backward();
        let face = self.face_in_front(forward);
        let twin_face = self.face_in_front(backward);
        let (forward_weight, backward_weight) = self.edge_weights(edge_to_split);
        let (forward_weight, backward_weight) = (forward_weight.clone(), backward_weight.clone());

        let new_vertex = self.add_unwired_vertex(vertex);
        let (new_forward, new_backward) = self.add_unwired_edge(
            origin,
            new_vertex,
            face,
            twin_face,
            forward_weight.clone(),
            backward_weight.clone(),
        );
        let new_edge = EdgeId::new(new_forward, new_backward);

        // Retarget the original edge to start at the new vertex.
        self.half_edges.insert(
            forward.id(),
            HalfEdge {
                origin: new_vertex,
                ..self.half_edges.get(&forward.id()).unwrap().clone()
            },
        );

        let forward_prev = self.prev_half_edge(forward);
        let backward_next = self.next_half_edge(backward);

        // Insert the new edge in the forward face cycle.
        self.link_subsequent_half_edges(forward_prev, new_edge.forward());
        self.link_subsequent_half_edges(new_edge.forward(), forward);

        // Insert the new edge in the backward face cycle.
        self.link_subsequent_half_edges(backward, new_edge.backward());
        self.link_subsequent_half_edges(new_edge.backward(), backward_next);

        // Update the outgoing spokes of vertexes.
        // XXX: Is this really needed?
        self.link_vertex_with_half_edge(new_vertex, forward);
        if self.outgoing_next_half_edge(origin) == forward {
            self.link_vertex_with_half_edge(origin, new_edge.forward());
        }

        new_edge
    }

    pub fn split_face_by_edge(
        &mut self,
        from: VertexId,
        to: VertexId,
        face_to_split: FaceId,
    ) -> FaceId {
        self.split_face_by_edge_chain(from, to, [], face_to_split)
    }

    pub fn split_face_by_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        face_to_split: FaceId,
    ) -> FaceId {
        self.split_face_by_edge_chain_with_all_weights(
            from,
            to,
            vertex_weights,
            std::iter::repeat((HEW::default(), HEW::default())),
            face_to_split,
            FW::default(),
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
    pub fn split_face_by_edge_chain_with_all_weights(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face_to_split: FaceId,
        new_face_weight: FW,
    ) -> FaceId {
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
        let (forward, backward) = self.add_unwired_edge(
            last_vertex,
            to,
            face_to_split,
            new_face,
            last_edge_weight.0,
            last_edge_weight.1,
        );
        new_edges.push(EdgeId::new(forward, backward));

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

        self.wire_inner_half_edge_chain(
            face_to_split,
            &face_to_split_edges
                .iter()
                .map(|edge| edge.forward())
                .collect::<Vec<HalfEdgeId>>(),
        );
        self.wire_inner_half_edge_chain(
            new_face,
            &new_face_edges
                .iter()
                .map(|edge| edge.forward())
                .collect::<Vec<HalfEdgeId>>(),
        );

        new_face
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
        let (forward, backward) = self.add_unwired_edge(
            from,
            dangling_vertex,
            face,
            twin_face,
            edge_weight.0,
            edge_weight.1,
        );
        let dangling_edge = EdgeId::new(forward, backward);

        (dangling_edge, dangling_vertex)
    }
}

#[cfg(all(test, feature = "stable-vec"))]
mod test {
    use crate::{
        EdgeId, FaceId, HalfEdgeId, StableDcel, VertexId, assert_face_boundary,
        init_dcel_with_3x3_hex_mesh,
    };

    #[test]
    fn test_split_edge_by_vertex() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        let face = FaceId::new(5);
        let edge = EdgeId::new(HalfEdgeId::new(38), HalfEdgeId::new(39));
        let old_origin = dcel.origin(edge.forward());

        let new_edge = dcel.split_edge_by_vertex(edge, (259, 150));

        assert_eq!(dcel.vertexes().num_elements(), 31);
        assert_eq!(dcel.half_edges().num_elements(), 78);
        assert_eq!(dcel.faces().num_elements(), 10);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 7);
        assert_face_boundary!(&dcel, 5, 7);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }

    #[test]
    fn test_split_face_by_edge() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertexes: Vec<VertexId> = dcel.face_vertexes(face_to_split).collect();

        // Split face 5 in two with a single edge.
        dcel.split_face_by_edge(
            face_to_split_vertexes[0],
            face_to_split_vertexes[3],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(dcel.faces().num_elements(), 11);

        // The original hexagon is now split into two quads.
        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        // Second quad face.
        assert_face_boundary!(&dcel, 5, 4);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
        // Second quad face.
        assert_face_boundary!(&dcel, 10, 4);
    }

    #[test]
    fn test_split_face_by_chain_of_two_edges() {
        let mut dcel = init_dcel_with_3x3_hex_mesh!(StableDcel<(i32, i32)>);
        let face_to_split = FaceId::new(5);
        let face_to_split_vertexes: Vec<VertexId> = dcel.face_vertexes(face_to_split).collect();

        // Split face 5 in two with a chain of two edges, with their common
        // point around the face's center.
        dcel.split_face_by_edge_chain(
            face_to_split_vertexes[0],
            face_to_split_vertexes[3],
            [(259, 150)],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(dcel.faces().num_elements(), 11);

        // The original hexagon is now split into two pentagons.

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        // First pentagon face.
        assert_face_boundary!(&dcel, 5, 5);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
        // Second pentagon face.
        assert_face_boundary!(&dcel, 10, 5);
    }
}
