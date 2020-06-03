use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, PartialEq, Eq)]
pub struct TreeNode<T> {
    pub val: T,
    pub childs: Vec<Rc<RefCell<TreeNode<T>>>>,
    pub parent: Option<Rc<RefCell<TreeNode<T>>>>,
}

impl<T> TreeNode<T> {
    pub(crate) fn add_child(
        parent: Rc<RefCell<TreeNode<T>>>,
        child_data: T,
    ) -> Rc<RefCell<TreeNode<T>>> {
        let child_node = TreeNodeBuilder::new(child_data)
            .parent(parent.clone())
            .build_ref();
        parent.borrow_mut().childs.push(child_node.clone());
        child_node
    }

    pub(crate) fn backspace<F>(
        node: Rc<RefCell<TreeNode<T>>>,
        fn_until: F,
    ) -> Rc<RefCell<TreeNode<T>>>
    where
        F: Fn(&T) -> bool,
    {
        let mut node = node;
        while !fn_until(&node.borrow().val) {
            node = {
                let temp = node.borrow();
                temp.parent.as_ref().unwrap().clone()
            };
        }
        node.clone()
    }
}

pub struct TreeNodeBuilder<T> {
    inner: TreeNode<T>,
}

impl<T> TreeNodeBuilder<T> {
    #[inline]
    pub fn new(val: T) -> Self {
        TreeNodeBuilder {
            inner: TreeNode {
                val,
                childs: vec![],
                parent: None,
            },
        }
    }

    #[inline]
    pub fn child(mut self, node: TreeNode<T>) -> Self {
        let value = Rc::new(RefCell::new(node));
        self.inner.childs.push(value);
        self
    }

    #[inline]
    pub fn parent(mut self, node: Rc<RefCell<TreeNode<T>>>) -> Self {
        self.inner.parent = Some(node);
        self
    }

    #[inline]
    pub fn build(self) -> TreeNode<T> {
        self.inner
    }

    #[inline]
    pub fn build_ref(self) -> Rc<RefCell<TreeNode<T>>> {
        Rc::new(RefCell::new(self.inner))
    }
}
