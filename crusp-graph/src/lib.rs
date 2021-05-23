#![allow(
    clippy::single_match,
    clippy::match_same_arms,
    clippy::match_ref_pats,
    clippy::clone_on_ref_ptr,
    clippy::needless_pass_by_value,
    clippy::redundant_field_names,
    clippy::redundant_pattern
)]
#![deny(clippy::wrong_pub_self_convention, clippy::used_underscore_binding,
        clippy::similar_names, clippy::pub_enum_variant_names,
        //clippy::missing_docs_in_private_items,
        clippy::non_ascii_literal, clippy::unicode_not_nfc,
        clippy::unwrap_used,
        clippy::option_map_or_none, clippy::map_unwrap_or,
        clippy::filter_map,
        clippy::shadow_unrelated, clippy::shadow_reuse, clippy::shadow_same,
        clippy::int_plus_one, clippy::string_add_assign, clippy::if_not_else,
        clippy::invalid_upcast_comparisons,
        clippy::cast_precision_loss, clippy::cast_lossless,
        clippy::cast_possible_wrap, clippy::cast_possible_truncation,
        clippy::mutex_integer, clippy::mut_mut, clippy::items_after_statements,
        clippy::print_stdout, clippy::mem_forget, clippy::maybe_infinite_iter)]

use crusp_core::{Mergeable, Nullable, Subsumed};
use priority_queue::PriorityQueue;

use std::{default::Default, marker::PhantomData};
use std::fmt::Debug;
use std::rc::Rc;

// TODO MAYBE SPLIT EVENT HANDLER AND GRAPH CONSTRAINT LIST OF VARIABLES

// TODO(vincent): variables: active failure count: almost ok
// TODO(vincent): variables: actions: almost ok
// TODO(vincent): chb read more: ?????

// TODO(vincent): variables: last change
// TODO(vincent): disable constraints
// TODO(vincent): add constraint
// TODO(vincent): Add builder then create proc macro for graph auto generation
// TODO(vincent): rmv useless pub

// Schema:
// Add event of type A
// Push event in queau of event A graph
// Add event of type B
// Push event in queau of event B graph
// when peek
// Check all events of all queues and gather output events based on them
// return the out_event with the highest priority

// create default event for event that only support one propagate function

pub trait GraphNode: std::hash::Hash + PartialEq + Eq + Ord + PartialOrd + Into<usize> + From<usize> + Copy + Debug {}

impl GraphNode for crusp_core::VariableId {}
impl GraphNode for crusp_core::ConstraintId {}
pub trait GraphEvent: Mergeable + Subsumed + Nullable + Debug {}

pub trait InputEventRegister<InNode, InEvent, Output> {
    // update pred out if already existing one
    //fn register<Pred>(&mut self, in_node: &InNode, in_event: &InEvent, out: &Output, filter) -> bool;
    //fn unregister<Pred>(&mut self, in_node: &InNode, in_event: &InEvent, out: &Output, filter: Pred)
    //    where Pred: Fn(&Output)-> bool;
}

pub trait InputEventHandler<InNode, InEvent>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    /// Notify incoming event to the handler. Do not necessarly trigger the event.
    fn notify(&mut self, node: &InNode, event: &InEvent) -> bool;
    // Tells if any non null event occurs  for the node `node` since the last call to peek_change
    //fn peek_change(&mut self, node: &InNode) -> bool;
}

pub trait InOutEventHandlerBuilder<OutNode, OutEvent, InNode, InEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    fn add_event(
        &mut self,
        out_node: &OutNode,
        out_event: &OutEvent,
        in_node: &InNode,
        in_event: &InEvent,
        cost: i64,
    );
}

pub trait OutputEventHandler<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    fn collect_and_pop_not_ignored(&mut self) -> Option<(OutNode, OutEvent)> {
        self.collect_and_pop(None)
    }
    fn collect_and_pop(&mut self, ignored: Option<OutNode>) -> Option<(OutNode, OutEvent)>;
    fn collect(&mut self, ignored: Option<OutNode>);
    fn collect_not_ignored(&mut self) {
        self.collect(None)
    }
}

pub trait OutputEventHandlerLookup<OutNode, OutEvent, Look>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    fn collect_look_and_pop_not_ignored(&mut self, look: &mut Look) -> Option<(OutNode, OutEvent)> {
        self.collect_look_and_pop(look, None)
    }
    // look for in events?
    // gurantee to collect all events since last collect
    fn collect_look_and_pop(&mut self, look: &mut Look, ignored: Option<OutNode>) -> Option<(OutNode, OutEvent)>;
    fn collect_look(&mut self, look: &mut Look, ignored: Option<OutNode>);
    fn collect_look_not_ignored(&mut self, look: &mut Look) {
        self.collect_look(look, None)
    }
}

pub trait VisitMut<T> {
    fn visit_mut(&mut self, t: &T);
}

pub trait LookEvent<Node: GraphNode, Event: GraphEvent> {
    fn look_event(&mut self, node: &Node, event: &Event);
}

pub trait GraphBuilder<OutNode, InNode>
where
    OutNode: GraphNode,
    InNode: GraphNode,
{
    fn add_node(&mut self, out_node: &OutNode, in_node: &InNode);
}

pub trait VisitOutputsNode<OutNode, InNode>
where
    OutNode: GraphNode,
    InNode: GraphNode,
{
    fn visit_in_nodes<Visitor>(&self, out_node: &OutNode, visitor: &mut Visitor)
    where
        Visitor: VisitMut<InNode>;
}

pub trait VisitAllOutputsNode<OutNode, Visitor>
where
    OutNode: GraphNode,
{
    fn visit_all_in_nodes(&self, out_node: &OutNode, visitor: &mut Visitor);
}

struct EventLink<InEvent: GraphEvent, Output> {
    in_event: InEvent,
    out: Output,
}

pub struct LazyInputEventGraphBuilder<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    in_events: Vec<Vec<EventLink<InEvent, Output>>>,
    _in_node: PhantomData<InNode>,
}

pub struct LazyInputEventGraph<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    in_events: Vec<Vec<EventLink<InEvent, Output>>>,
    _in_node: PhantomData<InNode>,
}

impl<InNode, InEvent, Output> Default for LazyInputEventGraphBuilder<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<InNode, InEvent, Output> LazyInputEventGraphBuilder<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    pub fn new() -> Self {
        LazyInputEventGraphBuilder {
            in_events: Vec::new(),
            _in_node: PhantomData,
        }
    }

    #[allow(clippy::shadow_reuse)]
    pub fn add_event(&mut self, node: InNode, event: InEvent, out: Output) {
        let idx: usize = node.into();
        if idx >= self.in_events.len() {
            self.in_events.resize_with(idx + 1, Vec::new)
        }
        self.in_events[idx].push(EventLink {
            in_event: event,
            out: out,
        });
    }

    pub fn finalize(self) -> LazyInputEventGraph<InNode, InEvent, Output> {
        LazyInputEventGraph {
            in_events: self.in_events,
            _in_node: PhantomData,
        }
    }
}

pub struct LazyInputEventHandler<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    graph: Rc<LazyInputEventGraph<InNode, InEvent, Output>>,
    events: Vec<(InNode, InEvent)>,
    //    changes: HahshMap<InNode, bool>,
}

impl<InNode, InEvent, Output> InputEventHandler<InNode, InEvent>
    for LazyInputEventHandler<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    fn notify(&mut self, node: &InNode, event: &InEvent) -> bool {
        if event.is_null() {
            return false;
        }
        match self.events.last_mut() {
            Some(&mut (l_node, ref mut l_evt)) if l_node.into() == (*node).into() => {
                *l_evt = l_evt.merge(*event);
            }
            _ => {
                self.events.push((*node, *event));
            }
        }
        true
    }

    /*fn peek_change(&mut self, node: &InNode) -> bool {
        unimplemented!()
        match self.changes.get_mut(node) {
            Some(ref mut ch) => {
                let ret = *ch;
                *ch = true;
                ret
            },
            None => false
        }
    }*/
}

impl<InNode, InEvent, Output> LazyInputEventHandler<InNode, InEvent, Output>
where
    InNode: GraphNode,
    InEvent: GraphEvent,
{
    pub fn builder() -> LazyInputEventGraphBuilder<InNode, InEvent, Output> {
        LazyInputEventGraphBuilder::new()
    }

    pub fn new(graph: LazyInputEventGraph<InNode, InEvent, Output>) -> Self {
        LazyInputEventHandler {
            graph: Rc::new(graph),
            events: Vec::new(),
        }
    }

    pub fn trigger_events<F>(&mut self, mut process: F)
    where
        F: FnMut(&Output),
    {
        if self.events.is_empty() {
            return;
        }
        self.events.sort_unstable_by(|lhs, rhs| {
            lhs.0.into().partial_cmp(&rhs.0.into()).expect("Comparable input nodes")
        });
        // consumes events here
        let events: Vec<_> = self.events.drain(..).collect();
        let mut events = events.into_iter();
        let (mut curr_node, mut curr_event) = events.next().expect("At least one element");
        for (in_node, in_event) in events {
            if curr_node.into() == in_node.into() {
                curr_event = curr_event.merge(in_event);
            } else {
                self.process_in_event(&curr_node, &curr_event, &mut process);
                curr_node = in_node;
                curr_event = in_event;
            }
        }
        self.process_in_event(&curr_node, &curr_event, &mut process);
    }

    pub fn trigger_look_events<F, Look>(&mut self, mut process: F, look_in: &mut Look)
    where
        F: FnMut(&Output),
        Look: LookEvent<InNode, InEvent>,
    {
        if self.events.is_empty() {
            return;
        }
        self.events.sort_unstable_by(|lhs, rhs| {
            lhs.0.into().partial_cmp(&rhs.0.into()).expect("Comparable input nodes")
        });
        // consumes events here
        let events: Vec<_> = self.events.drain(..).collect();
        let mut events = events.into_iter();
        let (mut curr_node, mut curr_event) = events.next().expect("At least one element");
        for (in_node, in_event) in events {
            if curr_node.into() == in_node.into() {
                curr_event = curr_event.merge(in_event);
            } else {
                look_in.look_event(&curr_node, &curr_event);
                self.process_in_event(&curr_node, &curr_event, &mut process);
                curr_node = in_node;
                curr_event = in_event;
            }
        }
        look_in.look_event(&curr_node, &curr_event);
        self.process_in_event(&curr_node, &curr_event, &mut process);
    }

    #[allow(clippy::filter_map)]
    pub fn process_in_event<F>(&self, in_node: &InNode, in_event: &InEvent, process: &mut F)
    where
        F: FnMut(&Output),
    {
        // /self.changes.entry(in_node).or_insert(true);
        // TODO: rmv bound checks
        let in_idx: usize = (*in_node).into();
        self.graph.in_events[in_idx]
            .iter()
            .filter(|&out_event| in_event.is_subsumed_under(&out_event.in_event.merge(*in_event)))
            .map(|link| &link.out)
            .for_each(|out| process(out));
    }
}

pub struct AdjacentListGraphBuilder<SrcNode, DstNode>
where
    SrcNode: GraphNode,
    DstNode: GraphNode,
{
    len: usize,
    ins: Vec<Vec<DstNode>>,
    _src_node: PhantomData<SrcNode>
}

impl<SrcNode, DstNode> AdjacentListGraphBuilder<SrcNode, DstNode>
where
    SrcNode: GraphNode,
    DstNode: GraphNode,
{
    fn new() -> Self {
        AdjacentListGraphBuilder {
            len: 0usize,
            ins: Vec::new(),
            _src_node: PhantomData,
        }
    }

    pub fn finalize(mut self) -> AdjacentListGraph<SrcNode, DstNode> {
        for ins in self.ins.iter_mut() {
            ins[..].sort();
            ins.dedup();
        }
        AdjacentListGraph {
            ins: self.ins,
            _src_node: PhantomData,
        }
    }
}

impl<OutNode, InNode> GraphBuilder<OutNode, InNode> for AdjacentListGraphBuilder<OutNode, InNode>
where
    OutNode: GraphNode,
    InNode: GraphNode,
{
    fn add_node(&mut self, out_node: &OutNode, in_node: &InNode) {
        let idx: usize = (*out_node).into();
        if idx >= self.len {
            self.ins.resize_with(idx + 1, Vec::new);
            self.len = idx + 1;
        }
        self.ins[idx].push(*in_node);
    }
}

pub struct AdjacentListGraph<SrcNode, DstNode>
where
    SrcNode: GraphNode,
    DstNode: GraphNode,
{
    ins: Vec<Vec<DstNode>>,
    _src_node: PhantomData<SrcNode>
}

impl<SrcNode, DstNode> AdjacentListGraph<SrcNode, DstNode>
where
    SrcNode: GraphNode,
    DstNode: GraphNode,
{
    pub fn builder() -> AdjacentListGraphBuilder<SrcNode, DstNode> {
        AdjacentListGraphBuilder::new()
    }
}

impl<SrcNode, DstNode> VisitOutputsNode<SrcNode, DstNode> for AdjacentListGraph<SrcNode, DstNode>
where
    SrcNode: GraphNode,
    DstNode: GraphNode,
{
    /*fn visit_all_in_nodes<Visitor>(&self, visitor: &mut Visitor)
        where Visitor: VisitMut<DstNode>
    {
        if let Some(idx) = self.map_to_idx.get(out_node) {
            let ins = self.ins[*idx].iter();
            for v in ins {
                visitor.visit_mut(&v);
            }
        }
    }*/

    fn visit_in_nodes<Visitor>(&self, out_node: &SrcNode, visitor: &mut Visitor)
    where
        Visitor: VisitMut<DstNode>,
    {
        if let Some(ins) = self.ins.get((*out_node).into()) {
            for v in ins.iter() {
                visitor.visit_mut(&v);
            }
        }
    }
}

pub struct OutCostEventLink<OutNode: GraphNode, OutEvent: GraphEvent> {
    idx: OutNode,
    event: OutEvent,
    cost: i64,
}

impl<OutNode: GraphNode, OutEvent: GraphEvent> OutCostEventLink<OutNode, OutEvent> {
    pub fn new(idx: OutNode, event: OutEvent, cost: i64) -> Self {
        OutCostEventLink {
            idx: idx,
            event: event,
            cost: cost,
        }
    }
}

pub struct HandlerOutputBuilder<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    last_out: OutNode,
    _event: PhantomData<OutEvent>,
    _out_node: PhantomData<OutNode>,
}

impl<OutNode, OutEvent> Default for HandlerOutputBuilder<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<OutNode, OutEvent> HandlerOutputBuilder<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    pub fn new() -> Self {
        HandlerOutputBuilder {
            last_out: 0usize.into(),
            _event: PhantomData,
            _out_node: PhantomData,
        }
    }

    pub fn add_node(&mut self, node: OutNode) {
        self.last_out = self.last_out.max(node);
    }

    pub fn finalize(self) -> HandlerOutput<OutNode, OutEvent> {
        let len = self.last_out.into() + 1;
        HandlerOutput {
            mode: vec![OutEvent::null(); len],
            queue: PriorityQueue::new(),
        }
    }
}

pub struct HandlerOutput<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    mode: Vec<OutEvent>,
    queue: PriorityQueue<OutNode, i64>,
}

impl<OutNode, OutEvent> HandlerOutput<OutNode, OutEvent>
where
    OutNode: GraphNode,
    OutEvent: GraphEvent,
{
    pub fn builder() -> HandlerOutputBuilder<OutNode, OutEvent> {
        HandlerOutputBuilder::new()
    }

    pub fn collect_out_event(&mut self, out: &OutCostEventLink<OutNode, OutEvent>, ignored_out: Option<OutNode>) {
        unsafe {
            let out_node = out.idx;
            let ignored = match ignored_out {
                Some(ignored_out) if ignored_out == out_node => {
                    true
                },
                _ => false,
            };
            if !ignored {
                self.queue.push(out_node, out.cost);
                let curr_state = self.mode.get_unchecked_mut(out_node.into());
                *curr_state = curr_state.merge(out.event);
            }
        }
    }

    #[inline]
    pub fn pop(&mut self) -> Option<(OutNode, OutEvent)> {
        let (out_idx, _cost) = self.queue.pop()?;
        let event = self.mode[out_idx.into()].nullify();
        Some((out_idx, event))
    }
}
