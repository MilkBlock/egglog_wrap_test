use derive_more::{Debug, Deref, DerefMut, IntoIterator};
use egglog::ast::NCommand;
use impl_trait_for_tuples::impl_for_tuples;
use smallvec::SmallVec;
use std::{borrow::Borrow, fmt, hash::Hash, marker::PhantomData, sync::atomic::AtomicU32};
use symbol_table::GlobalSymbol;

#[derive(Debug)]
pub enum TxCommand {
    StringCommand { string_command: String },
    NativeCommand { native_command: NCommand },
}

pub trait Tx: 'static {
    /// receive is guaranteed to not be called in proc macro
    fn send(&self, sended: TxCommand);
    fn on_new(&self, node: &(impl EgglogNode + 'static));
    fn on_set(&self, node: &mut (impl EgglogNode + 'static));
    fn on_func_set<'a, F: EgglogFunc>(
        &self,
        input: <F::Input as EgglogFuncInputs>::Ref<'a>,
        output: <F::Output as crate::wrap::EgglogFuncOutput>::Ref<'a>,
    );
}
pub trait Rx: 'static {
    fn on_func_get<'a, 'b, F: EgglogFunc>(
        &self,
        input: <F::Input as EgglogFuncInputs>::Ref<'a>,
    ) -> <F::Output as EgglogFuncOutput>::Ref<'b>;
    fn on_funcs_get<'a, 'b, F: EgglogFunc>(
        &self,
        max_size: Option<usize>,
    ) -> Vec<(
        <F::Input as EgglogFuncInputs>::Ref<'b>,
        <F::Output as EgglogFuncOutput>::Ref<'b>,
    )>;
}

pub trait SingletonGetter {
    type RetTy;
    fn tx() -> &'static Self::RetTy;
}

pub trait TxSgl: 'static + Sized {
    // delegate all functions from Tx
    fn receive(received: TxCommand);
    fn on_new(node: &(impl EgglogNode + 'static));
    fn on_set(node: &mut (impl EgglogNode + 'static));
    fn on_func_set<'a, F: EgglogFunc>(
        input: <F::Input as EgglogFuncInputs>::Ref<'a>,
        output: <F::Output as EgglogFuncOutput>::Ref<'a>,
    );
    // fn on_func_get<'a,'b, F: EgglogFunc>(
    //     &self,
    //     input: <F::Input as EgglogFuncInputs>::Ref<'a>,
    // ) -> <F::Output as EgglogFuncOutput>::Ref<'b>;
    // fn on_funcs_get<'a,'b, F: EgglogFunc>(max_size:Option<usize>) -> Vec<(<F::Input as EgglogFuncInputs>::Ref<'b>, <F::Output as EgglogFuncOutput>::Ref<'b>)>;
}

impl<R: Tx + 'static, T: SingletonGetter<RetTy = R> + 'static> TxSgl for T {
    fn receive(received: TxCommand) {
        Self::tx().send(received);
    }
    fn on_new(node: &(impl EgglogNode + 'static)) {
        Self::tx().on_new(node);
    }
    fn on_set(node: &mut (impl EgglogNode + 'static)) {
        Self::tx().on_set(node);
    }

    fn on_func_set<'a, F: EgglogFunc>(
        input: <F::Input as EgglogFuncInputs>::Ref<'a>,
        output: <F::Output as EgglogFuncOutput>::Ref<'a>,
    ) {
        Self::tx().on_func_set::<F>(input, output);
    }
    // fn on_func_get<'a,'b, F: EgglogFunc>(
    //     &self,
    //     input: <F::Input as EgglogFuncInputs>::Ref<'a>,
    // ) -> <F::Output as EgglogFuncOutput>::Ref<'b> {
    //     Self::tx().on_func_get::<F>(input)
    // }

    // fn on_funcs_get<'a,'b, F: EgglogFunc>(max_size:Option<usize>) -> Vec<(<F::Input as EgglogFuncInputs>::Ref<'b>, <F::Output as EgglogFuncOutput>::Ref<'b>)> {
    //     Self::tx().on_funcs_get::<F>(max_size)
    // }
}

/// version control triat
/// which should be implemented by Tx
pub trait VersionCtl {
    fn locate_latest(&self, node: Sym) -> Sym;
    fn locate_next(&self, node: Sym) -> Sym;
    fn locate_prev(&self, node: Sym) -> Sym;
    fn set_latest(&self, node: &mut Sym);
    fn set_next(&self, node: &mut Sym);
    fn set_prev(&self, node: &mut Sym);
}

/// version control triat
/// which should be implemented by Tx
pub trait VersionCtlSgl {
    fn locate_latest(node: Sym) -> Sym;
    fn locate_next(node: Sym) -> Sym;
    fn locate_prev(node: Sym) -> Sym;
    fn set_latest(node: &mut Sym);
    fn set_next(node: &mut Sym);
    fn set_prev(node: &mut Sym);
}

impl<Ret: Tx + VersionCtl + 'static, S: SingletonGetter<RetTy = Ret>> VersionCtlSgl for S {
    fn locate_latest(node: Sym) -> Sym {
        Self::tx().locate_latest(node)
    }
    fn locate_next(node: Sym) -> Sym {
        Self::tx().locate_next(node)
    }
    fn locate_prev(node: Sym) -> Sym {
        Self::tx().locate_prev(node)
    }
    fn set_latest(node: &mut Sym) {
        Self::tx().set_latest(node)
    }
    fn set_next(node: &mut Sym) {
        Self::tx().set_next(node)
    }
    fn set_prev(node: &mut Sym) {
        Self::tx().set_prev(node)
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
    fn locate_prev(&mut self);
}
/// trait of node behavior
pub trait EgglogNode: ToEgglog {
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
    R: TxSgl,
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
    R: TxSgl,
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
/// count the number of nodes of specific EgglogTy for specific binding Tx
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
    pub prev: Option<Sym>,
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
            prev: None,
        }
    }
}
impl fmt::Debug for WorkAreaNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorkAreaNode")
            .field("preds", &self.preds)
            .field("prev", &self.prev)
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
            prev: None,
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
/// This trait should be implemented for Tx singleton
/// usage:
/// ```rust
/// let last_version_node = node.clone();
/// Tx::commit(&self, node);
/// ```
pub trait TxCommit {
    fn on_commit<T: EgglogNode>(&self, node: &T);
    fn on_stage<T: EgglogNode + ?Sized>(&self, node: &T);
}

pub trait TxCommitSgl {
    fn on_commit<T: EgglogNode>(node: &T);
    fn on_stage<T: EgglogNode>(node: &T);
}

impl<Ret, S> TxCommitSgl for S
where
    Ret: Tx + VersionCtl + TxCommit,
    S: SingletonGetter<RetTy = Ret>,
{
    fn on_commit<T: EgglogNode>(node: &T) {
        S::tx().on_commit(node);
    }

    fn on_stage<T: EgglogNode>(node: &T) {
        S::tx().on_stage(node);
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

impl<T: EgglogNode> From<T> for WorkAreaNode {
    fn from(value: T) -> Self {
        WorkAreaNode::new(value.clone_dyn())
    }
}

/// Trait for input types that can be used in egglog functions
pub trait EgglogFuncInput {
    type Ref<'a>: EgglogFuncInputRef;
    fn as_node(&self) -> &dyn EgglogNode;
}
/// Trait for input types that can be used in egglog functions
pub trait EgglogFuncInputs {
    type Ref<'a>: EgglogFuncInputsRef;
    fn as_nodes(&self) -> Box<[&dyn EgglogNode]>;
}
/// Trait for input types ref that directly used as function argument
pub trait EgglogFuncInputRef {
    type DeRef: EgglogFuncInput + EgglogNode;
    fn as_node(&self) -> &dyn EgglogNode;
}
pub trait EgglogFuncInputsRef {
    type DeRef: EgglogFuncInputs;
    fn as_nodes(&self) -> Box<[&dyn EgglogNode]>;
}

/// Trait for output types that can be used in egglog functions
pub trait EgglogFuncOutput {
    type Ref<'a>: EgglogFuncOutputRef;
    fn as_node(&self) -> &dyn EgglogNode;
}
impl<T> EgglogFuncOutput for T
where
    T: EgglogNode + 'static,
{
    type Ref<'a> = &'a dyn AsRef<T>;
    fn as_node(&self) -> &dyn EgglogNode {
        self
    }
}
impl<T: EgglogFuncOutput + EgglogNode + 'static> EgglogFuncOutputRef for &dyn AsRef<T> {
    type DeRef<'a> = T;
    fn as_node(&self) -> &dyn EgglogNode {
        self.as_ref()
    }
}
pub trait EgglogFuncOutputRef {
    type DeRef<'a>: EgglogFuncOutput;
    fn as_node(&self) -> &dyn EgglogNode;
}
pub trait EgglogFunc {
    type Input: EgglogFuncInputs;
    type Output: EgglogFuncOutput;
    const FUNC_NAME: &'static str;
}
impl<T> EgglogFuncInput for T
where
    T: EgglogNode + 'static,
{
    type Ref<'a> = &'a dyn AsRef<T>;
    fn as_node(&self) -> &dyn EgglogNode {
        self
    }
}
impl<T> EgglogFuncInputRef for &dyn AsRef<T>
where
    T: EgglogNode + 'static,
{
    type DeRef = T;
    fn as_node(&self) -> &dyn EgglogNode {
        self.as_ref()
    }
}
#[impl_for_tuples(0, 8)]
#[tuple_types_custom_trait_bound(EgglogNode + EgglogFuncInput)]
impl EgglogFuncInputs for Tuple {
    for_tuples!( type Ref<'a> = ( #( Tuple::Ref<'a> ),* ); );
    fn as_nodes(&self) -> Box<[&dyn EgglogNode]> {
        Box::new([for_tuples!(
            #(&self.Tuple),*
        )])
    }
}
#[impl_for_tuples(0, 8)]
#[tuple_types_custom_trait_bound(EgglogFuncInputRef)]
impl EgglogFuncInputsRef for TupleRef {
    for_tuples!( type DeRef = ( #( TupleRef::DeRef ),* ); );
    fn as_nodes(&self) -> Box<[&dyn EgglogNode]> {
        Box::new([for_tuples!(
            #(self.TupleRef.as_node()),*
        )])
    }
}
