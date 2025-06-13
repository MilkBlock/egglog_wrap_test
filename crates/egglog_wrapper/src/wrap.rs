use derive_more::{Debug, Deref, DerefMut, IntoIterator};
use smallvec::SmallVec;
use std::{
    borrow::Borrow,
    fmt,
    hash::Hash,
    marker::PhantomData,
    sync::atomic::AtomicU32,
};
use symbol_table::GlobalSymbol;

pub trait Rx: 'static {
    fn receive(&self, received: String);
    fn on_new(&self, node: &(impl EgglogNode + 'static));
    fn on_set(&self, node: &mut (impl EgglogNode + 'static));
}

pub trait SingletonGetter {
    type RetTy;
    fn rx() -> &'static Self::RetTy;
}

pub trait RxSgl: 'static {
    // delegate all functions from LetStmtRxInner
    fn receive(received: String);
    fn on_new(node: &(impl EgglogNode + 'static));
    fn on_set(node: &mut (impl EgglogNode + 'static));
}

impl<R: Rx + 'static, T: SingletonGetter<RetTy = R> + 'static> RxSgl for T {
    fn receive(received: String) {
        Self::rx().receive(received);
    }
    fn on_new(node: &(impl EgglogNode + 'static)) {
        Self::rx().on_new(node);
    }
    fn on_set(node: &mut (impl EgglogNode + 'static)) {
        Self::rx().on_set(node);
    }
}

/// version control triat
/// which should be implemented by Rx
pub trait VersionCtl {
    fn locate_latest(&self, node: Sym) -> Sym;
    fn locate_next(&self, node: Sym) -> Sym;
    fn set_latest(&self, node: &mut Sym);
    fn set_next(&self, node: &mut Sym);
}

/// version control triat
/// which should be implemented by Rx
pub trait VersionCtlSgl {
    fn locate_latest(node: Sym) -> Sym;
    fn locate_next(node: Sym) -> Sym;
    fn set_latest(node: &mut Sym);
    fn set_next(node: &mut Sym);
}

impl<Ret: Rx + VersionCtl + 'static, S: SingletonGetter<RetTy = Ret>> VersionCtlSgl for S {
    fn locate_latest(node: Sym) -> Sym {
        Self::rx().locate_latest(node)
    }
    fn locate_next(node: Sym) -> Sym {
        Self::rx().locate_next(node)
    }
    fn set_latest(node: &mut Sym) {
        Self::rx().set_latest(node)
    }
    fn set_next(node: &mut Sym) {
        Self::rx().set_next(node)
    }
}

pub trait EgglogTy {
    const TY_NAME: &'static str;
    const TY_NAME_LOWER: &'static str;
    const SORT_DEF: TySort;
}
pub trait UpdateCounter<T: EgglogTy> {
    fn inc_counter(&mut self, counter: &mut TyCounter<T>) -> Sym<T>;
}
pub struct TySort(pub &'static str);
pub struct FuncSort(pub &'static str);
pub struct RelationSort(pub &'static str);

impl<T> Sym<T> {
    pub fn erase(&self) -> Sym<()> {
        // safety note: type erasure
        unsafe { *&*(self as *const Sym<T> as *const Sym) }
    }
    pub fn erase_ref(&self) -> &Sym<()> {
        // safety note: type erasure
        unsafe { &*(self as *const Sym<T> as *const Sym) }
    }
    pub fn erase_mut(&mut self) -> &mut Sym<()> {
        // safety note: type erasure
        unsafe { &mut *(self as *mut Sym<T> as *mut Sym) }
    }
}

/// trait of basic functions to interact with egglog
pub trait ToEgglog {
    fn to_egglog(&self) -> String;
}

/// version control triat
/// which should be implemented by Node
pub trait LocateVersion {
    fn locate_latest(&mut self);
    fn locate_next(&mut self);
}
/// trait of node behavior
pub trait EgglogNode: ToEgglog + 'static {
    fn succs_mut(&mut self) -> Vec<&mut Sym>;
    fn succs(&self) -> Vec<Sym>;
    /// set new sym and return the new sym
    fn next_sym(&mut self) -> Sym;
    // return current sym
    fn cur_sym(&self) -> Sym;
    fn cur_sym_mut(&mut self) -> &mut Sym;

    fn clone_dyn(&self) -> Box<dyn EgglogNode>;
}

// collect all sorts into inventory, so that we could send the definitions of types.
inventory::collect!(TySort);
inventory::collect!(FuncSort);
inventory::collect!(RelationSort);

pub trait EgglogEnumVariantTy: Clone + 'static {
    const TY_NAME: &'static str;
}
/// instance of specified EgglogTy & its VariantTy
#[derive(Debug, Clone)]
pub struct Node<T, R, I, S>
where
    T: EgglogTy,
    R: RxSgl,
    I: NodeInner<T>,
    S: EgglogEnumVariantTy,
{
    pub ty: I,
    pub sym: Sym<T>,
    pub _p: PhantomData<R>,
    pub _s: PhantomData<S>,
}

/// allow type erasure on S
impl<T, R, I, S> AsRef<Node<T, R, I, ()>> for Node<T, R, I, S>
where
    T: EgglogTy,
    R: RxSgl,
    I: NodeInner<T>,
    S: EgglogEnumVariantTy,
{
    fn as_ref(&self) -> &Node<T, R, I, ()> {
        // Safety notes:
        // 1. Node's memory layout is unaffected by PhantomData
        // 2. We're only changing the S type parameter from a concrete type to unit type (),
        //    which doesn't affect the actual data
        unsafe { &*(self as *const Node<T, R, I, S> as *const Node<T, R, I, ()>) }
    }
}

#[derive(PartialEq, Eq, Hash, Debug)]
pub struct Sym<T = ()> {
    pub inner: GlobalSymbol,
    pub p: PhantomData<T>,
}

impl<T> Sym<T> {
    pub fn new(global_sym: GlobalSymbol) -> Self {
        Self {
            inner: global_sym,
            p: PhantomData,
        }
    }
    pub fn as_str(&self) -> &'static str {
        self.inner.as_str()
    }
}
impl<T: std::clone::Clone> Copy for Sym<T> {}
impl<T> Clone for Sym<T> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            p: PhantomData,
        }
    }
}

pub trait NodeInner<T> {}
impl<T> std::fmt::Display for Sym<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.as_str())
    }
}
impl<T> From<Sym<T>> for &str {
    fn from(value: Sym<T>) -> Self {
        value.inner.as_str()
    }
}
impl<T: EgglogTy> From<Syms<T>> for Syms {
    fn from(value: Syms<T>) -> Self {
        value.into_iter().map(|s| s.erase()).collect()
    }
}
/// count the number of nodes of specific EgglogTy for specific binding Rx
pub struct TyCounter<T: EgglogTy> {
    counter: AtomicU32,
    t: PhantomData<T>,
}
impl<T: EgglogTy> TyCounter<T> {
    pub const fn new() -> Self {
        TyCounter {
            counter: AtomicU32::new(0),
            t: PhantomData,
        }
    }
    // get next symbol of specified type T
    pub fn next_sym(&self) -> Sym<T> {
        Sym {
            inner: format!("{}{}", T::TY_NAME_LOWER, self.inc()).into(),
            p: PhantomData::<T>,
        }
    }
    pub fn get_counter(&self) -> u32 {
        self.counter.load(std::sync::atomic::Ordering::Acquire)
    }
    /// counter increment atomically
    pub fn inc(&self) -> u32 {
        self.counter
            .fetch_add(1, std::sync::atomic::Ordering::AcqRel)
    }
}

impl EgglogEnumVariantTy for () {
    const TY_NAME: &'static str = "Unknown";
}

#[derive(DerefMut, Deref)]
pub struct WorkAreaNode {
    pub next: Option<Sym>,
    pub preds: Syms,
    #[deref]
    #[deref_mut]
    pub egglog: Box<dyn EgglogNode>,
}

impl Clone for WorkAreaNode {
    fn clone(&self) -> Self {
        Self {
            next: self.next.clone(),
            preds: self.preds.clone(),
            egglog: self.egglog.clone_dyn(),
        }
    }
}
impl fmt::Debug for WorkAreaNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolNode")
            .field("preds", &self.preds)
            .field("sym", &self.egglog.cur_sym())
            .field("succs", &self.egglog.succs())
            .finish()
    }
}
impl WorkAreaNode {
    pub fn new(node: Box<dyn EgglogNode>) -> Self {
        Self {
            preds: Syms::default(),
            egglog: node,
            next: None,
        }
    }
    pub fn succs_mut(&mut self) -> impl Iterator<Item = &mut Sym> {
        self.egglog.succs_mut().into_iter()
    }
    pub fn preds_mut(&mut self) -> impl Iterator<Item = &mut Sym> {
        self.preds.iter_mut()
    }
    pub fn preds(&self) -> impl Iterator<Item = &Sym> {
        self.preds.iter()
    }
}

impl Borrow<GlobalSymbol> for Sym {
    fn borrow(&self) -> &GlobalSymbol {
        &self.inner
    }
}

#[derive(Clone, Deref, DerefMut, IntoIterator, Debug, Default)]
pub struct Syms<T = ()> {
    #[into_iterator(owned, ref, ref_mut)]
    inner: SmallVec<[Sym<T>; 4]>,
}

impl From<SmallVec<[Sym; 4]>> for Syms {
    fn from(value: SmallVec<[Sym; 4]>) -> Self {
        Syms { inner: value }
    }
}

impl<S> FromIterator<Sym<S>> for Syms<S> {
    fn from_iter<T: IntoIterator<Item = Sym<S>>>(iter: T) -> Self {
        Syms {
            inner: iter.into_iter().collect(),
        }
    }
}
impl Syms {
    pub fn new() -> Self {
        Syms {
            inner: SmallVec::new(),
        }
    }
}

/// global commit
/// This trait should be implemented for Rx singleton
/// usage:
/// ```rust
/// let last_version_node = node.clone();
/// Rx::commit(&self, node);
/// ```
pub trait RxCommit {
    fn on_commit<T: EgglogNode>(&self, node: &T);
    fn on_stage<T: EgglogNode + ?Sized>(&self, node: &T);
}

pub trait RxCommitSgl {
    fn on_commit<T: EgglogNode>(node: &T);
    fn on_stage<T: EgglogNode>(node: &T);
}

impl<Ret, S> RxCommitSgl for S
where
    Ret: Rx + VersionCtl + RxCommit,
    S: SingletonGetter<RetTy = Ret>,
{
    fn on_commit<T: EgglogNode>(node: &T) {
        S::rx().on_commit(node);
    }
    
    fn on_stage<T: EgglogNode>(node: &T) {
        S::rx().on_stage(node);
    }
}

/// single node commit
/// This trait should be implemented for Node
/// usage:
/// ```rust
/// let last_version_node = node.clone();
/// node.set_a()
///     .set_b()
///     .commit();
/// ```
pub trait Commit {
    fn commit(&self);
    fn stage(&self);
}

/// In Egglog there are 2 ways to interact with egraph
/// 1. String of egglog code
/// 2. Vector of Egglog Command Struct
/// Use this Interpreter trait to concile them
///
/// Also there are
pub trait Interpreter {
    type Interpreted;
    fn interpret(interpreted: Self::Interpreted);
}

// pub trait EgglogNodeMarker{ }

impl<T: EgglogNode + Clone + 'static> From<T> for WorkAreaNode {
    fn from(value: T) -> Self {
        WorkAreaNode::new(Box::new(value.clone()))
    }
}

pub trait EgglogFunc<'a, R: RxSgl> {
    type Input;
    type Output: EgglogNode;
    const FUNC_NAME:&'static str;
}
