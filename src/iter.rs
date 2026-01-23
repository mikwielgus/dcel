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
            .flat_map(|edge| [edge.forward(), edge.backward()])
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
            .flat_map(|edge| [edge.forward(), edge.backward()])
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
        self.spokes(self.vertex_next_edge(vertex))
    }

    #[inline]
    pub fn vertex_spokes_reverse(
        &self,
        vertex: VertexId,
    ) -> SpokesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        self.spokes_reverse(self.vertex_next_edge(vertex))
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

impl<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn circulate_vertexes_with_excludes(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> CirculateVertexesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesWalker {
            circulator: self
                .circulate_half_edges_with_excludes(
                    initial_edge.forward(),
                    excluded_edges
                        .into_iter()
                        .flat_map(|edge| [edge.forward(), edge.backward()]),
                )
                .walker(),
        }
        .iter(self)
    }

    #[inline]
    pub fn circulate_vertexes_with_excludes_reverse(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> CirculateVertexesWithExcludesReverseIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateVertexesWithExcludesReverseWalker {
            circulator: self
                .circulate_half_edges_with_excludes_reverse(
                    initial_edge.forward(),
                    excluded_edges
                        .into_iter()
                        .flat_map(|edge| [edge.forward(), edge.backward()]),
                )
                .walker(),
        }
        .iter(self)
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

    #[inline]
    pub fn circulate_edges_with_excludes(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> CirculateEdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CirculateEdgesWithExcludesWalker {
            circulator: self
                .circulate_half_edges_with_excludes(
                    initial_edge.forward(),
                    excluded_edges
                        .into_iter()
                        .flat_map(|edge| [edge.forward(), edge.backward()]),
                )
                .walker(),
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
            circulator: self
                .circulate_half_edges_with_excludes_reverse(
                    initial_edge.forward(),
                    excluded_edges
                        .into_iter()
                        .flat_map(|edge| [edge.forward(), edge.backward()]),
                )
                .walker(),
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
    use super::*;
    use crate::test_common;

    #[test]
    fn test_vertex_rim() {
        let dcel = test_common::init_dcel_with_3x3_hex_mesh();

        let assert_vertex_rim = |id: usize, count: usize| {
            let vertex_rim_vertexes: Vec<VertexId> =
                dcel.vertex_rim_vertexes(VertexId::new(id)).collect();
            assert_eq!(vertex_rim_vertexes.len(), count);

            let vertex_rim_vertexes_reverse: Vec<VertexId> = dcel
                .vertex_rim_vertexes_reverse(VertexId::new(id))
                .collect();
            assert_eq!(
                vertex_rim_vertexes,
                vertex_rim_vertexes_reverse
                    .into_iter()
                    .rev()
                    .collect::<Vec<VertexId>>()
            );

            let vertex_rim_half_edges: Vec<HalfEdgeId> =
                dcel.vertex_rim_half_edges(VertexId::new(id)).collect();
            assert_eq!(vertex_rim_half_edges.len(), count);

            let vertex_rim_half_edges_reverse: Vec<HalfEdgeId> = dcel
                .vertex_rim_half_edges_reverse(VertexId::new(id))
                .collect();
            assert_eq!(
                vertex_rim_half_edges,
                vertex_rim_half_edges_reverse
                    .into_iter()
                    .rev()
                    .collect::<Vec<HalfEdgeId>>()
            );

            let vertex_rim_edges: Vec<EdgeId> = dcel.vertex_rim_edges(VertexId::new(id)).collect();
            assert_eq!(vertex_rim_edges.len(), count);

            let vertex_rim_edges_reverse: Vec<EdgeId> =
                dcel.vertex_rim_edges_reverse(VertexId::new(id)).collect();
            assert_eq!(
                vertex_rim_edges,
                vertex_rim_edges_reverse
                    .into_iter()
                    .rev()
                    .collect::<Vec<EdgeId>>()
            );
        };

        assert_vertex_rim(0, 0);
        assert_vertex_rim(1, 0);
        assert_vertex_rim(2, 0);
        assert_vertex_rim(3, 0);
        assert_vertex_rim(4, 0);
        assert_vertex_rim(5, 12);
        assert_vertex_rim(6, 0);
        assert_vertex_rim(7, 0);
        assert_vertex_rim(8, 12);
        assert_vertex_rim(9, 12);
        assert_vertex_rim(10, 0);
        assert_vertex_rim(11, 0);
        assert_vertex_rim(12, 12);
        assert_vertex_rim(13, 0);
        assert_vertex_rim(14, 0);
        assert_vertex_rim(15, 12);
        assert_vertex_rim(16, 12);
        assert_vertex_rim(17, 12);
        assert_vertex_rim(18, 12);
        assert_vertex_rim(19, 0);
        assert_vertex_rim(20, 0);
        assert_vertex_rim(21, 0);
        assert_vertex_rim(22, 0);
        assert_vertex_rim(23, 0);
        assert_vertex_rim(24, 0);
        assert_vertex_rim(25, 0);
        assert_vertex_rim(26, 0);
        assert_vertex_rim(27, 0);
        assert_vertex_rim(28, 0);
        assert_vertex_rim(29, 0);
    }

    #[test]
    fn test_vertex_spokes_interspokes() {
        let dcel = test_common::init_dcel_with_3x3_hex_mesh();

        let assert_vertex_spokes_interspokes = |id: usize, count: usize| {
            let vertex_half_spokes: Vec<HalfEdgeId> =
                dcel.vertex_half_spokes(VertexId::new(id)).collect();
            assert_eq!(vertex_half_spokes.len(), count);

            let vertex_half_spokes_reverse: Vec<HalfEdgeId> =
                dcel.vertex_half_spokes_reverse(VertexId::new(id)).collect();
            assert_eq!(
                vertex_half_spokes.first(),
                vertex_half_spokes_reverse.first()
            );
            assert_eq!(
                vertex_half_spokes
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<HalfEdgeId>>(),
                vertex_half_spokes_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<HalfEdgeId>>()
            );

            let vertex_spokes: Vec<EdgeId> = dcel.vertex_spokes(VertexId::new(id)).collect();
            assert_eq!(vertex_spokes.len(), count);

            let vertex_spokes_reverse: Vec<EdgeId> =
                dcel.vertex_spokes_reverse(VertexId::new(id)).collect();
            assert_eq!(vertex_spokes.first(), vertex_spokes_reverse.first());
            assert_eq!(
                vertex_spokes.into_iter().skip(1).collect::<Vec<EdgeId>>(),
                vertex_spokes_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<EdgeId>>()
            );

            let vertex_interspokes: Vec<FaceId> =
                dcel.vertex_interspokes(VertexId::new(id)).collect();
            assert_eq!(vertex_interspokes.len(), count);

            let vertex_interspokes_reverse: Vec<FaceId> =
                dcel.vertex_interspokes_reverse(VertexId::new(id)).collect();
            assert_eq!(
                vertex_interspokes.first(),
                vertex_interspokes_reverse.first()
            );
            assert_eq!(
                vertex_interspokes
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<FaceId>>(),
                vertex_interspokes_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<FaceId>>()
            );
        };

        assert_vertex_spokes_interspokes(0, 3);
        assert_vertex_spokes_interspokes(1, 2);
        assert_vertex_spokes_interspokes(2, 2);
        assert_vertex_spokes_interspokes(3, 2);
        assert_vertex_spokes_interspokes(4, 3);
        assert_vertex_spokes_interspokes(5, 3);
        assert_vertex_spokes_interspokes(6, 3);
        assert_vertex_spokes_interspokes(7, 2);
        assert_vertex_spokes_interspokes(8, 3);
        assert_vertex_spokes_interspokes(9, 3);
        assert_vertex_spokes_interspokes(10, 2);
        assert_vertex_spokes_interspokes(11, 2);
        assert_vertex_spokes_interspokes(12, 3);
        assert_vertex_spokes_interspokes(13, 3);
        assert_vertex_spokes_interspokes(14, 3);
        assert_vertex_spokes_interspokes(15, 3);
        assert_vertex_spokes_interspokes(16, 3);
        assert_vertex_spokes_interspokes(17, 3);
        assert_vertex_spokes_interspokes(18, 3);
        assert_vertex_spokes_interspokes(19, 2);
        assert_vertex_spokes_interspokes(20, 3);
        assert_vertex_spokes_interspokes(21, 2);
        assert_vertex_spokes_interspokes(22, 2);
        assert_vertex_spokes_interspokes(23, 2);
        assert_vertex_spokes_interspokes(24, 2);
        assert_vertex_spokes_interspokes(25, 3);
        assert_vertex_spokes_interspokes(26, 2);
        assert_vertex_spokes_interspokes(27, 3);
        assert_vertex_spokes_interspokes(28, 2);
        assert_vertex_spokes_interspokes(29, 2);
    }

    #[test]
    fn test_face_boundary() {
        let dcel = test_common::init_dcel_with_3x3_hex_mesh();

        let assert_face_boundary = |id: usize, count: usize| {
            let face_vertexes: Vec<VertexId> = dcel.face_vertexes(FaceId::new(id)).collect();
            assert_eq!(face_vertexes.len(), count);

            let face_vertexes_reverse: Vec<VertexId> =
                dcel.face_vertexes_reverse(FaceId::new(id)).collect();
            assert_eq!(face_vertexes.first(), face_vertexes_reverse.first());
            assert_eq!(
                face_vertexes.into_iter().skip(1).collect::<Vec<VertexId>>(),
                face_vertexes_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<VertexId>>()
            );

            let face_half_edges: Vec<HalfEdgeId> = dcel.face_half_edges(FaceId::new(id)).collect();
            assert_eq!(face_half_edges.len(), count);

            let face_half_edges_reverse: Vec<HalfEdgeId> =
                dcel.face_half_edges_reverse(FaceId::new(id)).collect();
            assert_eq!(face_half_edges.first(), face_half_edges_reverse.first());
            assert_eq!(
                face_half_edges
                    .into_iter()
                    .skip(1)
                    .collect::<Vec<HalfEdgeId>>(),
                face_half_edges_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<HalfEdgeId>>()
            );

            let face_edges: Vec<EdgeId> = dcel.face_edges(FaceId::new(id)).collect();
            assert_eq!(face_edges.len(), count);

            let face_edges_reverse: Vec<EdgeId> =
                dcel.face_edges_reverse(FaceId::new(id)).collect();
            assert_eq!(face_edges.first(), face_edges_reverse.first());
            assert_eq!(
                face_edges.into_iter().skip(1).collect::<Vec<EdgeId>>(),
                face_edges_reverse
                    .into_iter()
                    .skip(1)
                    .rev()
                    .collect::<Vec<EdgeId>>()
            );
        };

        assert_face_boundary(0, 0);
        assert_face_boundary(1, 6);
        assert_face_boundary(2, 6);
        assert_face_boundary(3, 6);
        assert_face_boundary(4, 6);
        assert_face_boundary(5, 6);
        assert_face_boundary(6, 6);
        assert_face_boundary(7, 6);
        assert_face_boundary(8, 6);
        assert_face_boundary(9, 6);
    }
}
