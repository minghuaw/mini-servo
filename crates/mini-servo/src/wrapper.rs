// #[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
// pub(crate) struct Wrapper<T>(pub T);

#[derive(Debug, Clone)]
pub(crate) struct RcNode(pub rcdom::Handle);
