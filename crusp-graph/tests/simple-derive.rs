//extern crate constraint_derive;

use crusp_graph::*;
use crusp_graph_derive::crusp_lazy_graph;
use std::fmt::Debug;

#[derive(
    PartialEq, Eq, std::hash::Hash, std::cmp::PartialOrd, std::cmp::Ord, Clone, Copy, Debug,
)]
pub struct OutNode {
    idx: usize,
}
impl GraphNode for OutNode {}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct OutEvent {
    val: i32,
}
impl Nullable for OutEvent {
    fn is_null(&self) -> bool {
        self.val == 0
    }
    fn null() -> Self {
        OutEvent { val: 0 }
    }
    fn nullify(&mut self) -> Self {
        let prev = *self;
        *self = Self::null();
        prev
    }
}
impl Mergeable for OutEvent {
    fn merge(&self, rhs: Self) -> Self {
        let ret = self.val | rhs.val;
        OutEvent { val: ret }
    }
}
impl Subsumed for OutEvent {
    fn is_subsumed_under(&self, _rhs: &Self) -> bool {
        true
    }
}
impl GraphEvent for OutEvent {}

#[derive(
    PartialEq, Eq, std::hash::Hash, std::cmp::PartialOrd, std::cmp::Ord, Clone, Copy, Debug,
)]
pub struct InNode1 {
    idx: usize,
}
impl GraphNode for InNode1 {}

#[derive(Copy, Clone, Debug)]
pub struct InEvent1 {
    val: i32,
}
impl Nullable for InEvent1 {
    fn is_null(&self) -> bool {
        self.val == 0
    }
    fn null() -> Self {
        InEvent1 { val: 0 }
    }
    fn nullify(&mut self) -> Self {
        let prev = *self;
        *self = Self::null();
        prev
    }
}
impl Mergeable for InEvent1 {
    fn merge(&self, rhs: Self) -> Self {
        let ret = self.val | rhs.val;
        InEvent1 { val: ret }
    }
}
impl Subsumed for InEvent1 {
    fn is_subsumed_under(&self, _rhs: &Self) -> bool {
        true
    }
}
impl GraphEvent for InEvent1 {}

#[derive(
    PartialEq, Eq, std::hash::Hash, std::cmp::PartialOrd, std::cmp::Ord, Clone, Copy, Debug,
)]
pub struct InNode2 {
    idx: usize,
}
impl GraphNode for InNode2 {}

#[derive(Copy, Clone, Debug)]
pub struct InEvent2 {
    val: i32,
}
impl Nullable for InEvent2 {
    fn is_null(&self) -> bool {
        self.val == 0
    }
    fn null() -> Self {
        InEvent2 { val: 0 }
    }
    fn nullify(&mut self) -> Self {
        let prev = *self;
        *self = Self::null();
        prev
    }
}
impl Mergeable for InEvent2 {
    fn merge(&self, rhs: Self) -> Self {
        let ret = self.val | rhs.val;
        InEvent2 { val: ret }
    }
}
impl Subsumed for InEvent2 {
    fn is_subsumed_under(&self, _rhs: &Self) -> bool {
        true
    }
}
impl GraphEvent for InEvent2 {}

#[crusp_lazy_graph]
struct GraphName {
    #[output]
    out: (OutNode, OutEvent),
    #[input]
    in1: (InNode1, InEvent1),
    #[input]
    in2: (InNode2, InEvent2),
}

#[derive(Debug)]
pub struct MyVisitor {
    pub n1: usize,
    pub n2: usize,
}

impl MyVisitor {
    fn new() -> MyVisitor {
        MyVisitor { n1: 0, n2: 0 }
    }
}

impl VisitMut<InNode1> for MyVisitor {
    fn visit_mut(&mut self, _t: &InNode1) {
        self.n1 += 1;
    }
}

impl VisitMut<InNode2> for MyVisitor {
    fn visit_mut(&mut self, _t: &InNode2) {
        self.n2 += 1;
    }
}

pub fn main() {
    let oe0 = OutEvent { val: 0 };
    let oe1 = OutEvent { val: 1 };
    let oe2 = OutEvent { val: 2 };
    let on0 = OutNode { idx: 0 };
    let on1 = OutNode { idx: 1 };
    let on2 = OutNode { idx: 2 };
    let in1 = InNode1 { idx: 0 };
    let ie1 = InEvent1 { val: 1 };
    let in12 = InNode1 { idx: 1 };
    let ie12 = InEvent1 { val: 2 };
    let in2 = InNode2 { idx: 0 };
    let ie2 = InEvent2 { val: 1 };
    let mut graph = GraphName::builder();
    graph.add_event(&on0, &oe0, &in1, &ie1, 0i64);
    graph.add_event(&on1, &oe1, &in12, &ie12, 2i64);
    graph.add_event(&on2, &oe2, &in2, &ie2, 1i64);
    let mut graph = graph.finalize();
    let mut visitor = MyVisitor::new();
    assert_eq!(visitor.n1, 0);
    assert_eq!(visitor.n2, 0);
    graph.visit_all_in_nodes(&on0, &mut visitor);
    assert_eq!(visitor.n1, 1);
    assert_eq!(visitor.n2, 0);
    graph.visit_all_in_nodes(&on1, &mut visitor);
    assert_eq!(visitor.n1, 2);
    assert_eq!(visitor.n2, 0);
    graph.visit_all_in_nodes(&on2, &mut visitor);
    assert_eq!(visitor.n1, 2);
    assert_eq!(visitor.n2, 1);
    graph.notify(&in1, &ie1);
    let event = graph.collect_and_pop();
    assert_eq!(event, Some((on0, oe0)));
    let event = graph.collect_and_pop();
    assert_eq!(event, None);
    graph.notify(&in1, &ie1);
    graph.notify(&in12, &ie12);
    graph.notify(&in2, &ie2);
    let event = graph.collect_and_pop();
    assert_eq!(event, Some((on1, oe1)));
    let event = graph.collect_and_pop();
    assert_eq!(event, Some((on2, oe2)));
    graph.notify(&in12, &ie12);
    let event = graph.collect_and_pop();
    assert_eq!(event, Some((on1, oe1)));
    let event = graph.collect_and_pop();
    assert_eq!(event, Some((on0, oe0)));
    let event = graph.collect_and_pop();
    assert_eq!(event, None);
    let event = graph.collect_and_pop();
    assert_eq!(event, None);
}
