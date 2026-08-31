use std::rc::Rc;

pub(super) type Callback0 = Rc<dyn Fn()>;
pub(super) type Callback1<A> = Rc<dyn Fn(A)>;
pub(super) type Callback2<A, B> = Rc<dyn Fn(A, B)>;
pub(super) type Callback3<A, B, C> = Rc<dyn Fn(A, B, C)>;
pub(super) type Callback4<A, B, C, D> = Rc<dyn Fn(A, B, C, D)>;
pub(super) type Callback5<A, B, C, D, E> = Rc<dyn Fn(A, B, C, D, E)>;
pub(super) type Callback6<A, B, C, D, E, F> = Rc<dyn Fn(A, B, C, D, E, F)>;
pub(super) type Callback7<A, B, C, D, E, F, G> = Rc<dyn Fn(A, B, C, D, E, F, G)>;
pub(super) type Callback8<A, B, C, D, E, F, G, H> = Rc<dyn Fn(A, B, C, D, E, F, G, H)>;
pub(super) type Callback9<A, B, C, D, E, F, G, H, I> = Rc<dyn Fn(A, B, C, D, E, F, G, H, I)>;
