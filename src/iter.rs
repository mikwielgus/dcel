// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::Get;

use crate::{
    Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId,
    walkers::{
        CirculateEdgesWithExcludesIter, CirculateEdgesWithExcludesReverseIter,
        CirculateEdgesWithExcludesReverseWalker, CirculateEdgesWithExcludesWalker,
        CirculateHalfEdgesWithExcludesIter, CirculateHalfEdgesWithExcludesReverseIter,
        CirculateHalfEdgesWithExcludesReverseWalker, CirculateHalfEdgesWithExcludesWalker,
        CirculateVertexesWithExcludesIter, CirculateVertexesWithExcludesReverseIter,
        CirculateVertexesWithExcludesReverseWalker, CirculateVertexesWithExcludesWalker,
        FaceEdgesIter, FaceEdgesReverseIter, FaceEdgesReverseWalker, FaceEdgesWalker,
        FaceHalfEdgesIter, FaceHalfEdgesReverseIter, FaceHalfEdgesReverseWalker,
        FaceHalfEdgesWalker, FaceVertexesIter, FaceVertexesReverseIter, FaceVertexesReverseWalker,
        FaceVertexesWalker, HalfSpokesIter, HalfSpokesReverseIter, HalfSpokesReverseWalker,
        HalfSpokesWalker, InterspokesIter, InterspokesReverseIter, InterspokesReverseWalker,
        InterspokesWalker, SpokesIter, SpokesReverseIter, SpokesReverseWalker, SpokesWalker,
    },
};

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn circulate_vertexes_with_excludes(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_vertexes: impl IntoIterator<Item = VertexId>,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesWalker {
            initial_vertex: self.origin(initial_half_edge),
            curr_half_edge: Some(initial_half_edge),
            excluded_vertexes: excluded_vertexes.into_iter().collect(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_vertexes_with_excludes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_vertexes: impl IntoIterator<Item = VertexId>,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesReverseWalker {
            initial_vertex: self.origin(initial_half_edge),
            curr_half_edge: Some(initial_half_edge),
            excluded_vertexes: excluded_vertexes.into_iter().collect(),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_vertexes(
        &self,
        vertex: VertexId,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.turn_back_half_edge(self.incoming_next_half_edge(vertex));
        let initial_vertex = self.origin(initial_half_edge);

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if self.vertex_spokes(vertex).collect::<Vec<EdgeId>>().len() <= 2 {
            return CirculateVertexesWithExcludesWalker {
                initial_vertex,
                curr_half_edge: None,
                excluded_vertexes: vec![vertex],
            }
            .iter(self);
        }

        CirculateVertexesWithExcludesWalker {
            initial_vertex,
            curr_half_edge: Some(initial_half_edge),
            excluded_vertexes: vec![vertex],
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_rim_vertexes_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.turn_half_edge(self.incoming_next_half_edge(vertex));
        let initial_vertex = self.origin(initial_half_edge);

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if self.vertex_spokes(vertex).collect::<Vec<EdgeId>>().len() <= 2 {
            return CirculateVertexesWithExcludesReverseWalker {
                initial_vertex,
                curr_half_edge: None,
                excluded_vertexes: vec![vertex],
            }
            .iter(self);
        }

        CirculateVertexesWithExcludesReverseWalker {
            initial_vertex,
            curr_half_edge: Some(initial_half_edge),
            excluded_vertexes: vec![vertex],
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertex_half_spokes(
        &self,
        vertex: VertexId,
    ) -> HalfSpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.outgoing_next_half_edge(vertex);

        HalfSpokesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_half_spokes_reverse(
        &self,
        vertex: VertexId,
    ) -> HalfSpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.outgoing_next_half_edge(vertex);

        HalfSpokesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_half_edges(
        &self,
        vertex: VertexId,
    ) -> CirculateHalfEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.turn_back_half_edge(self.incoming_next_half_edge(vertex));
        let twin_half_spokes = self
            .vertex_half_spokes(vertex)
            .map(|half_edge| self.twin(half_edge))
            .collect::<Vec<HalfEdgeId>>();

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if twin_half_spokes.len() <= 2 {
            return CirculateHalfEdgesWithExcludesWalker {
                initial_half_edge,
                curr_half_edge: None,
                excluded_half_edges: twin_half_spokes,
            }
            .iter(self);
        }

        CirculateHalfEdgesWithExcludesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
            excluded_half_edges: twin_half_spokes,
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_rim_half_edges_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateHalfEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.turn_half_edge(self.incoming_next_half_edge(vertex));
        let twin_half_spokes = self
            .vertex_half_spokes(vertex)
            .map(|half_edge| self.twin(half_edge))
            .collect::<Vec<HalfEdgeId>>();

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if twin_half_spokes.len() <= 2 {
            return CirculateHalfEdgesWithExcludesReverseWalker {
                initial_half_edge,
                curr_half_edge: None,
                excluded_half_edges: twin_half_spokes,
            }
            .iter(self);
        }

        CirculateHalfEdgesWithExcludesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
            excluded_half_edges: twin_half_spokes,
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn half_spokes(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> HalfSpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        HalfSpokesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn half_spokes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> HalfSpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        HalfSpokesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_spokes(&self, vertex: VertexId) -> SpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_edge = self.vertex_next_edge(vertex);

        SpokesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_spokes_reverse(
        &self,
        vertex: VertexId,
    ) -> SpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_edge = self.vertex_next_edge(vertex);

        SpokesReverseWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_edges(
        &self,
        vertex: VertexId,
    ) -> CirculateEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_edge = self.turn_back_edge(self.reverse_edge(self.vertex_next_edge(vertex)));
        let spokes = self.vertex_spokes(vertex).collect::<Vec<EdgeId>>();

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if spokes.len() <= 2 {
            return CirculateEdgesWithExcludesWalker {
                initial_edge,
                curr_edge: None,
                excluded_edges: spokes,
            }
            .iter(self);
        }

        CirculateEdgesWithExcludesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
            excluded_edges: spokes,
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_rim_edges_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_edge = self.turn_edge(self.reverse_edge(self.vertex_next_edge(vertex)));
        let spokes = self.vertex_spokes(vertex).collect::<Vec<EdgeId>>();

        // If there is only two spokes or less, there is no cycle to complete
        // the walk. To avoid causing an infinite loop, return an empty
        // walker-iterator instead.
        if spokes.len() <= 2 {
            return CirculateEdgesWithExcludesReverseWalker {
                initial_edge,
                curr_edge: None,
                excluded_edges: spokes,
            }
            .iter(self);
        }

        CirculateEdgesWithExcludesReverseWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
            excluded_edges: spokes,
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn spokes(&self, initial_edge: EdgeId) -> SpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        SpokesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn spokes_reverse(
        &self,
        initial_edge: EdgeId,
    ) -> SpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        SpokesReverseWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn interspokes(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> InterspokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        InterspokesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn interspokes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> InterspokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        InterspokesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn circulate_half_edges_with_excludes(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_half_edges: impl IntoIterator<Item = HalfEdgeId>,
    ) -> CirculateHalfEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateHalfEdgesWithExcludesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
            excluded_half_edges: excluded_half_edges.into_iter().collect(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_half_edges_with_excludes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_half_edges: impl IntoIterator<Item = HalfEdgeId>,
    ) -> CirculateHalfEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateHalfEdgesWithExcludesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
            excluded_half_edges: excluded_half_edges.into_iter().collect(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_edges_with_excludes(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> CirculateEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateEdgesWithExcludesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
            excluded_edges: excluded_edges.into_iter().collect(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_edges_with_excludes_reverse(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> CirculateEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateEdgesWithExcludesReverseWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
            excluded_edges: excluded_edges.into_iter().collect(),
        }
        .iter(self)
    }
}

impl<
    VW,
    HEW,
    FW,
    VC: Get<usize, Item = Vertex<VW>>,
    HEC: Get<usize, Item = HalfEdge<HEW>>,
    FC: Get<usize, Item = Face<FW>>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn face_vertexes(&self, face: FaceId) -> FaceVertexesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no vertexes. Since the unbounded face is supposed
        // to behave similarly to other faces, it is better to branch out here
        // than to have the code below panic.
        if face == self.unbounded_face() {
            return FaceVertexesWalker {
                face_half_edges_walker: FaceHalfEdgesWalker {
                    // Uninitialized half-edge.
                    initial_half_edge: HalfEdgeId::new(0),
                    // Setting `curr_vertex` to None makes the iterator produce no
                    // elements.
                    curr_half_edge: None,
                },
            }
            .iter(self);
        }

        let initial_half_edge = self
            .faces
            .get(&face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        FaceVertexesWalker {
            face_half_edges_walker: FaceHalfEdgesWalker {
                initial_half_edge,
                curr_half_edge: Some(initial_half_edge),
            },
        }
        .iter(self)
    }

    #[inline]
    pub fn face_vertexes_reverse(
        &self,
        face: FaceId,
    ) -> FaceVertexesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no vertexes. Since the unbounded face is supposed
        // to behave similarly to other faces, it is better to branch out here
        // than to have the code below panic.
        if face == self.unbounded_face() {
            return FaceVertexesReverseWalker {
                face_half_edges_reverse_walker: FaceHalfEdgesReverseWalker {
                    // Uninitialized half-edge.
                    initial_half_edge: HalfEdgeId::new(0),
                    // Setting `curr_vertex` to None makes the iterator produce no
                    // elements.
                    curr_half_edge: None,
                },
            }
            .iter(self);
        }

        let initial_half_edge = self
            .faces
            .get(&face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        FaceVertexesReverseWalker {
            face_half_edges_reverse_walker: FaceHalfEdgesReverseWalker {
                initial_half_edge,
                curr_half_edge: Some(initial_half_edge),
            },
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn face_half_edges(&self, face: FaceId) -> FaceHalfEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no half-edges. Since the unbounded face is
        // supposed to behave similarly to other faces, it is better to branch
        // out here than to have the code below panic.
        if face == self.unbounded_face() {
            return FaceHalfEdgesWalker {
                // Uninitialized half-edge.
                initial_half_edge: HalfEdgeId::new(0),
                // Setting `curr_edge` to None makes the iterator produce no
                // elements.
                curr_half_edge: None,
            }
            .iter(self);
        }

        let initial_half_edge = self
            .faces
            .get(&face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        FaceHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn face_half_edges_reverse(
        &self,
        face: FaceId,
    ) -> FaceHalfEdgesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no half-edges. Since the unbounded face is
        // supposed to behave similarly to other faces, it is better to branch
        // out here than to have the code below panic.
        if face == self.unbounded_face() {
            return FaceHalfEdgesReverseWalker {
                // Uninitialized half-edge.
                initial_half_edge: HalfEdgeId::new(0),
                // Setting `curr_edge` to None makes the iterator produce no
                // elements.
                curr_half_edge: None,
            }
            .iter(self);
        }

        let initial_half_edge = self
            .faces
            .get(&face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        FaceHalfEdgesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn face_edges(&self, face: FaceId) -> FaceEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no edges. Since the unbounded face is supposed to
        // behave similarly to other faces, it is better to branch out here than
        // to have the code below panic.
        if face == self.unbounded_face() {
            return FaceEdgesWalker {
                // Uninitialized edge.
                initial_edge: EdgeId::new(HalfEdgeId::new(0), HalfEdgeId::new(0)),
                // Setting `curr_edge` to None makes the iterator produce no
                // elements.
                curr_edge: None,
            }
            .iter(self);
        }

        let initial_edge = self.full_edge(
            self.faces
                .get(&face.id())
                .unwrap()
                .incident_half_edge
                .unwrap(),
        );

        FaceEdgesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn face_edges_reverse(
        &self,
        face: FaceId,
    ) -> FaceEdgesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        // Unbounded face has no edges. Since the unbounded face is supposed to
        // behave similarly to other faces, it is better to branch out here than
        // to have the code below panic.
        if face == self.unbounded_face() {
            return FaceEdgesReverseWalker {
                // Uninitialized edge.
                initial_edge: EdgeId::new(HalfEdgeId::new(0), HalfEdgeId::new(0)),
                // Setting `curr_edge` to None makes the iterator produce no
                // elements.
                curr_edge: None,
            }
            .iter(self);
        }

        let initial_edge = self.full_edge(
            self.faces
                .get(&face.id())
                .unwrap()
                .incident_half_edge
                .unwrap(),
        );

        FaceEdgesReverseWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    const HEX_MESH_3X3: [[[(i32, i32); 6]; 3]; 3] = [
        [
            [
                (87, -50),
                (0, -100),
                (-87, -50),
                (-87, 50),
                (0, 100),
                (87, 50),
            ],
            [
                (260, -50),
                (173, -100),
                (87, -50),
                (87, 50),
                (173, 100),
                (260, 50),
            ],
            [
                (433, -50),
                (346, -100),
                (260, -50),
                (260, 50),
                (346, 100),
                (433, 50),
            ],
        ],
        [
            [
                (173, 100),
                (87, 50),
                (0, 100),
                (0, 200),
                (87, 250),
                (173, 200),
            ],
            [
                (346, 100),
                (260, 50),
                (173, 100),
                (173, 200),
                (260, 250),
                (346, 200),
            ],
            [
                (520, 100),
                (433, 50),
                (346, 100),
                (346, 200),
                (433, 250),
                (520, 200),
            ],
        ],
        [
            [
                (87, 250),
                (0, 200),
                (-87, 250),
                (-87, 350),
                (0, 400),
                (87, 350),
            ],
            [
                (260, 250),
                (173, 200),
                (87, 250),
                (87, 350),
                (173, 400),
                (260, 350),
            ],
            [
                (433, 250),
                (346, 200),
                (260, 250),
                (260, 350),
                (346, 400),
                (433, 350),
            ],
        ],
    ];

    fn init_dcel_with_3x3_hex_mesh() -> Dcel<(i32, i32)> {
        let mesh: Vec<Vec<(i32, i32)>> = HEX_MESH_3X3
            .iter()
            .flat_map(|row| row.iter())
            .map(|face| face.iter().copied().collect())
            .collect();
        let mut dcel: Dcel<(i32, i32)> = Dcel::new();
        dcel.insert_mesh(mesh);

        dcel
    }

    #[test]
    fn test_vertex_rim_vertexes() {
        // TODO.
    }

    #[test]
    fn test_vertex_half_spokes() {}
}
