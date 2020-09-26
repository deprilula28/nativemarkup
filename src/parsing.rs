use html5ever::{
    driver::ParseOpts,
    interface::{Attribute, ExpandedName, QualName},
    parse_document,
    tendril::{StrTendril, TendrilSink},
    tree_builder::{TreeBuilderOpts, TreeSink},
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;
use std::rc::Weak;

/// Represents a HTML abstract syntax tree
pub struct HtmlAst<'a> {
    data: Vec<Tag<'a>>,
}

/// Represents a single HTML tag
///
/// # Example
///
/// ```html
/// <tag attribute="value">
///  -- inner --
/// </tag>
/// ```
#[derive(Clone)]
pub struct Tag<'a> {
    name: &'a str,
    parent: Option<Rc<Tag<'a>>>,
    attributes: HashMap<String, HtmlAttribute>,
    inner: Vec<Tag<'a>>,
    text_contents: &'a str,
}

impl<'a> Tag<'a> {
    pub fn named(name: &'a str) -> Tag<'a> {
        Tag {
            name,
            .. Default::default()
        }
    }
}

impl<'a> Default for Tag<'a> {
    fn default() -> Tag<'a> {
        Tag {
            name: "",
            parent: None,
            attributes: HashMap::new(),
            inner: Vec::new(),
            text_contents: "",
        }
    }
}

#[derive(Clone)]
pub enum HtmlAttribute {
    String(String),
}

pub fn html_parser_opts() -> ParseOpts {
    ParseOpts {
        tree_builder: TreeBuilderOpts {
            drop_doctype: true,
            ..Default::default()
        },
        ..Default::default()
    }
}

pub fn parse_text(text: &str, opts: ParseOpts) {
    parse_document(HtmlParser::new(), opts)
    .from_utf8().read_from(text.as_bytes_mut()).unwrap();
}

pub struct HtmlParser<'a> {
    ast: HtmlAst<'a>,
}

impl<'a> HtmlParser<'a> {
    pub fn new() -> HtmlParser<'a> {
        HtmlParser {
            ast: HtmlAst { 
                data: Vec::new() 
            }
        }
    }
}

impl<'s> TreeSink for HtmlParser<'s> {
    type Handle = Tag<'s>;
    type Output = HtmlAst<'s>;

    fn finish(self) -> Self::Output {
        self.ast
    }

    fn parse_error(&mut self, msg: Cow<'static, str>) {}

    /// Get a handle to the `Document` node.
    fn get_document(&mut self) -> Self::Handle {
        self 
    }

    /// What is the name of this element?
    ///
    /// Should never be called on a non-element node;
    /// feel free to `panic!`.
    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> ExpandedName<'a> {

    }

    /// Create an element.
    ///
    /// When creating a template element (`name.ns.expanded() == expanded_name!(html "template")`),
    /// an associated document fragment called the "template contents" should
    /// also be created. Later calls to self.get_template_contents() with that
    /// given element return it.
    /// See [the template element in the whatwg spec][whatwg template].
    ///
    /// [whatwg template]: https://html.spec.whatwg.org/multipage/#the-template-element
    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        flags: ElementFlags,
    ) -> Self::Handle;

    /// Create a comment node.
    fn create_comment(&mut self, text: StrTendril) -> Self::Handle;

    /// Create a Processing Instruction node.
    fn create_pi(&mut self, target: StrTendril, data: StrTendril) -> Self::Handle;

    /// Append a node as the last child of the given node.  If this would
    /// produce adjacent sibling text nodes, it should concatenate the text
    /// instead.
    ///
    /// The child node will not already have a parent.
    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>);

    /// When the insertion point is decided by the existence of a parent node of the
    /// element, we consider both possibilities and send the element which will be used
    /// if a parent node exists, along with the element to be used if there isn't one.
    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    );

    /// Append a `DOCTYPE` element to the `Document` node.
    fn append_doctype_to_document(
        &mut self,
        name: StrTendril,
        public_id: StrTendril,
        system_id: StrTendril,
    );

    /// Mark a HTML `<script>` as "already started".
    fn mark_script_already_started(&mut self, _node: &Self::Handle) {}

    /// Indicate that a node was popped off the stack of open elements.
    fn pop(&mut self, _node: &Self::Handle) {}

    /// Get a handle to a template's template contents. The tree builder
    /// promises this will never be called with something else than
    /// a template element.
    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle;

    /// Do two handles refer to the same node?
    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool;

    /// Set the document's quirks mode.
    fn set_quirks_mode(&mut self, mode: QuirksMode);

    /// Append a node as the sibling immediately before the given node.
    ///
    /// The tree builder promises that `sibling` is not a text node.  However its
    /// old previous sibling, which would become the new node's previous sibling,
    /// could be a text node.  If the new node is also a text node, the two should
    /// be merged, as in the behavior of `append`.
    ///
    /// NB: `new_node` may have an old parent, from which it should be removed.
    fn append_before_sibling(&mut self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>);

    /// Add each attribute to the given element, if no attribute with that name
    /// already exists. The tree builder promises this will never be called
    /// with something else than an element.
    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>);

    /// Associate the given form-associatable element with the form element
    fn associate_with_form(
        &mut self,
        _target: &Self::Handle,
        _form: &Self::Handle,
        _nodes: (&Self::Handle, Option<&Self::Handle>),
    ) {
    }

    /// Detach the given node from its parent.
    fn remove_from_parent(&mut self, target: &Self::Handle);

    /// Remove all the children from node and append them to new_parent.
    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle);

    /// Returns true if the adjusted current node is an HTML integration point
    /// and the token is a start tag.
    fn is_mathml_annotation_xml_integration_point(&self, _handle: &Self::Handle) -> bool {
        false
    }

    /// Called whenever the line number changes.
    fn set_current_line(&mut self, _line_number: u64) {}

    /// Indicate that a `script` element is complete.
    fn complete_script(&mut self, _node: &Self::Handle) -> NextParserState {
        NextParserState::Continue
    }
}
