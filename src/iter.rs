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
        initial_vertex: VertexId,
        curr_vertex: VertexId,
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
        let next_vertex = dcel.next_vertex(self.curr_vertex);

        (next_vertex != self.initial_vertex)
            .then(|| std::mem::replace(&mut self.curr_vertex, next_vertex))
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
        let initial_vertex = self.origin(
            self.faces
                .get(&face.id())
                .unwrap()
                .incident_half_edge
                .unwrap(),
        );

        FaceVertexesWalker {
            initial_vertex,
            curr_vertex: initial_vertex,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    FaceHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: HalfEdgeId,
    },
    FaceHalfEdgesIter
);

impl FaceHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.next_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
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
        let initial_half_edge = self
            .faces
            .get(&face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        FaceHalfEdgesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    FaceEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: EdgeId,
    },
    FaceEdgesIter
);

impl FaceEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.next_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
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
        let initial_edge = self.full_edge(
            self.faces
                .get(&face.id())
                .unwrap()
                .incident_half_edge
                .unwrap(),
        );

        FaceEdgesWalker {
            initial_edge,
            curr_edge: initial_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CwHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: HalfEdgeId,
    },
    CwHalfEdgesIter
);

impl CwHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.cw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
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
            curr_half_edge: initial_half_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwHalfEdgesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: HalfEdgeId,
    },
    CcwHalfEdgesIter
);

impl CcwHalfEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.ccw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
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
            curr_half_edge: initial_half_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CwEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: EdgeId,
    },
    CwEdgesIter
);

impl CwEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.cw_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
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
            curr_edge: initial_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwEdgesWalker {
        initial_edge: EdgeId,
        curr_edge: EdgeId,
    },
    CcwEdgesIter
);

impl CcwEdgesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let next_edge = dcel.ccw_edge(self.curr_edge);

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
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
            curr_edge: initial_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    EdgesWithExcludesWalker {
        initial_edge: EdgeId,
        curr_edge: EdgeId,
        excluded_edges: Vec<EdgeId>,
    },
    EdgesWithExcludesIter
);

impl EdgesWithExcludesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        let mut candidate_next_edge = dcel.next_edge(self.curr_edge);

        while self.excluded_edges.contains(&candidate_next_edge) {
            candidate_next_edge = dcel.ccw_edge(candidate_next_edge);
        }

        let next_edge = candidate_next_edge;

        (next_edge != self.initial_edge).then(|| std::mem::replace(&mut self.curr_edge, next_edge))
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
            curr_edge: initial_edge,
            excluded_edges: excluded_edges.into_iter().collect(),
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CwFacesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: HalfEdgeId,
    },
    CwFacesIter
);

impl CwFacesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.cw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
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
    pub fn cw_faces(&self, initial_face: FaceId) -> CwFacesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self
            .faces
            .get(&initial_face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        CwFacesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
        .iter(self)
    }
}

create_walker_and_iter!(
    CcwFacesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: HalfEdgeId,
    },
    CcwFacesIter
);

impl CcwFacesWalker {
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Item = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.ccw_half_edge(self.curr_half_edge);

        (next_half_edge != self.initial_half_edge)
            .then(|| std::mem::replace(&mut self.curr_half_edge, next_half_edge))
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
    pub fn ccw_faces(&self, initial_face: FaceId) -> CcwFacesIter<'_, VW, HEW, FW, VC, HEC, FC> {
        let initial_half_edge = self
            .faces
            .get(&initial_face.id())
            .unwrap()
            .incident_half_edge
            .unwrap();

        CcwFacesWalker {
            initial_half_edge,
            curr_half_edge: initial_half_edge,
        }
        .iter(self)
    }
}
