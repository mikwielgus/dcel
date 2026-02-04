// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::Get;

use crate::{
    Dcel, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId,
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

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_vertexes(
        &self,
        vertex: VertexId,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesWalker {
            circulator: self.vertex_rim_half_edges(vertex).walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_rim_vertexes_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesReverseWalker {
            circulator: self.vertex_rim_half_edges_reverse(vertex).walker(),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn vertex_half_spokes(
        &self,
        vertex: VertexId,
    ) -> HalfSpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.half_spokes(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    pub fn vertex_half_spokes_reverse(
        &self,
        vertex: VertexId,
    ) -> HalfSpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.half_spokes_reverse(self.outgoing_next_half_edge(vertex))
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_half_edges(
        &self,
        vertex: VertexId,
    ) -> CirculateHalfEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.next_half_edge(self.outgoing_next_half_edge(vertex));
        let excluded_half_edges = self
            .vertex_spokes(vertex)
            .flat_map(|edge| [edge.lesser(), edge.greater()])
            .collect::<Vec<HalfEdgeId>>();

        // Boundary vertexes do not have cycles circulating them.
        // Return an empty walker-iterator instead.
        if self.is_boundary_vertex(vertex) {
            return CirculateHalfEdgesWithExcludesWalker {
                initial_half_edge,
                curr_half_edge: None,
                excluded_half_edges,
            }
            .iter(self);
        }

        self.circulate_half_edges_with_excludes(initial_half_edge, excluded_half_edges)
    }

    #[inline]
    pub fn vertex_rim_half_edges_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateHalfEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self.prev_half_edge(self.incoming_next_half_edge(vertex));
        let excluded_half_edges = self
            .vertex_spokes(vertex)
            .flat_map(|edge| [edge.lesser(), edge.greater()])
            .collect::<Vec<HalfEdgeId>>();

        // Boundary vertexes do not have cycles circulating them.
        // Return an empty walker-iterator instead.
        if self.is_boundary_vertex(vertex) {
            return CirculateHalfEdgesWithExcludesReverseWalker {
                initial_half_edge,
                curr_half_edge: None,
                excluded_half_edges,
            }
            .iter(self);
        }

        self.circulate_half_edges_with_excludes_reverse(initial_half_edge, excluded_half_edges)
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

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_spokes(&self, vertex: VertexId) -> SpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.spokes(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    pub fn vertex_spokes_reverse(
        &self,
        vertex: VertexId,
    ) -> SpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.spokes_reverse(self.outgoing_next_half_edge(vertex))
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_rim_edges(
        &self,
        vertex: VertexId,
    ) -> CirculateEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateEdgesWithExcludesWalker {
            circulator: self.vertex_rim_half_edges(vertex).walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn vertex_rim_edges_reverse(
        &self,
        vertex: VertexId,
    ) -> CirculateEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateEdgesWithExcludesReverseWalker {
            circulator: self.vertex_rim_half_edges_reverse(vertex).walker(),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn vertex_interspokes(
        &self,
        vertex: VertexId,
    ) -> InterspokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.interspokes(self.outgoing_next_half_edge(vertex))
    }

    #[inline]
    pub fn vertex_interspokes_reverse(
        &self,
        vertex: VertexId,
    ) -> InterspokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.interspokes_reverse(self.outgoing_next_half_edge(vertex))
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn spokes(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> SpokesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        SpokesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }

    #[inline]
    pub fn spokes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> SpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        SpokesReverseWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
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

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn circulate_vertexes(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.circulate_vertexes_with_excludes(initial_half_edge, std::iter::empty())
    }

    #[inline]
    pub fn circulate_vertexes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.circulate_vertexes_with_excludes_reverse(initial_half_edge, std::iter::empty())
    }

    #[inline]
    pub fn circulate_vertexes_with_excludes(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_half_edges: impl IntoIterator<Item = HalfEdgeId>,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesWalker {
            circulator: self
                .circulate_half_edges_with_excludes(initial_half_edge, excluded_half_edges)
                .walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_vertexes_with_excludes_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
        excluded_half_edges: impl IntoIterator<Item = HalfEdgeId>,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesReverseWalker {
            circulator: self
                .circulate_half_edges_with_excludes_reverse(initial_half_edge, excluded_half_edges)
                .walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_half_edges(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CirculateHalfEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.circulate_half_edges_with_excludes(initial_half_edge, std::iter::empty())
    }

    #[inline]
    pub fn circulate_half_edges_reverse(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CirculateHalfEdgesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.circulate_half_edges_with_excludes_reverse(initial_half_edge, std::iter::empty())
    }

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
}

impl<
    VW,
    HEW,
    FW,
    VC: Get<usize, Value = Vertex<VW>>,
    HEC: Get<usize, Value = HalfEdge<HEW>>,
    FC: Get<usize, Value = Face<FW>>,
> Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn face_vertexes(&self, face: FaceId) -> FaceVertexesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        FaceVertexesWalker {
            circulator: self.face_half_edges(face).walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn face_vertexes_reverse(
        &self,
        face: FaceId,
    ) -> FaceVertexesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        FaceVertexesReverseWalker {
            circulator: self.face_half_edges_reverse(face).walker(),
        }
        .iter(self)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Value = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
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

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC: Get<usize, Value = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn face_edges(&self, face: FaceId) -> FaceEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        FaceEdgesWalker {
            circulator: self.face_half_edges(face).walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn face_edges_reverse(
        &self,
        face: FaceId,
    ) -> FaceEdgesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        FaceEdgesReverseWalker {
            circulator: self.face_half_edges_reverse(face).walker(),
        }
        .iter(self)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        Dcel, assert_face_boundary, assert_vertex_rim, assert_vertex_spokes_interspokes,
        init_dcel_with_3x3_hex_mesh,
    };

    #[test]
    fn test_vertex_rim() {
        let dcel = init_dcel_with_3x3_hex_mesh!(Dcel<(i32, i32)>);

        assert_vertex_rim!(&dcel, 0, 0);
        assert_vertex_rim!(&dcel, 1, 0);
        assert_vertex_rim!(&dcel, 2, 0);
        assert_vertex_rim!(&dcel, 3, 0);
        assert_vertex_rim!(&dcel, 4, 0);
        assert_vertex_rim!(&dcel, 5, 12);
        assert_vertex_rim!(&dcel, 6, 0);
        assert_vertex_rim!(&dcel, 7, 0);
        assert_vertex_rim!(&dcel, 8, 12);
        assert_vertex_rim!(&dcel, 9, 12);
        assert_vertex_rim!(&dcel, 10, 0);
        assert_vertex_rim!(&dcel, 11, 0);
        assert_vertex_rim!(&dcel, 12, 12);
        assert_vertex_rim!(&dcel, 13, 0);
        assert_vertex_rim!(&dcel, 14, 0);
        assert_vertex_rim!(&dcel, 15, 12);
        assert_vertex_rim!(&dcel, 16, 12);
        assert_vertex_rim!(&dcel, 17, 12);
        assert_vertex_rim!(&dcel, 18, 12);
        assert_vertex_rim!(&dcel, 19, 0);
        assert_vertex_rim!(&dcel, 20, 0);
        assert_vertex_rim!(&dcel, 21, 0);
        assert_vertex_rim!(&dcel, 22, 0);
        assert_vertex_rim!(&dcel, 23, 0);
        assert_vertex_rim!(&dcel, 24, 0);
        assert_vertex_rim!(&dcel, 25, 0);
        assert_vertex_rim!(&dcel, 26, 0);
        assert_vertex_rim!(&dcel, 27, 0);
        assert_vertex_rim!(&dcel, 28, 0);
        assert_vertex_rim!(&dcel, 29, 0);
    }

    #[test]
    fn test_vertex_spokes_interspokes() {
        let dcel = init_dcel_with_3x3_hex_mesh!(Dcel<(i32, i32)>);

        assert_vertex_spokes_interspokes!(&dcel, 0, 3);
        assert_vertex_spokes_interspokes!(&dcel, 1, 2);
        assert_vertex_spokes_interspokes!(&dcel, 2, 2);
        assert_vertex_spokes_interspokes!(&dcel, 3, 2);
        assert_vertex_spokes_interspokes!(&dcel, 4, 3);
        assert_vertex_spokes_interspokes!(&dcel, 5, 3);
        assert_vertex_spokes_interspokes!(&dcel, 6, 3);
        assert_vertex_spokes_interspokes!(&dcel, 7, 2);
        assert_vertex_spokes_interspokes!(&dcel, 8, 3);
        assert_vertex_spokes_interspokes!(&dcel, 9, 3);
        assert_vertex_spokes_interspokes!(&dcel, 10, 2);
        assert_vertex_spokes_interspokes!(&dcel, 11, 2);
        assert_vertex_spokes_interspokes!(&dcel, 12, 3);
        assert_vertex_spokes_interspokes!(&dcel, 13, 3);
        assert_vertex_spokes_interspokes!(&dcel, 14, 3);
        assert_vertex_spokes_interspokes!(&dcel, 15, 3);
        assert_vertex_spokes_interspokes!(&dcel, 16, 3);
        assert_vertex_spokes_interspokes!(&dcel, 17, 3);
        assert_vertex_spokes_interspokes!(&dcel, 18, 3);
        assert_vertex_spokes_interspokes!(&dcel, 19, 2);
        assert_vertex_spokes_interspokes!(&dcel, 20, 3);
        assert_vertex_spokes_interspokes!(&dcel, 21, 2);
        assert_vertex_spokes_interspokes!(&dcel, 22, 2);
        assert_vertex_spokes_interspokes!(&dcel, 23, 2);
        assert_vertex_spokes_interspokes!(&dcel, 24, 2);
        assert_vertex_spokes_interspokes!(&dcel, 25, 3);
        assert_vertex_spokes_interspokes!(&dcel, 26, 2);
        assert_vertex_spokes_interspokes!(&dcel, 27, 3);
        assert_vertex_spokes_interspokes!(&dcel, 28, 2);
        assert_vertex_spokes_interspokes!(&dcel, 29, 2);
    }

    #[test]
    fn test_face_boundary() {
        let dcel = init_dcel_with_3x3_hex_mesh!(Dcel<(i32, i32)>);

        assert_face_boundary!(&dcel, 0, 0);
        assert_face_boundary!(&dcel, 1, 6);
        assert_face_boundary!(&dcel, 2, 6);
        assert_face_boundary!(&dcel, 3, 6);
        assert_face_boundary!(&dcel, 4, 6);
        assert_face_boundary!(&dcel, 5, 6);
        assert_face_boundary!(&dcel, 6, 6);
        assert_face_boundary!(&dcel, 7, 6);
        assert_face_boundary!(&dcel, 8, 6);
        assert_face_boundary!(&dcel, 9, 6);
    }
}
