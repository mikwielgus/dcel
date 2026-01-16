use maplike::Get;

use crate::{Dcel, EdgeId, FaceId, HalfEdge, HalfEdgeId, Vertex, VertexId};

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
        let next_half_edge = dcel.turn_back_half_edge(self.curr_half_edge?);

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
        let next_half_edge = dcel.turn_half_edge(self.curr_half_edge?);

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
        let next_edge = dcel.turn_back_edge(self.curr_edge?);

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
        let next_edge = dcel.turn_edge(self.curr_edge?);

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
            candidate_next_edge = dcel.turn_edge(candidate_next_edge);
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
        let next_half_edge = dcel.turn_back_half_edge(self.curr_half_edge?);

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
        let next_half_edge = dcel.turn_half_edge(self.curr_half_edge?);

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
