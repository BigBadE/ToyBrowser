pub struct Node {
    children: Vec<Element>,
    node_type: NodeType,
}

impl Node {
    pub fn add_child(&mut self, child: Element) {
        self.children.push(child);
    }
}

pub struct Element {
    tag: String,
    attributes: Vec<Attribute>,
}

pub struct DocumentType {
    internel_subset: String,
    name: String,
    notations: Vec<Attribute>,
    public_id: String,
    system_id: String
}

pub struct Document {

}

pub struct DocumentFragment {

}

type Attribute = (String, String);

pub fn text(text: String) -> Node {
    Node { children: Vec::new(), node_type: NodeType::Text(text) }
}

pub fn element(children: Vec<Element>, attributes: Vec<Attribute>, tag: String) -> Node {
    Node {
        children,
        node_type: NodeType::Element(Element { attributes, tag }),
    }
}

pub fn document(children: Vec<Element>) -> Node {
    Node {
        children,
        node_type: NodeType::Document(Document {}),
    }
}

pub enum NodeType {
    Text(String),
    Attribute(Attribute),
    Element(Element),
    CDataSection(String),
    ProcessingInstruction(String),
    Comment(String),
    Document(Document),
    DocumentType(DocumentType),
    DocumentFragment(DocumentFragment)
}