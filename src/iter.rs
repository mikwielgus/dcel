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
        FaceEdgesIter, FaceEdgesReverseIter, FaceEdgesReverseWalker, FaceEdgesWalker,
        FaceHalfEdgesIter, FaceHalfEdgesReverseIter, FaceHalfEdgesReverseWalker,
        FaceHalfEdgesWalker, FaceVertexesIter, FaceVertexesReverseIter, FaceVertexesReverseWalker,
        FaceVertexesWalker, HalfSpokesIter, HalfSpokesReverseIter, HalfSpokesReverseWalker,
        HalfSpokesWalker, InterspokesIter, InterspokesReverseIter, InterspokesReverseWalker,
        InterspokesWalker, SpokesIter, SpokesReverseIter, SpokesReverseWalker, SpokesWalker,
    },
};
impl<VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
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

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
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

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
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

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
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
    use crate::VertexId;

    use super::*;

    // Vertexes for a regular pentagon centered at (0,0) with radius 1.0.
    // Coordinates calculated as (cos(2πn/5), sin(2πn/5)) for n = 0 to 4.
    const PENTAGON_VERTEXES: [[f32; 2]; 5] = [
        [1.0, 0.0],               // 0 degrees.
        [0.309017, 0.9510565],    // 72 degrees.
        [-0.809017, 0.58778525],  // 144 degrees.
        [-0.809017, -0.58778525], // 216 degrees.
        [0.309017, -0.9510565],   // 288 degrees.
    ];

    const ADJOINED_SQUARES_2X2: [[[i32; 2]; 4]; 4] = [
        // Bottom-left square (CCW)
        [[-1, 0], [0, 0], [0, -1], [-1, -1]],
        // Bottom-right square (CCW)
        [[0, 0], [1, 0], [1, -1], [0, -1]],
        // Top-left square (CCW)
        [[-1, 1], [0, 1], [0, 0], [-1, 0]],
        // Top-right square (CCW)
        [[0, 1], [1, 1], [1, 0], [0, 0]],
    ];

    #[test]
    fn test_half_spokes() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.half_spokes(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.half_spokes(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.half_spokes(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_half_spokes_reverse() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.half_spokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.half_spokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.half_spokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_spokes() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.spokes(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.spokes(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.spokes(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(1))))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_spokes_reverse() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.spokes_reverse(
                    dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id)))
                )
                .collect::<Vec<EdgeId>>()
                .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.spokes_reverse(
                    dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id)))
                )
                .collect::<Vec<EdgeId>>()
                .len(),
                3
            );
        }

        assert_eq!(
            dcel.spokes_reverse(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(1))))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_interspokes() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.interspokes(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<FaceId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.interspokes(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<FaceId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.interspokes(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<FaceId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_interspokes_reverse() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.interspokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<FaceId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.interspokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<FaceId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.interspokes_reverse(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<FaceId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_iter_face_vertexes() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_vertexes(dcel.unbounded_face())
                .collect::<Vec<VertexId>>()
                .len(),
            0
        );
        assert_eq!(dcel.face_vertexes(face).collect::<Vec<VertexId>>().len(), 5);
    }

    #[test]
    fn test_iter_face_vertexes_reverse() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_vertexes_reverse(dcel.unbounded_face())
                .collect::<Vec<VertexId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_vertexes_reverse(face)
                .collect::<Vec<VertexId>>()
                .len(),
            5
        );
    }

    #[test]
    fn test_iter_face_half_edges() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_half_edges(dcel.unbounded_face())
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_half_edges(face)
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            5
        );
    }

    #[test]
    fn test_iter_face_half_edges_reverse() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_half_edges_reverse(dcel.unbounded_face())
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_half_edges_reverse(face)
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            5
        );
    }

    #[test]
    fn test_iter_face_edges() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_edges(dcel.unbounded_face())
                .collect::<Vec<EdgeId>>()
                .len(),
            0
        );
        assert_eq!(dcel.face_edges(face).collect::<Vec<EdgeId>>().len(), 5);
    }

    #[test]
    fn test_iter_face_edges_reverse() {
        let mut dcel = Dcel::<[f32; 2]>::new();
        let face = dcel.insert_polygon(PENTAGON_VERTEXES);

        assert_eq!(
            dcel.face_edges_reverse(dcel.unbounded_face())
                .collect::<Vec<EdgeId>>()
                .len(),
            0
        );
        assert_eq!(
            dcel.face_edges_reverse(face).collect::<Vec<EdgeId>>().len(),
            5
        );
    }
}
