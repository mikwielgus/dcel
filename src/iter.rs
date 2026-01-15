// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

use maplike::Get;

use crate::{Dcel, EdgeId, Face, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

macro_rules! create_walker_and_iter {
    ($walker:ident { $($field:ident: $type:ty),* $(,)? }, $iter:ident) => {
        pub struct $walker {
            $(pub $field: $type,)*
        }

        impl $walker {
            pub fn iter<'a, VW, HEW, FW, VC, HEC, FC>(
                self,
                dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
            ) -> $iter<'a, VW, HEW, FW, VC, HEC, FC> {
                $iter { walker: self, dcel }
            }
        }

        pub struct $iter<'a, VW, HEW, FW, VC, HEC, FC> {
            walker: $walker,
            dcel: &'a Dcel<VW, HEW, FW, VC, HEC, FC>,
        }

        impl<'a, VW, HEW, FW, VC, HEC, FC> $iter<'a, VW, HEW, FW, VC, HEC, FC> {
            pub fn walker(self) -> $walker {
                self.walker
            }
        }
    };
}

create_walker_and_iter!(
    FaceVertexesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    FaceVertexesIter
);

impl FaceVertexesWalker {
    #[inline]
    pub fn next<
        VW,
        HEW,
        FW,
        VC: Get<usize, Item = Vertex<VW>>,
        HEC: Get<usize, Item = HalfEdge<HEW>>,
        FC,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<VertexId> {
        let next_half_edge = dcel.next_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
        .map(|half_edge| dcel.origin(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC: Get<usize, Item = Vertex<VW>>, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>
    Iterator for FaceVertexesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = VertexId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
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
                // Uninitialized half-edge.
                initial_half_edge: HalfEdgeId::new(0),
                // Setting `curr_vertex` to None makes the iterator produce no
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

        FaceVertexesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    FaceHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    FaceHalfEdgesIter
);

impl FaceHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.next_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for FaceHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
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
}

create_walker_and_iter!(
    FaceEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
    },
    FaceEdgesIter
);

impl FaceEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.next_edge(self.curr_edge?);

        std::mem::replace(
            &mut self.curr_edge,
            (next_edge != self.initial_edge).then_some(next_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for FaceEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
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
}

create_walker_and_iter!(
    CwHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    CwHalfEdgesIter
);

impl CwHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.cw_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn cw_half_edges(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CwHalfEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CwHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    CcwHalfEdgesIter
);

impl CcwHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.ccw_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CcwHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC, FC: Get<usize, Item = Face<FW>>> Dcel<VW, HEW, FW, VC, HEC, FC> {
    #[inline]
    pub fn ccw_half_edges(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CcwHalfEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CcwHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CwEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
    },
    CwEdgesIter
);

impl CwEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.cw_edge(self.curr_edge?);

        std::mem::replace(
            &mut self.curr_edge,
            (next_edge != self.initial_edge).then_some(next_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    #[inline]
    pub fn cw_edges(&self, initial_edge: EdgeId) -> CwEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CwEdgesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
    },
    CcwEdgesIter
);

impl CcwEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.ccw_edge(self.curr_edge?);

        std::mem::replace(
            &mut self.curr_edge,
            (next_edge != self.initial_edge).then_some(next_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CcwEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn ccw_edges(&self, initial_edge: EdgeId) -> CcwEdgesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CcwEdgesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    EdgesWithExcludesWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
        excluded_edges: Vec<EdgeId>,
    },
    EdgesWithExcludesIter
);

impl EdgesWithExcludesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let mut candidate_next_edge = dcel.next_edge(self.curr_edge?);

        while self.excluded_edges.contains(&candidate_next_edge) {
            candidate_next_edge = dcel.ccw_edge(candidate_next_edge);
        }

        let next_edge = candidate_next_edge;

        std::mem::replace(
            &mut self.curr_edge,
            (next_edge != self.initial_edge).then_some(next_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for EdgesWithExcludesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn edges_with_excludes(
        &self,
        initial_edge: EdgeId,
        excluded_edges: impl IntoIterator<Item = EdgeId>,
    ) -> EdgesWithExcludesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        EdgesWithExcludesWalker {
            initial_edge,
            curr_edge: Some(initial_edge),
            excluded_edges: excluded_edges.into_iter().collect(),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CwFacesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    CwFacesIter
);

impl CwFacesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.cw_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CwFacesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = FaceId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next(self.dcel)
            .map(|half_edge| self.dcel.face_in_front(half_edge))
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn cw_faces(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CwFacesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CwFacesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwFacesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    CcwFacesIter
);

impl CcwFacesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.ccw_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC> Iterator
    for CcwFacesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = FaceId;

    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next(self.dcel)
            .map(|half_edge| self.dcel.face_in_front(half_edge))
    }
}

impl<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC: Get<usize, Item = Face<FW>>>
    Dcel<VW, HEW, FW, VC, HEC, FC>
{
    pub fn ccw_faces(
        &self,
        initial_half_edge: HalfEdgeId,
    ) -> CcwFacesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        CcwFacesWalker {
            initial_half_edge,
            curr_half_edge: Some(initial_half_edge),
        }
        .iter(self)
    }
}

#[cfg(test)]
mod test {
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
    fn test_ccw_half_edges() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.ccw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.ccw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.ccw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_cw_half_edges() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.cw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.cw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(id)))
                    .collect::<Vec<HalfEdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.cw_half_edges(dcel.outgoing_next_half_edge(VertexId::new(1)))
                .collect::<Vec<HalfEdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_ccw_edges() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.ccw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.ccw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.ccw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(1))))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
    }

    #[test]
    fn test_cw_edges() {
        let mut dcel = Dcel::<[i32; 2]>::new();
        dcel.insert_mesh(ADJOINED_SQUARES_2X2);

        for id in [3, 5, 6, 8] {
            assert_eq!(
                dcel.cw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                2
            );
        }

        for id in [0, 2, 4, 7] {
            assert_eq!(
                dcel.cw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(id))))
                    .collect::<Vec<EdgeId>>()
                    .len(),
                3
            );
        }

        assert_eq!(
            dcel.cw_edges(dcel.full_edge(dcel.outgoing_next_half_edge(VertexId::new(1))))
                .collect::<Vec<EdgeId>>()
                .len(),
            4
        );
    }
}
