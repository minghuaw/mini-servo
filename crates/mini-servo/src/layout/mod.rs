mod element;
mod iter;
mod node;
mod safe_element;
mod safe_node;

pub use element::*;
pub use iter::*;
pub use node::*;
pub use safe_element::*;
pub use safe_node::*;

pub type BlitzNode<'dom> = &'dom blitz_dom::Node;

#[cfg(test)]
mod tests {
    
}
