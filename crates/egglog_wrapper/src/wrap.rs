use std::{hash::Hash, io::IntoInnerError, marker::PhantomData, path::PathBuf, sync::{atomic::AtomicU32, Mutex, OnceLock}};
use derive_more::{Deref, DerefMut, IntoIterator};
use egglog::{util::IndexMap, EGraph, SerializeConfig};
use smallvec::{Array, SmallVec};
use symbol_table::{GlobalSymbol, Symbol};

use crate::collect_type_defs;

pub trait LetStmtRx{
    fn receive(received:String);
    fn singleton() -> &'static Self;
    fn add_symnode(symnode:SymbolNode);
    fn update_symnode(old:GlobalSymbol,symnode:SymbolNode);
}

pub trait EgglogTy{
    const TY_NAME : &'static str;
    const TY_NAME_LOWER: &'static str;
    const SORT_DEF:Sort;
}
pub trait UpdateCounter<T:EgglogTy>{
    fn inc_counter(&mut self, counter:&mut TyCounter<T>) -> Sym<T>;
}
pub struct Sort(pub &'static str);

pub trait ToEgglog{
    fn to_egglog(&self) -> String;
}

// collect all sorts into inventory, so that we could send the definitions of types.
inventory::collect!(Sort);

pub trait EgglogEnumVariantTy{
    const TY_NAME:&'static str;
}
/// instance of EgglogTy
#[derive(Debug, Clone, ::derive_more::Deref)]
pub struct Node<T:EgglogTy, R:LetStmtRx, I:NodeInner<T> ,S:EgglogEnumVariantTy>{
    pub ty : I,
    #[deref]
    pub sym : Sym<T>,
    pub p: PhantomData<R>,
    pub s: PhantomData<S>
}

impl<T: EgglogTy, R: LetStmtRx, I: NodeInner<T>, S: EgglogEnumVariantTy> AsRef<Node<T, R, I, ()>> for Node<T, R, I, S> {
    fn as_ref(&self) -> &Node<T, R, I, ()> {
        // Safety notes:
        // 1. Node's memory layout is unaffected by PhantomData
        // 2. We're only changing the S type parameter from a concrete type to unit type (),
        //    which doesn't affect the actual data
        unsafe {
            &*(self as *const Node<T, R, I, S> as *const Node<T, R, I, ()>)
        }
    }
}

#[derive(Debug,Clone,derive_more::Deref)]
pub struct Sym<T>{
    #[deref]
    pub inner:GlobalSymbol,
    pub p:PhantomData<T>
}


impl<T:Clone> Copy for Sym<T>{ }

pub trait NodeInner<T>{

}
// pub struct Sym<f>
impl<T> std::fmt::Display for Sym<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
impl<T> From<Sym<T>> for &str{
    fn from(value: Sym<T>) -> Self {
        value.inner.as_str()
    }
}
/// count the number of nodes of specific EgglogTy for specific binding Rx
pub struct TyCounter<T:EgglogTy>{
    counter:AtomicU32,
    t:PhantomData<T>,
}
impl<T:EgglogTy> TyCounter<T>{
    pub const fn new() -> Self{
        TyCounter{
            counter: AtomicU32::new(0),
            t: PhantomData
        }
    }
    pub fn next_sym(&self) -> Sym<T>{
        Sym{  
            inner :format!("{}{}",T::TY_NAME_LOWER,self.inc()).into(),
            p:PhantomData::<T>
        }
    }
    pub fn get_counter(&self) -> u32 {
        self.counter.load(std::sync::atomic::Ordering::Acquire)
    }
    /// 递增计数器
    pub fn inc(&self) -> u32{
        self.counter.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
    }
}

impl EgglogEnumVariantTy for (){
    const TY_NAME:&'static str = "Unknown";
}
impl LetStmtRx for (){
    fn receive(_received:String) {
        todo!()
    }
    fn singleton() -> &'static Self {
        todo!()
    }
    fn add_symnode(_:SymbolNode) {
        todo!()
    }
    
    fn update_symnode(old:GlobalSymbol,symnode:SymbolNode) {
        todo!()
    }
}

pub struct SymbolNode{
    pub sym  : GlobalSymbol,
    pub preds : Syms,
    pub succ : Syms,
}
impl Hash for SymbolNode{
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.sym.hash(state);
        self.preds.hash(state);
        self.succ.hash(state);
    }
}



pub struct RxInner{ 
    egraph: EGraph,
    map : IndexMap<GlobalSymbol, SymbolNode>
}

pub struct Rx{
    inner:Mutex<RxInner>
}
impl Rx{
    pub fn interpret(&self,s:String){
        println!("{}",s);
        let mut guard = self.inner.lock().unwrap();
        guard.egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    pub fn to_dot(&self,file_name:PathBuf){
        let mut guard = self.inner.lock().unwrap();
        let mut serialized = guard.egraph.serialize(SerializeConfig::default());
        // if args.serialize_split_primitive_outputs {
        //     serialized.split_classes(|id, _| egraph.from_node_id(id).is_primitive())
        // }
        // for _ in 0..args.serialize_n_inline_leaves {
        //     serialized.inline_leaves();
        // }

        // if we are splitting primitive outputs, add `-split` to the end of the file name
        // let serialize_filename = if args.serialize_split_primitive_outputs {
        //     input.with_file_name(format!(
        //         "{}-split",
        //         input.file_stem().unwrap().to_str().unwrap()
        //     ))
        // } else {
        //     input.clone()
        // };
        let dot_path = file_name.with_extension("dot");
        serialized
            .to_dot_file(dot_path.clone())
            .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    }
}


unsafe impl Send for Rx{ }
unsafe impl Sync for Rx{ }
impl LetStmtRx for Rx{
    fn receive(received:String) {
        Self::singleton().interpret(received);
    }

    fn add_symnode(symnode:SymbolNode){
        let mut guard = Self::singleton().inner.lock().unwrap();
        for node in &symnode.succ{
            guard.map.get_mut(node)
                .unwrap_or_else(||panic!("node {} not found", node.as_str())).preds.push(symnode.sym);
        }
        guard.map.insert(symnode.sym, symnode);
    }


    /// update all predecessor recursively in guest and send updated term by egglog repr to host
    /// when you update the node
    fn update_symnode(old:GlobalSymbol, symnode:SymbolNode){
        let mut guard = Self::singleton().inner.lock().unwrap();
        // insert the new node 
        // Self::add_symnode(symnode.clone);
        let old_node = guard.map.swap_remove(&old).unwrap();
        // old_node.preds
        
        for node in &old_node.preds{
            if !guard.map.contains_key(node){
                panic!("node {} not found", node.as_str())
            }else{
                guard.map.get_mut(node).unwrap().preds.push(symnode.sym);
            }
        }
    }
    
    fn singleton() -> &'static Self {
        static INSTANCE: OnceLock<Rx> = OnceLock::new();
        INSTANCE.get_or_init(||{
            Rx{
                inner: Mutex::new(RxInner{
                    egraph: {
                        let mut e = EGraph::default();
                        let type_defs = collect_type_defs();
                        println!("{}",type_defs);
                        e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
                        e
                    },
                    map: IndexMap::default(),
                })
            }
        })
    }
}


impl<T> From<Sym<T>>  for GlobalSymbol{
    fn from(value: Sym<T>) -> Self {
        value.inner
    }
}

#[derive(Clone,Deref,DerefMut,IntoIterator)]
pub struct TypedSyms<T>{
    #[into_iterator(owned, ref,  ref_mut)]
    inner:SmallVec<[Sym<T>;4]>
}

#[derive(Clone,Deref,DerefMut,IntoIterator)]
pub struct Syms{
    #[into_iterator(owned, ref,  ref_mut)]
    inner:SmallVec<[GlobalSymbol;4]>
}
impl<T> From<TypedSyms<T>> for Syms{
    fn from(value: TypedSyms<T>) -> Self {
        value.into_iter().collect()
    }
}
impl<S> FromIterator<Sym<S>> for Syms {
    fn from_iter<T: IntoIterator<Item = Sym<S>>>(iter: T) -> Self {
        Syms{
            inner: iter.into_iter().map(|x|x.inner).collect()
        }
    }
}
impl From<SmallVec<[GlobalSymbol;4]>> for Syms {
    fn from(value: SmallVec<[GlobalSymbol;4]>) -> Self {
        Syms { inner: value }
    }
}

impl<S> FromIterator<Sym<S>> for TypedSyms<S> {
    fn from_iter<T: IntoIterator<Item = Sym<S>>>(iter: T) -> Self {
        TypedSyms{
            inner: iter.into_iter().collect()
        }
    }
}
impl Syms{
    pub fn new() -> Self{
        Syms { inner: SmallVec::new() }
    }
}