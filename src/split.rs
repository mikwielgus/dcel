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
    pub fn split_edge_by_vertex(
        &mut self,
        edge_to_split: EdgeId,
        vertex: VW,
    ) -> (VertexId, EdgeId) {
        let (source, _) = self.edge_endpoints(edge_to_split);
        let forward = edge_to_split.lesser();
        let backward = edge_to_split.greater();
        let face = self.incident_face(forward);
        let twin_face = self.incident_face(backward);
        let (forward_weight, backward_weight) = self.edge_weights(edge_to_split);
        let (forward_weight, backward_weight) = (forward_weight.clone(), backward_weight.clone());

        let new_vertex = self.add_unwired_vertex(vertex);
        let (new_forward, _) = self.add_unwired_edge(
            source,
            new_vertex,
            face,
            twin_face,
            forward_weight.clone(),
            backward_weight.clone(),
        );
        let new_edge = self.full_edge(new_forward);

        // Retarget the sourceal edge to start at the new vertex.
        self.half_edges.insert(
            forward.id(),
            HalfEdge {
                source: new_vertex,
                ..self.half_edges.get(&forward.id()).unwrap().clone()
            },
        );

        let forward_prev = self.prev(forward);
        let backward_next = self.next(backward);

        // Insert the new edge in the forward face cycle.
        self.link_subsequent_half_edges(forward_prev, new_edge.lesser());
        self.link_subsequent_half_edges(new_edge.lesser(), forward);

        // Insert the new edge in the backward face cycle.
        self.link_subsequent_half_edges(backward, new_edge.greater());
        self.link_subsequent_half_edges(new_edge.greater(), backward_next);

        // Update the outgoing spokes of vertices.
        // XXX: Is this really needed?
        self.link_vertex_with_half_edge(new_vertex, forward);
        if self.outgoing_next_half_edge(source) == forward {
            self.link_vertex_with_half_edge(source, new_edge.lesser());
        }

        (new_vertex, new_edge)
    }

    pub fn split_face_by_edge(
        &mut self,
        from: VertexId,
        to: VertexId,
        face_to_split: FaceId,
    ) -> ((HalfEdgeId, HalfEdgeId), FaceId) {
        let (mut new_edges, new_face) = self.split_face_by_edge_chain(from, to, [], face_to_split);
        let new_edge = new_edges
            .pop()
            .expect("split_face_by_edge should create exactly one edge");

        (new_edge, new_face)
    }

    pub fn split_face_by_edge_chain(
        &mut self,
        from: VertexId,
        to: VertexId,
        vertex_weights: impl IntoIterator<Item = VW>,
        face_to_split: FaceId,
    ) -> (Vec<(HalfEdgeId, HalfEdgeId)>, FaceId) {
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
    ) -> (Vec<(HalfEdgeId, HalfEdgeId)>, FaceId) {
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
        new_edges.push((forward, backward));

        let mut face_to_split_directed_edges: Vec<(HalfEdgeId, HalfEdgeId)> = vec![];
        face_to_split_directed_edges.push((from_incoming, self.twin(from_incoming)));
        face_to_split_directed_edges.extend(new_edges.iter().copied());
        face_to_split_directed_edges.push((to_outgoing, self.twin(to_outgoing)));

        // TODO: No need to run the whole loop here actually.
        let mut curr_half_edge = self.next(to_outgoing);
        while curr_half_edge != from_incoming {
            face_to_split_directed_edges.push((curr_half_edge, self.twin(curr_half_edge)));
            curr_half_edge = self.next(curr_half_edge);
        }

        let mut new_face_directed_edges: Vec<(HalfEdgeId, HalfEdgeId)> = vec![];
        new_face_directed_edges.push((to_incoming, self.twin(to_incoming)));
        new_face_directed_edges.extend(
            new_edges
                .iter()
                .rev()
                .map(|(forward, backward)| (*backward, *forward)),
        );
        new_face_directed_edges.push((from_outgoing, self.twin(from_outgoing)));

        // TODO: No need to run the whole loop here actually.
        let mut curr_half_edge = self.next(from_outgoing);
        while curr_half_edge != to_incoming {
            new_face_directed_edges.push((curr_half_edge, self.twin(curr_half_edge)));
            curr_half_edge = self.next(curr_half_edge);
        }

        self.wire_inner_half_edge_chain(
            face_to_split,
            &face_to_split_directed_edges
                .iter()
                .map(|(forward, _)| *forward)
                .collect::<Vec<HalfEdgeId>>(),
        );
        self.wire_inner_half_edge_chain(
            new_face,
            &new_face_directed_edges
                .iter()
                .map(|(forward, _)| *forward)
                .collect::<Vec<HalfEdgeId>>(),
        );

        (new_edges, new_face)
    }

    fn add_unwired_dangling_edge_chain(
        &mut self,
        from: VertexId,
        dangling_vertex_weights: impl IntoIterator<Item = VW>,
        edge_weights: impl IntoIterator<Item = (HEW, HEW)>,
        face: FaceId,
        twin_face: FaceId,
    ) -> (Vec<(HalfEdgeId, HalfEdgeId)>, VertexId, (HEW, HEW)) {
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
        edge_weights: (HEW, HEW),
        face: FaceId,
        twin_face: FaceId,
    ) -> ((HalfEdgeId, HalfEdgeId), VertexId) {
        let dangling_vertex = self.add_unwired_vertex(dangling_vertex_weight);
        let (forward, backward) = self.add_unwired_edge(
            from,
            dangling_vertex,
            face,
            twin_face,
            edge_weights.0,
            edge_weights.1,
        );
        let dangling_edge = (forward, backward);

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
        let old_source = dcel.source(edge.lesser());

        let (_new_vertex, new_edge) = dcel.split_edge_by_vertex(edge, (259, 150));

        assert_eq!(dcel.vertices().num_elements(), 31);
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
        let face_to_split_vertices: Vec<VertexId> = dcel.face_vertices(face_to_split).collect();

        // Split face 5 in two with a single edge.
        let (_new_edge, _new_face) = dcel.split_face_by_edge(
            face_to_split_vertices[0],
            face_to_split_vertices[3],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(dcel.vertices().num_elements(), 30);
        assert_eq!(dcel.half_edges().num_elements(), 78);
        assert_eq!(dcel.faces().num_elements(), 11);

        // The sourceal hexagon is now split into two quads.
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
        let face_to_split_vertices: Vec<VertexId> = dcel.face_vertices(face_to_split).collect();

        // Split face 5 in two with a chain of two edges, with their common
        // point around the face's center.
        let (_new_edges, _new_face) = dcel.split_face_by_edge_chain(
            face_to_split_vertices[0],
            face_to_split_vertices[3],
            [(259, 150)],
            face_to_split,
        );

        // There are now eleven faces in total: one unbounded and ten bounded.
        assert_eq!(dcel.vertices().num_elements(), 31);
        assert_eq!(dcel.half_edges().num_elements(), 80);
        assert_eq!(dcel.faces().num_elements(), 11);

        // The sourceal hexagon is now split into two pentagons.

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
