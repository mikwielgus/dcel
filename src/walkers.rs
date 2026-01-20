// SPDX-FileCopyrightText: 2026 dcel contributors
//
// SPDX-License-Identifier: MIT OR Apache-2.0

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
    CirculateVertexesWithExcludesWalker {
        circulator: CirculateHalfEdgesWithExcludesWalker,
    },
    CirculateVertexesWithExcludesIter
);

impl CirculateVertexesWithExcludesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<VertexId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.origin(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateVertexesWithExcludesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = VertexId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    CirculateVertexesWithExcludesReverseWalker {
        circulator: CirculateHalfEdgesWithExcludesReverseWalker,
    },
    CirculateVertexesWithExcludesReverseIter
);

impl CirculateVertexesWithExcludesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<VertexId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.origin(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateVertexesWithExcludesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = VertexId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    HalfSpokesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    HalfSpokesIter
);

impl HalfSpokesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for HalfSpokesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    HalfSpokesReverseWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    HalfSpokesReverseIter
);

impl HalfSpokesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for HalfSpokesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    SpokesWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
    },
    SpokesIter
);

impl SpokesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for SpokesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    SpokesReverseWalker {
        initial_edge: EdgeId,
        curr_edge: Option<EdgeId>,
    },
    SpokesReverseIter
);

impl SpokesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for SpokesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    InterspokesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    InterspokesIter
);

impl InterspokesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for InterspokesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = FaceId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next(self.dcel)
            .map(|half_edge| self.dcel.face_in_front(half_edge))
    }
}

create_walker_and_iter!(
    InterspokesReverseWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    InterspokesReverseIter
);

impl InterspokesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for InterspokesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = FaceId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker
            .next(self.dcel)
            .map(|half_edge| self.dcel.face_in_front(half_edge))
    }
}

create_walker_and_iter!(
    CirculateHalfEdgesWithExcludesWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
        excluded_half_edges: Vec<HalfEdgeId>,
    },
    CirculateHalfEdgesWithExcludesIter
);

impl CirculateHalfEdgesWithExcludesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let mut candidate_next_half_edge = dcel.next_half_edge(self.curr_half_edge?);

        while self.excluded_half_edges.contains(&candidate_next_half_edge) {
            candidate_next_half_edge = dcel.turn_half_edge(candidate_next_half_edge);
        }

        let next_half_edge = candidate_next_half_edge;

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateHalfEdgesWithExcludesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    CirculateHalfEdgesWithExcludesReverseWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
        excluded_half_edges: Vec<HalfEdgeId>,
    },
    CirculateHalfEdgesWithExcludesReverseIter
);

impl CirculateHalfEdgesWithExcludesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let mut candidate_next_half_edge = dcel.prev_half_edge(self.curr_half_edge?);

        while self.excluded_half_edges.contains(&candidate_next_half_edge) {
            candidate_next_half_edge =
                dcel.twin(dcel.turn_back_half_edge(dcel.twin(candidate_next_half_edge)));
        }

        let next_half_edge = candidate_next_half_edge;

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateHalfEdgesWithExcludesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    CirculateEdgesWithExcludesWalker {
        circulator: CirculateHalfEdgesWithExcludesWalker,
    },
    CirculateEdgesWithExcludesIter
);

impl CirculateEdgesWithExcludesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.full_edge(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateEdgesWithExcludesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    CirculateEdgesWithExcludesReverseWalker {
        circulator: CirculateHalfEdgesWithExcludesReverseWalker,
    },
    CirculateEdgesWithExcludesReverseIter
);

impl CirculateEdgesWithExcludesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.full_edge(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for CirculateEdgesWithExcludesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    FaceVertexesWalker {
        circulator: FaceHalfEdgesWalker,
    },
    FaceVertexesIter
);

impl FaceVertexesWalker {
    #[inline]
    pub fn next<
        VW,
        HEW,
        FW,
        VC: Get<usize, Value = Vertex<VW>>,
        HEC: Get<usize, Value = HalfEdge<HEW>>,
        FC,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<VertexId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.origin(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Iterator for FaceVertexesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = VertexId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    FaceVertexesReverseWalker {
        circulator: FaceHalfEdgesReverseWalker,
    },
    FaceVertexesReverseIter
);

impl FaceVertexesReverseWalker {
    #[inline]
    pub fn next<
        VW,
        HEW,
        FW,
        VC: Get<usize, Value = Vertex<VW>>,
        HEC: Get<usize, Value = HalfEdge<HEW>>,
        FC,
    >(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<VertexId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.origin(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC: Get<usize, Value = Vertex<VW>>, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>
    Iterator for FaceVertexesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
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
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
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

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for FaceHalfEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    FaceHalfEdgesReverseWalker {
        initial_half_edge: HalfEdgeId,
        curr_half_edge: Option<HalfEdgeId>,
    },
    FaceHalfEdgesReverseIter
);

impl FaceHalfEdgesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<HalfEdgeId> {
        let next_half_edge = dcel.prev_half_edge(self.curr_half_edge?);

        std::mem::replace(
            &mut self.curr_half_edge,
            (next_half_edge != self.initial_half_edge).then_some(next_half_edge),
        )
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for FaceHalfEdgesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = HalfEdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    FaceEdgesWalker {
        circulator: FaceHalfEdgesWalker,
    },
    FaceEdgesIter
);

impl FaceEdgesWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.full_edge(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for FaceEdgesIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}

create_walker_and_iter!(
    FaceEdgesReverseWalker {
        circulator: FaceHalfEdgesReverseWalker,
    },
    FaceEdgesReverseIter
);

impl FaceEdgesReverseWalker {
    #[inline]
    pub fn next<VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC>(
        &mut self,
        dcel: &Dcel<VW, HEW, FW, VC, HEC, FC>,
    ) -> Option<EdgeId> {
        self.circulator
            .next(dcel)
            .map(|half_edge| dcel.full_edge(half_edge))
    }
}

impl<'a, VW, HEW, FW, VC, HEC: Get<usize, Value = HalfEdge<HEW>>, FC> Iterator
    for FaceEdgesReverseIter<'a, VW, HEW, FW, VC, HEC, FC>
{
    type Item = EdgeId;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.walker.next(self.dcel)
    }
}
