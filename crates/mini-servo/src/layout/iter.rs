use blitz_dom::stylo::Traverser;

use crate::layout::{BlitzLayoutElement, SafeBlitzLayoutNode};

use super::BlitzLayoutNode;

pub struct SafeBlitzChildrenIterator<'dom> {
    pub traverser: Traverser<'dom>,
}

impl<'dom> Iterator for SafeBlitzChildrenIterator<'dom> {
    type Item = SafeBlitzLayoutNode<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        self.traverser.next().map(|value| {
            let node = BlitzLayoutNode { value };
            SafeBlitzLayoutNode { node, pseudo: None } 
        })
    }
}

pub struct BlitzLayoutNodeIterator<'dom> {
    pub traverser: Traverser<'dom>,
}

impl<'dom> Iterator for BlitzLayoutNodeIterator<'dom> {
    type Item = BlitzLayoutNode<'dom>;

    fn next(&mut self) -> Option<Self::Item> {
        self.traverser.next().map(|value| BlitzLayoutNode { value })
    }
}