use std::ops::{Deref, DerefMut, Index};
use std::mem;
use std::fmt::{Debug, Display};

#[derive(Debug)]
struct ContiguousKDNode<P> {
    m_axis: usize,
    m_midpoint: P,
    m_dirs: [Option<usize>; 2], // m_dirs[0] -> Left (less than) or equal to, m_dirs[1] -> Greater than
}

type CKDNode<P> = ContiguousKDNode<P>;

/** Wrapper type with `Deref` implementation. Allows CKDNodes to accept any type that dereferences into 
  another type with the index trait. */
  #[derive(Debug)]
pub struct CKDWrapper<T>(T);
impl<T> Deref for CKDWrapper<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        return &self.0;
    }
}

impl<T> DerefMut for CKDWrapper<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        return &mut self.0;
    }
}

impl<T: Display> Display for CKDWrapper<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return self.0.fmt(f);
    }
}

#[derive(Debug)]
pub struct ContiguousKDTree<P> {
    m_k: usize,
    m_nodelist: Vec<CKDNode<P>>
}

pub type CKDTree<P> = ContiguousKDTree<P>;

impl<T: Index<usize>> CKDNode<CKDWrapper<T>> {

    /** Construct a `CKDNode` given some point `T`, where T has the trait `Index<usize>`. T
     is moved into the node and wrapped inside a `CKDWrapper` that implements `Deref<T>`. 
     The type of the returned CKDNode can be thought of is `CDKNode<CKDWrapper<T>>`.*/
    fn new(axis: usize, point: T, left_idx: Option<usize>, right_idx: Option<usize>) -> Self {
        return CKDNode {
            m_axis: axis,
            m_midpoint: CKDWrapper(point),
            m_dirs: [left_idx, right_idx]
        }
    }

    /** Construct an <b>"empty"</b> `CKDNode` given some point `T`, where T has the trait `Index<usize>`. T
     is moved into the node and wrapped inside a `CKDWrapper` that implements `Deref<T>`. 
     The type of the returned CKDNode can be thought of is `CDKNode<CKDWrapper<T>>`.*/
    fn new_empty(axis: usize, point: T) -> Self {
        return Self::new(axis, point, None, None);
    }
}

pub struct Iter<'a, P> {
    m_buffer: &'a Vec<CKDNode<P>>,
    m_next: usize
}

#[derive(Debug)]
enum VisitState {
    Visited = 1,
    NotVisited = 0
}

type StackLayer = (usize, VisitState);

pub enum TreeOrder {
    IN_ORDER = 0,
    PRE_ORDER = 1
}

pub struct TreeIter<'a, P> {
    m_buffer: &'a Vec<CKDNode<P>>,
    m_stack: Vec<StackLayer>,
    m_order: TreeOrder
}

pub struct TreeTravelIter<'a, 'b, P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize> {
    m_node_list: &'a Vec<CKDNode<P>>,
    m_node_at: Option<&'a CKDNode<P>>,
    m_ref_point: &'b <P as Deref>::Target
}

impl<'a, 'b, P> Iterator for TreeTravelIter<'a, 'b, P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize>,
    <<P as Deref>::Target as Index<usize>>::Output: PartialOrd + Copy {
    type Item = (usize, &'a <P as Deref>::Target);

    fn next(&mut self) -> Option<Self::Item> {
        match self.m_node_at {
            Some(node) => {
                let out_data: (usize, &<P as Deref>::Target) = (node.m_axis, node.m_midpoint.deref());
                self.m_node_at = node.travel(self.m_ref_point).and_then(| idx | Some(&self.m_node_list[idx]));
                Some(out_data)
            },

            None => None
        }
    }
}

impl<T> CKDNode<T> {

    fn get_left(&self) -> &Option<usize> {
        return &self.m_dirs[0];
    }

    fn get_right(&self) -> &Option<usize> {
        return &self.m_dirs[1];
    }

    /** Set the left link of this CDKNode */
    fn set_left(&mut self, left_idx: Option<usize>) -> &mut Self {
        self.m_dirs[0] = left_idx;
        return self;
    }
        
    /** Set the right link of this CDKNode */
    fn set_right(&mut self, right_idx: Option<usize>) -> &mut Self {
        self.m_dirs[1] = right_idx;
        return self;
    }

    /** Checks whether this node is a leaf or not (i.e. both links are None). Returns true if it is
    a leaf and false if it is not. */
    fn is_leaf(&self) -> bool {
        return self.m_dirs[0].is_none() && self.m_dirs[1].is_none();
    }
}

impl<P> CKDNode<P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize>,
    <<P as Deref>::Target as Index<usize>>::Output: PartialOrd + Copy {

    /** Construct a `CKDNode` given some point `P`, where P has the trait `Deref<Target = Index<usize>>`. */
    fn new_from_container(axis: usize, point: P, left_idx: Option<usize>, right_idx: Option<usize>) -> Self {
        return CKDNode {
            m_axis: axis,
            m_midpoint: point,
            m_dirs: [left_idx, right_idx]
        };
    }

    /** Constructs an empty `CDKNode` from a "wrapped" inputted point. */
    fn new_empty_from_container(axis: usize, point: P) -> Self {
        return Self::new_from_container(axis, point, None, None);
    }

    /** Compares this node with another point along this nodes axis */
    fn compare<'other, T>(&self, point: &'other T) -> bool 
    where T: AsRef<<P as Deref>::Target>, {
        return point.as_ref()[self.m_axis] > self.m_midpoint[self.m_axis];
    }

    /** Returns the value held in this node's `m_midpoint` field when indexed by
     this node's `m_axis`. */
    fn get_axis_value(&self) -> <<P as Deref>::Target as Index<usize>>::Output {
        return self.m_midpoint[self.m_axis];
    }

    /** Compares the midpoint of this CKDNode with the borrowed point along the axis of
     this CKDNode. If the input point is GREATER, travel to the right. If it is LESS
     THAN OR EQUAL TO, travel to the left. Returns the index of the next left or right
     node in an Option<usize>. If there is no 'next' in that location None is returned. */
    fn travel<'other>(&self, point: &'other <P as Deref>::Target) -> Option<usize> {
        return self.m_dirs[(point[self.m_axis] > self.m_midpoint[self.m_axis]) as usize];
    }

    /** Compares the midpoint of this CKDNode with the borrowed point along the axis of
     this CKDNode. If the input point is greater, travel to the right. If it is less
     than or equal to, travel to the left. Returns a mutable reference to this CKDNode's left
     or right Option<usize> values. */
    fn travel_mut<'other>(&mut self, point: &'other <P as Deref>::Target) -> &mut Option<usize> {
        return &mut self.m_dirs[(point[self.m_axis] > self.m_midpoint[self.m_axis]) as usize];
    }

    /** Can be thought of as the <b>negation</b> of CKDNode::travel().
     Compares the midpoint of this CKDNode with the borrowed point along the axis of
     this CKDNode. If the input point is LESS THAN OR EQUAL TO, travel to the right. If it is 
     GREATER THAN, travel to the left. */
    fn travel_invert<'other>(&self, point: &'other <P as Deref>::Target) -> Option<usize> {
        return self.m_dirs[!(point[self.m_axis] > self.m_midpoint[self.m_axis]) as usize];
    }

    /** Can be thought of as the <b>negation</b> of CKDNode::travel_mut().
     Compares the midpoint of this CKDNode with the borrowed point along the axis of
     this CKDNode. If the input point is LESS THAN OR EQUAL TO, travel to the right. If it is 
     GREATER THAN, travel to the left. */
    fn travel_invert_mut<'other>(&mut self, point: &'other <P as Deref>::Target) -> &mut Option<usize> {
        return &mut self.m_dirs[!(point[self.m_axis] > self.m_midpoint[self.m_axis]) as usize];
    }
}

impl<T> CKDTree<CKDWrapper<T>> 
where 
    T: Index<usize>,
    <T as Index<usize>>::Output: PartialOrd + Copy {

    /** Insert an unwrapped Indexable element `T` into the CKDTree as `CKDWrapper<T>` */
    pub fn push(&mut self, point: T) -> &mut Self {
        if self.m_nodelist.len() == 0 {
            self.m_nodelist.push(CKDNode::new_empty(0, point));
            return self;
        }

        let new_idx = self.m_nodelist.len();
        let (last_node, prev_idx): (&mut CKDNode<CKDWrapper<T>>, usize) = self.get_last_mut(&point);
        
        _ = mem::replace(last_node.travel_mut(&point), Some(new_idx));
        self.m_nodelist.push(CKDNode::new_empty(self.next_axis_from_idx(prev_idx), point));
        return self;
    }
}

impl<P> CKDTree<P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize>,
    <<P as Deref>::Target as Index<usize>>::Output: PartialOrd + Copy {

    /** Creates a new CKDTree */
    pub fn new(dimensions: usize) -> Self {
        return CKDTree {
            m_k: dimensions,
            m_nodelist: Vec::new()
        };
    }

    /** Returns the next `&CKDNode` in this `CKDTree` given the current &CKDNode 'node' and a comparison point */
    fn to_next<'other, T>(&self, node: &CKDNode<P>, point: &'other P) -> Option<&CKDNode<P>>
    where 
        T: Deref,
        <T as Deref>::Target: Index<usize>,
        <<T as Deref>::Target as Index<usize>>::Output: PartialOrd + Copy {
        return node.travel(point).and_then(|next_idx: usize| Some(&self.m_nodelist[next_idx]));
    }

    // Returns the next 
    fn to_next_mut<'other>(&mut self, node: &mut CKDNode<P>, point: &'other P) -> Option<&mut CKDNode<P>> {
        return node.travel(point).and_then(|next_idx: usize| Some(&mut self.m_nodelist[next_idx]));
    }

    /** Retreive a `&CKDNode` by index */
    fn get(&self, idx: usize) -> &CKDNode<P> {
        return &self.m_nodelist[idx];
    }

    /** Retreive a `&mut CKDNode` by index */
    fn get_mut(&mut self, idx: usize) -> &mut CKDNode<P> {
        return &mut self.m_nodelist[idx];
    }

    /** Given an index, returns the next axis a succeeding node would have */
    fn next_axis_from_idx(&self, idx: usize) -> usize {
        return (self.m_nodelist[idx].m_axis + 1) % self.m_k;
    }

    /** Given a `&CKDNode`, returns the axis a suceeding node from this one would have */
    fn next_axis(&self, node: &CKDNode<P>) -> usize {
        return (node.m_axis + 1) % self.m_k;
    }

    /** Returns the size of this CKDNode */
    pub fn size(&self) -> usize {
        return self.m_nodelist.len();
    }

    /** Returns a reference to the last CKDNode before the input point (if it were to be inserted),
    along with the index of that point.*/
    fn get_last<'other>(&self, point: &'other <P as Deref>::Target) -> (&CKDNode<P>, usize) {
        let mut prev_idx: usize = 0;
        let mut next_idx: Option<usize> = Some(0);
        let mut cur_node: &CKDNode<P> = self.get(0);

        while next_idx.is_some() {
            let cur_idx: usize = next_idx.unwrap();
            cur_node = self.get(cur_idx);
            prev_idx = cur_idx;
            next_idx = cur_node.travel(point);
        }

        return (cur_node, prev_idx);
    }

    /** Returns a mutable reference to the last CKDNode before the input point (if it were to be inserted)
    along with the index of that point. */
    fn get_last_mut<'other>(&mut self, point: &'other <P as Deref>::Target) -> (&mut CKDNode<P>, usize) {

        let mut prev_idx: usize = 0;
        let mut next_idx: Option<usize> = Some(0);
        let mut cur_node:  *mut CKDNode<P> = self.get_mut(0);

        while next_idx.is_some() {
            let cur_idx: usize = next_idx.unwrap();
            cur_node = self.get_mut(cur_idx);
            prev_idx = cur_idx;
            unsafe { next_idx = (*cur_node).travel(point); }
        }

        return unsafe { (cur_node.as_mut().unwrap(), prev_idx) };
    }

    /** Insert a Deref wrapper `W` holding an Indexable `T` into the CKDTree as `W<T>`  */
    pub fn push_wrapped<'other>(&mut self, point: P) -> &mut Self {
        if self.m_nodelist.len() == 0 {
            self.m_nodelist.push(CKDNode::new_empty_from_container(0, point));
            return self;
        }

        let new_idx = self.m_nodelist.len();
        let (last_node, prev_idx): (&mut CKDNode<P>, usize) = self.get_last_mut(&point);
        
        _ = mem::replace(last_node.travel_mut(&point), Some(new_idx));
        self.m_nodelist.push(CKDNode::new_empty_from_container(self.next_axis_from_idx(prev_idx), point));
        return self;
    }

    pub fn iter<'parent>(&'parent self) -> Iter<'parent, P> {
        return Iter {
            m_buffer: &self.m_nodelist,
            m_next: 0
        }
    }

    pub fn tree_iter<'parent>(&'parent self, order: TreeOrder) -> TreeIter<'parent, P> {
        return TreeIter {
            m_buffer: &self.m_nodelist,
            m_stack: if self.size() == 0 { Vec::new() } else { vec![(0, VisitState::NotVisited)] },
            m_order: order
        };
    }

    pub fn tree_travel_iter<'parent, 'other>(&'parent self, point: &'other <P as Deref>::Target) -> TreeTravelIter<'parent, 'other, P> {
        return TreeTravelIter {
            m_node_list: &self.m_nodelist,
            m_node_at: self.m_nodelist.get(0),
            m_ref_point: point
        };
    }
}

impl<'parent, P> Iterator for Iter<'parent, P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize> {

    type Item = (usize, &'parent <P as Deref>::Target);
    fn next(&mut self) -> Option<Self::Item> {
        let out = self.m_buffer.get(self.m_next).and_then(| node | Some((node.m_axis, node.m_midpoint.deref()) )); 
        self.m_next += 1;
        return out;
    }
}

impl<'a, P> TreeIter<'a,P> {
    fn peek_node(&self) -> &ContiguousKDNode<P> {
        return &self.m_buffer[self.m_stack[self.m_stack.len() - 1].0];
    }

    fn peek(&self) -> Option<&StackLayer> {
        return self.m_stack.get(self.m_stack.len().wrapping_sub(1));
    }

    fn peek_mut(&mut self) -> &mut StackLayer {
        let len = self.m_stack.len() - 1;
        return &mut self.m_stack[len];
    }

    fn is_empty(&self) -> bool {
        return self.m_stack.is_empty();
    }
}

impl<'a, P> TreeIter<'a, P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize> {

    /** Advances the iterator to the next node in the kdtree following an in-order heuristic */
    fn next_in_order<'b>(&'b mut self) -> Option<(usize, &'a <P as Deref>::Target)> {
        if self.m_stack.len() == 0 { return None; }

        let stack_data: &mut (usize, VisitState) = self.peek_mut();
        let cur_idx: usize = stack_data.0; // store now so i don't have to deal with the borrow checker when indexing

        return Some(
            match stack_data.1 {
                VisitState::NotVisited => {
                    stack_data.1 = VisitState::Visited;
                    let mut cur_node: &ContiguousKDNode<P> = &self.m_buffer[cur_idx];
                    while cur_node.m_dirs[0].is_some() {
                        self.m_stack.push((cur_node.m_dirs[0].unwrap(), VisitState::Visited));
                        cur_node = &self.m_buffer[cur_node.m_dirs[0].unwrap()];
                    }
                    
                    self.m_stack.pop();
                    if cur_node.m_dirs[1].is_some() { self.m_stack.push((cur_node.m_dirs[1].unwrap(), VisitState::NotVisited)); }
                    (cur_node.m_axis, cur_node.m_midpoint.deref()) // output of the match
                },

                VisitState::Visited => {
                    let cur_node: &ContiguousKDNode<P> = &self.m_buffer[cur_idx];
                    self.m_stack.pop();
                    if cur_node.m_dirs[1].is_some() { self.m_stack.push((cur_node.m_dirs[1].unwrap(), VisitState::NotVisited)); }
                    (cur_node.m_axis, cur_node.m_midpoint.deref()) // output of the match (2)
                }
            }
        );
    }

    /** Advances the iterator to the next node in the KDTree following a Pre-Order heuristic */
    fn next_pre_order<'b>(&'b mut self) -> Option<(usize, &'a <P as Deref>::Target)> {
        // backtrack case
        while let Some((i, VisitState::Visited)) = self.peek() {
            let backtrack_idx: usize = *i;

            self.m_stack.pop();
            let backtrack_node: &ContiguousKDNode<P> = &self.m_buffer[backtrack_idx];
            if backtrack_node.m_dirs[1].is_some() { self.m_stack.push((backtrack_node.m_dirs[1].unwrap(), VisitState::NotVisited)); }
        }

        if self.is_empty() { return None; } // effectively the "base" case

        let stack_data: &mut (usize, VisitState) = self.peek_mut();
        let cur_idx: usize = stack_data.0;

        stack_data.1 = VisitState::Visited;
        let out_node: &'a ContiguousKDNode<P> = &self.m_buffer[cur_idx];
        if out_node.m_dirs[0].is_some() { self.m_stack.push((out_node.m_dirs[0].unwrap(), VisitState::NotVisited));}
        return Some((out_node.m_axis, out_node.m_midpoint.deref()));
    }
}

impl<'a, P> Iterator for TreeIter<'a, P> 
where 
    P: Deref,
    <P as Deref>::Target: Index<usize> + Sized {
    type Item = (usize, &'a <P as Deref>::Target);
    fn next(&mut self) -> Option<(usize, &'a <P as Deref>::Target)> {
        match self.m_order {
            TreeOrder::IN_ORDER => self.next_in_order(),
            TreeOrder::PRE_ORDER => self.next_pre_order(),
        }
    }
}

// impl<P> CKDTree<P>
// where
//     P: Deref<Target = Vec<f32>> {

//     pub fn from_median() -> 
// }

impl<P> CKDTree<P>
where
    P: Deref,
    <P as Deref>::Target: Index<usize,Output = f32>, {

    /** Find the nearest neighbor to the input point among the points stored in this `CKDTree` based upon a distance function `distance_func()`.*/
    pub fn nearest_neighbor<'t>(&'t self, point: & <P as Deref>::Target, distance_func: impl Fn(& <P as Deref>::Target, & <P as Deref>::Target) -> f32) -> Option<&<P as Deref>::Target> {
        if self.m_nodelist.len() == 0 { return None; }
        type Point<P> = <P as Deref>::Target;
        let (mut vec_stack, mut nearest, mut min_dist) = (vec![(0, VisitState::NotVisited)], None, f32::MAX);

        // Called whenever the input point's distance must be compared to a node in the tree
        let update_nearest = |mut v_stack: Vec<StackLayer>, mut near: Option<&'t Point<P>>, mut old_dist: f32, node_at: &'t CKDNode<P>| {
            v_stack.pop();
            let dist = distance_func(&node_at.m_midpoint, point);
            if dist < old_dist { (near, old_dist) = (Some(&*node_at.m_midpoint), dist); }
            return (v_stack, near, old_dist)
        };

        while vec_stack.len() > 0 {
            let last_idx: usize = vec_stack.len() - 1;
            let top: &mut (usize, VisitState) = &mut vec_stack[last_idx];
            let cur_node: &CKDNode<P> = self.get(top.0);

            match top.1 {
                VisitState::Visited => { // visited case: first update the closest, then check for overlap w/ 
                    (vec_stack, nearest, min_dist) = update_nearest(vec_stack, nearest, min_dist, &cur_node);
                    if f32::abs(cur_node.get_axis_value() - point[cur_node.m_axis]) < min_dist {
                        cur_node.travel_invert(point).and_then(|next_idx| { Some(vec_stack.push((next_idx, VisitState::NotVisited))) });
                    }
                },

                VisitState::NotVisited => { // unvisited case: if at leaf we update closest, otherwise we keep moving down the tree
                    top.1 = VisitState::Visited;
                    if cur_node.is_leaf() { (vec_stack, nearest, min_dist) = update_nearest(vec_stack, nearest, min_dist, &cur_node); }
                    else { cur_node.travel(point).and_then(|next_idx| Some(vec_stack.push((next_idx, VisitState::NotVisited))) ); }
                }
            }
        }
        return nearest;
    }
}

#[cfg(test)]
mod tests {
    use num::pow::Pow;

    use super::ContiguousKDTree;
    use super::CKDTree;
    use super::CKDWrapper;
    use super::TreeIter;
    use super::VisitState;
    use super::TreeOrder;

    use std::ops::Index;
    use std::sync::Arc;

    #[test]
    fn no_wrapper_new_kdtree_test() {
        let vec: Vec<f32> = vec![1f32, 10f32];
        let mut tree = ContiguousKDTree::new(2);
        tree.push(vec![32f32, 22f32]);
        tree.push(vec![0f32, 25f32]);
        tree.push(vec![11f32, 2f32]);

        println!("{:?}", tree);

    }

    #[test]
    fn wrapper_new_kdtree_test() {
        let vec: Vec<f32> = vec![1f32, 10f32];
        let mut tree = CKDTree::new(2);
        tree.push_wrapped(Box::new(vec));
        tree.push_wrapped(Box::new(vec![32f32, 22f32]));
        tree.push_wrapped(Box::new(vec![0f32, 25f32]));
        tree.push_wrapped(Box::new(vec![11f32, 2f32]));

        println!("{:?}", tree);
    }

    #[derive(Debug)]
    struct Point(f32,f32);
    impl Point {
        fn new(x: f32, y: f32) -> Self {
            return Point(x,y);
        }

        fn new_i32(x: i32, y: i32) -> Self {
            return Point(x as f32, y as f32);
        }

        fn distance(&self, other: &Self) -> f32 {
            return ((other.0 - self.0).powi(2) + (other.1 - self.1).powi(2)).sqrt();
        }

        fn distance_squared(&self, other: &Self) -> f32 {
            return (other.0 - self.0).powi(2) + (other.1 - self.1).powi(2);
        }

        fn as_array(&self) -> [f32; 2] {
            return [self.0, self.1];
        }
    }

    impl Index<usize> for Point {
        type Output = f32;
        fn index(&self, index: usize) -> &Self::Output {
            let ways: [&f32; 2] = [&self.0, &self.1];
            return ways[index];
        }
    }

    #[test]
    fn arc_kdtree_test() {
        println!("Hello, world!");

        // INPUT POINTS, ARC MADE FOR DEMONSTRATION
        let point_vec: Vec<Point> = vec![Point::new_i32(1,1), Point::new_i32(-1, 5), Point::new_i32(7, 11), Point::new_i32(6, 9)];
        let arc_point_vec: Vec<Arc<Point>> = point_vec.into_iter().map(|p| Arc::new(p)).collect::<Vec<Arc<Point>>>();

        // THE TREE
        let mut arc_point_tree: ContiguousKDTree<Arc<Point>> = CKDTree::new(2);
        for item in arc_point_vec.iter() {
            arc_point_tree.push_wrapped(item.clone());
        }

        // NEAREST NEIGHBOR COMPARISONS TAKE THE UNDERLYING 'WRAPPED' TYPEs
        let mut point_tree: ContiguousKDTree<CKDWrapper<[f32; 2]>> = CKDTree::new(2);
        let test_points: [Point; 5] = [Point::new_i32(7, 3), Point::new_i32(-8, 12), Point::new_i32(7, 11), Point::new_i32(5, 4), Point::new_i32(-8, -6)];
        for pt in test_points {
            let closest: &Point = arc_point_tree.nearest_neighbor(&pt, |p1,p2| p1.distance_squared(p2)).unwrap();
            println!("input: {:?}, closest: {:?}", pt, closest);
            point_tree.push(pt.as_array()); // normal_tree gets ownership of each point now
        }

        for item in arc_point_vec.iter() {
            let closest: &[f32; 2] = point_tree.nearest_neighbor(&item.as_ref().as_array(), |p1,p2| (p2[1] - p1[1]).powi(2) + (p2[0] - p1[0]).powi(2)).unwrap();
            println!("input: {:?}, closest: {:?}", item.as_array(), closest);
        }
    }

    #[test]
    fn tree_in_order_iterator_test() {
        let mut foo: ContiguousKDTree<CKDWrapper<[i32; 2]>> = CKDTree::new(2);
        foo.push([1, 2]).push([1, 6]).push([0, 5]).push([-1, 7]).push([5, 9]).push([7, 11]);

        let tree_iter: TreeIter<CKDWrapper<[i32; 2]>> = foo.tree_iter(TreeOrder::IN_ORDER);
        let mut unwraps = 0;
        for item in tree_iter {
            println!("{:?}", item);
            unwraps += 1;
        }

        println!("we were able to unwrap {} times", unwraps);

    }

    #[test]
    fn tree_pre_order_iterator_test() {
        let mut foo: ContiguousKDTree<CKDWrapper<[i32; 2]>> = CKDTree::new(2);
        foo.push([1, 2]).push([1, 6]).push([0, 5]).push([-1, 7]).push([5, 9]).push([7, 11]);

        let tree_iter: TreeIter<CKDWrapper<[i32; 2]>> = foo.tree_iter(TreeOrder::PRE_ORDER);
        let mut unwraps = 0;
        for item in tree_iter {
            println!("{:?}", item);
            unwraps += 1;
        }  

        println!("we were able to unwrap {} times", unwraps);
    }

    #[test]
    fn empty_tree_iter() {
        let foo: CKDTree<CKDWrapper<[f32; 3]>> = CKDTree::new(3);
        for i in foo.tree_iter(TreeOrder::PRE_ORDER) {
            println!("{:?}", i);
        }
    }

    #[test]
    fn tree_travel_iter_test() {
        let mut foo: ContiguousKDTree<CKDWrapper<[i32; 2]>> = CKDTree::new(2);
        foo.push([1, 2]).push([1, 6]).push([0, 5]).push([-1, 7]).push([5, 9]).push([7, 11]);
        for i in foo.tree_travel_iter(&[1, 5]) {
            println!("axis: {}, point: {:?}", i.0, i.1);
        }

    }
}