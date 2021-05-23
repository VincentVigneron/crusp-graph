extern crate proc_macro;
extern crate quote;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

macro_rules! span {
    () => {{
        Span::call_site()
    }};
}

#[derive(Debug, Clone)]
struct GraphElt {
    ident: syn::Ident,
    node: syn::Ident,
    event: syn::Ident,
}

#[derive(Debug, Clone)]
struct GraphStructure {
    ident: syn::Ident,
    out: GraphElt,
    ins: Vec<GraphElt>,
}

fn read_typepath_ident(path: &syn::TypePath) -> syn::Ident {
    path.path.segments.first().expect("One type").ident.clone()
}

fn pair_to_idents(pair: &syn::TypeTuple) -> (syn::Ident, syn::Ident) {
    let pair: Vec<_> = pair
        .elems
        .iter()
        .map(|tp| {
            if let syn::Type::Path(ref tp) = tp {
                tp
            } else {
                unimplemented!()
            }
        })
        .collect();
    if pair.len() != 2 {
        panic!()
    }
    (read_typepath_ident(pair[0]), read_typepath_ident(pair[1]))
}

fn field_to_graph_elt(field: &syn::Field) -> GraphElt {
    let ident = field.ident.clone().expect("Identifier expected");
    let tuple = if let syn::Type::Tuple(ref tuple) = field.ty {
        tuple
    } else {
        unimplemented!()
    };
    let (node, event) = pair_to_idents(tuple);
    GraphElt { ident, node, event }
}

// TODO(vincent): check if item is a DataStruct
#[proc_macro_attribute]
pub fn crusp_lazy_graph(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(item as DeriveInput);
    //eprintln!("{:#?}", ast);
    let data = if let syn::Data::Struct(ref data) = ast.data {
        data
    } else {
        unimplemented!()
    };
    // get visibility
    let ident = ast.ident.clone();
    let ident_builder = format!("{}Builder", ast.ident);
    let graph_ident_builder = syn::Ident::new(&ident_builder, span!());
    let fields = if let syn::Fields::Named(ref fields) = data.fields {
        &fields.named
    } else {
        unimplemented!()
    };
    let (out, ins): (Vec<_>, Vec<_>) = fields
        .iter()
        .partition(|field| field.attrs[0].path.segments[0].ident == "output");
    if out.len() != 1 {
        panic!()
    }
    if ins.is_empty() {
        panic!()
    }
    let out = out
        .into_iter()
        .map(|field| field_to_graph_elt(field))
        .next()
        .expect("Exactly one element");
    let ins: Vec<_> = ins
        .into_iter()
        .map(|field| field_to_graph_elt(field))
        .collect();
    let graph = GraphStructure { ident, out, ins };
    //eprintln!("{:#?}", graph);

    let graph_ident = graph.ident;
    let (out_ident, out_node, out_event) = (graph.out.ident, graph.out.node, graph.out.event);

    let out_field = quote!(
        #out_ident: ::crusp_graph::HandlerOutput<#out_node, #out_event>
    );
    let out_builder_field = quote!(
        #out_ident: ::crusp_graph::HandlerOutputBuilder<#out_node, #out_event>
    );
    let in_builder_fields: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (ident, node, event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            quote!(
                #ident: ::crusp_graph::LazyInputEventGraphBuilder<
                    #node,
                    #event,
                    ::crusp_graph::OutCostEventLink<#out_event>
                >
            )
        })
        .collect();
    let in_rev_builder_fields: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (ident, in_node, _event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            let ident_name = format!("__crusp__rev_{}", ident);
            let ident = syn::Ident::new(&ident_name, span!());
            let out_node = out_node.clone();
            quote!(
                #ident: ::crusp_graph::AdjacentListGraphBuilder<
                    #out_node,
                    #in_node,
                >
            )
        })
        .collect();
    let in_fields: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (ident, node, event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            quote!(
                #ident: ::crusp_graph::LazyInputEventHandler<
                    #node,
                    #event,
                    ::crusp_graph::OutCostEventLink<#out_event>
                >
            )
        })
        .collect();
    let in_rev_fields: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (ident, in_node, _event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            let ident_name = format!("__crusp__rev_{}", ident);
            let ident = syn::Ident::new(&ident_name, span!());
            let out_node = out_node.clone();
            quote!(
                #ident: ::std::rc::Rc<::crusp_graph::AdjacentListGraph<
                    #out_node,
                    #in_node,
                >>
            )
        })
        .collect();
    let in_events_handler: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (ident, node, event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            let in_node = node;
            let in_event = event;
            quote!(
                impl ::crusp_graph::InputEventHandler<#in_node, #in_event> for #graph_ident
                {
                    #[allow(clippy::inline_always)]
                    #[inline(always)]
                     fn notify(&mut self, in_node: &#in_node, in_event: &#in_event) -> bool {
                         if self.#ident.notify(in_node, in_event) {
                            true
                         } else {
                             false
                         }
                    }
                }
            )
        })
        .collect();
    let inout_events_handler: Vec<_> = graph.ins.iter()
        .map(|field| {
            let graph_ident_builder = graph_ident_builder.clone();
            let (in_ident, node, event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            let rev_ident = format!("__crusp__rev_{}", in_ident);
            let rev_ident = syn::Ident::new(&rev_ident, span!());
            let out_node = out_node.clone();
            let out_event = out_event.clone();
            let out_ident = out_ident.clone();
            let in_node = node;
            let in_event = event;
            quote!(
                impl ::crusp_graph::InOutEventHandlerBuilder<#out_node, #out_event, #in_node, #in_event>
                    for #graph_ident_builder
                {
                    fn add_event(&mut self, out_node: &#out_node, out_event: &#out_event, in_node: &#in_node, in_event: &#in_event, cost: i64) {
                        let idx = self.#out_ident.add_node(*out_node);
                        let out = <::crusp_graph::OutCostEventLink<#out_event>>::new(
                            idx,
                            *out_event,
                            cost
                        );
                        self.#in_ident.add_event(*in_node, *in_event, out);
                        self.#rev_ident.add_node(out_node, in_node);
                    }
                }
            )
        })
        .collect();
    let graph_builder_impl = {
        let graph_ident_builder = graph_ident_builder.clone();
        let graph_ident = graph_ident.clone();
        let out_ident = out_ident.clone();
        let out_node = out_node.clone();
        let out_event = out_event.clone();
        let in_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                quote!(#field)
            })
            .collect();
        let in_rev_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                let rev_field = format!("__crusp__rev_{}", field);
                let rev_field = syn::Ident::new(&rev_field, span!());
                quote!(#rev_field)
            })
            .collect();
        let in_idents2 = in_idents.clone();
        let in_idents3 = in_idents.clone();
        let out_events: Vec<_> = std::iter::repeat(&out_event)
            .clone()
            .take(in_rev_idents.len())
            .collect();
        let out_nodes: Vec<_> = std::iter::repeat(&out_node)
            .clone()
            .take(in_idents.len())
            .collect();
        let in_nodes: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.node.clone();
                quote!(#field)
            })
            .collect();
        let in_events: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.event.clone();
                quote!(#field)
            })
            .collect();
        let in_nodes2 = in_nodes.clone();
        let in_events2 = in_events.clone();
        let out_events2 = out_events.clone();
        let in_rev_idents2 = in_rev_idents.clone();
        let in_rev_idents3 = in_rev_idents.clone();
        let in_rev_nodes = in_nodes.clone();

        quote!(
            impl #graph_ident_builder
            {
                pub fn new() -> Self {
                    #graph_ident_builder {
                        #(#in_idents: <::crusp_graph::LazyInputEventHandler<#in_nodes, #in_events, ::crusp_graph::OutCostEventLink<#out_events>>>::builder()),*,
                        #(#in_rev_idents: <::crusp_graph::AdjacentListGraph<#out_nodes,#in_rev_nodes>>::builder()),*,
                        #out_ident: <::crusp_graph::HandlerOutput<#out_node, #out_event>>::builder(),
                    }
                }

                pub fn finalize(self) -> #graph_ident {
                    #graph_ident {
                        #(#in_idents2: <::crusp_graph::LazyInputEventHandler<#in_nodes2, #in_events2, ::crusp_graph::OutCostEventLink<#out_events2>>>::new(self.#in_idents3.finalize())),*,
                        #(#in_rev_idents2: ::std::rc::Rc::new(self.#in_rev_idents3.finalize())),*,
                        #out_ident: self.#out_ident.finalize(),
                    }
                }
            }
        )
    };
    let graph_impl = {
        let graph_ident_builder = graph_ident_builder.clone();
        let graph_ident = graph_ident.clone();
        let out_ident = out_ident.clone();
        let out_node = out_node.clone();
        let out_event = out_event.clone();
        let in_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                quote!(#field)
            })
            .collect();
        let out_events: Vec<_> = std::iter::repeat(&out_event)
            .clone()
            .take(in_idents.len())
            .collect();
        let in_nodes: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.node.clone();
                quote!(#field)
            })
            .collect();
        let in_events: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.event.clone();
                quote!(#field)
            })
            .collect();

        quote!(
            impl  #graph_ident
            {
                pub fn builder() -> #graph_ident_builder {
                    <#graph_ident_builder>::new()
                }

                #[allow(clippy::type_complexity)]
                #[inline]
                pub fn split_in_out(&mut self) -> (
                        &mut ::crusp_graph::HandlerOutput<#out_node, #out_event>,
                        #(&mut ::crusp_graph::LazyInputEventHandler<
                            #in_nodes, #in_events,
                            ::crusp_graph::OutCostEventLink<#out_events>>
                        ),*
                    )
                {
                    (
                        unsafe{ &mut *((&mut self.#out_ident) as *mut _)},
                        #(unsafe{ &mut *((&mut self.#in_idents) as *mut _)}),*
                    )
                }
            }
        )
    };
    let impl_pop = {
        let out_ident = out_ident.clone();
        let out_node = out_node.clone();
        let out_event = out_event.clone();
        let in_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                quote!(#field)
            })
            .collect();
        let in_idents2: Vec<_> = in_idents.clone();

        quote!(
            impl ::crusp_graph::OutputEventHandler<#out_node, #out_event> for #graph_ident
            {
                fn collect(&mut self, ignored: Option<OutNode>) {
                    let (__crusp__outs, #(#in_idents),*) = self.split_in_out();
                    #(#in_idents2.trigger_events(|__crusp__out| __crusp__outs.collect_out_event(__crusp__out, ignored)));*;
                }
                fn collect_and_pop(&mut self, ignored: Option<OutNode>) -> Option<(#out_node, #out_event)> {
                    self.collect(ignored);
                    self.#out_ident.pop()
                }
        })
    };
    let impl_pop_look = {
        let out_ident = out_ident;
        let out_node = out_node.clone();
        let out_event = out_event;
        let in_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                quote!(#field)
            })
            .collect();
        let in_nodes: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.node.clone();
                quote!(#field)
            })
            .collect();
        let in_events: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.event.clone();
                quote!(#field)
            })
            .collect();
        let in_idents2: Vec<_> = in_idents.clone();

        quote!(
            impl <Look> ::crusp_graph::OutputEventHandlerLookup<#out_node, #out_event, Look> for #graph_ident
               where
               #(Look: LookEvent<#in_nodes, #in_events>),*,
            {
                fn collect_look(&mut self, look: &mut Look, ignored: Option<OutNode>) {
                    let (__crusp__outs, #(#in_idents),*) = self.split_in_out();
                    #(#in_idents2.trigger_look_events(|__crusp__out| __crusp__outs.collect_out_event(__crusp__out, ignored), look));*;
                }
                fn collect_look_and_pop(&mut self, look: &mut Look, ignored: Option<OutNode>) -> Option<(#out_node, #out_event)> {
                    self.collect_look(look, ignored);
                    self.#out_ident.pop()
                }
        })
    };
    let impl_visit_all = {
        let graph_ident = graph_ident.clone();
        let out_node = out_node.clone();
        let in_nodes: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.node.clone();
                quote!(#field)
            })
            .collect();
        let in_rev_idents: Vec<_> = graph
            .ins
            .iter()
            .map(|field| {
                let field = field.ident.clone();
                let rev_field = format!("__crusp__rev_{}", field);
                let rev_field = syn::Ident::new(&rev_field, span!());
                quote!(#rev_field)
            })
            .collect();

        quote!(
            impl <Visitor> ::crusp_graph::VisitAllOutputsNode<#out_node, Visitor> for #graph_ident
               where
               #(Visitor: VisitMut<#in_nodes>),*,
            {
                fn visit_all_in_nodes(&self, out_node: &#out_node, visitor: &mut Visitor)
                {
                    #(self.#in_rev_idents.visit_in_nodes(out_node, visitor));*;
                }
            }
        )
    };
    let impl_visitors: Vec<_> = graph
        .ins
        .iter()
        .map(|field| {
            let (in_ident, node, _event) =
                (field.ident.clone(), field.node.clone(), field.event.clone());
            let rev_ident = format!("__crusp__rev_{}", in_ident);
            let rev_ident = syn::Ident::new(&rev_ident, span!());
            let out_node = out_node.clone();
            let in_node = node;
            quote!(
                impl ::crusp_graph::VisitOutputsNode<#out_node, #in_node>
                    for #graph_ident
                {
                    fn visit_in_nodes<Visitor>(&self, out_node: &#out_node, visitor: &mut Visitor)
                        where Visitor: VisitMut<#in_node>
                    {
                        self.#rev_ident.visit_in_nodes(out_node, visitor);
                    }
                }
            )
        })
        .collect();
    let expanded = quote!(
        struct #graph_ident_builder
        {
            #out_builder_field,
            #(#in_builder_fields),*,
            #(#in_rev_builder_fields),*,
        }

        struct #graph_ident
        {
            #out_field,
            #(#in_fields),*,
            #(#in_rev_fields),*
        }

        #(#impl_visitors)*

       #impl_visit_all

       #(#in_events_handler)*

       #(#inout_events_handler)*

       #graph_builder_impl

       #graph_impl

       #impl_pop

       #impl_pop_look
    );
    expanded.into()
}
