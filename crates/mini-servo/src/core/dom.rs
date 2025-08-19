use crate::core::node::Node;

pub(crate) trait Dom {
    type Document;
    type Node: Node;
}
