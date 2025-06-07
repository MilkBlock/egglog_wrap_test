use std::{borrow::Borrow, collections::VecDeque, fmt, hash::Hash, marker::PhantomData, path::PathBuf, sync::{atomic::AtomicU32, Mutex, OnceLock}};
use dashmap::DashMap;
use derive_more::{Debug, Deref, DerefMut, IntoIterator};
use egglog::{util::{IndexMap, IndexSet}, EGraph, SerializeConfig};
use smallvec::SmallVec;
use symbol_table::{GlobalSymbol, Symbol};
use bevy_ecs::world::World;

use crate::collect_type_defs;

pub trait LetStmtRx:'static{
    fn receive(received:String);
    fn singleton() -> &'static Self;
    fn add_symnode(symnode:SymbolNode);
    fn update_symnode(old:Sym,symnode:SymbolNode);
    fn update_symnodes(iter:impl Iterator<Item=(Sym,SymbolNode)>);
    fn locate_latest(old:&mut Sym) -> Sym;
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

/// trait of basic functions to interact with egglog
pub trait ToEgglog{
    fn to_egglog(&self) -> String;
    fn locate_latest(&mut self);
}
/// trait of node behavior
pub trait EgglogNode:ToEgglog {
    fn succs_mut(&mut self)-> Vec<&mut Sym>;
    fn succs(&self)-> Vec<Sym>;
    /// set new sym and return the new sym
    fn next_sym(&mut self) -> Sym;
    // return current sym 
    fn cur_sym(&self) -> Sym;

    fn clone_dyn(&self) -> Box<dyn EgglogNode>;
}

// collect all sorts into inventory, so that we could send the definitions of types.
inventory::collect!(Sort);

pub trait EgglogEnumVariantTy :Clone{
    const TY_NAME:&'static str;
}
/// instance of specified EgglogTy
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

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash,Debug)]
pub struct Sym<T=()>{
    pub inner:GlobalSymbol,
    pub p:PhantomData<T>
}
impl<T> Sym<T>{
    pub fn detype(&self) -> Sym{
        Sym { inner: self.inner, p: PhantomData }
    }
    pub fn detype_mut(&mut self) -> &mut Sym{
        unsafe{&mut *(self as *mut Sym<T> as *mut Sym<()>)}
    }
    pub fn new(global_sym:GlobalSymbol) -> Self{
        Self{
            inner: global_sym,
            p: PhantomData,
        }
    }
    pub fn as_str(&self)-> &'static str{
        self.inner.as_str()
    }
}
impl<T: std::clone::Clone> Copy for Sym<T>{ }
impl<T> Clone for Sym<T>{
    fn clone(&self) -> Self {
        Self { inner: self.inner.clone(), p: PhantomData }
    }
}

pub trait NodeInner<T>{}
// pub struct Sym<f>
impl<T> std::fmt::Display for Sym<T>{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.inner.as_str())
    }
}
impl<T> From<Sym<T>> for &str{
    fn from(value: Sym<T>) -> Self {
        value.inner.as_str()
    }
}
impl<T:EgglogTy> From<Syms<T>> for Syms{
    fn from(value: Syms<T>) -> Self {
        value.into_iter().map(|s|s.detype()).collect()
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
    
    fn update_symnode(_:Sym,_:SymbolNode) {
        todo!()
    }
    
    fn update_symnodes(iter:impl Iterator<Item=(Sym,SymbolNode)>) {
        todo!()
    }
    
    fn locate_latest(old:&mut Sym) -> Sym {
        todo!()
    }
}

#[derive(DerefMut,Deref)]
pub struct SymbolNode{
    next : Option<Sym>,
    preds : Syms,
    #[deref]
    #[deref_mut]
    egglog : Box<dyn EgglogNode>,
}
impl Clone for SymbolNode{
    fn clone(&self) -> Self {
        Self { next: self.next.clone(), preds: self.preds.clone(), egglog: self.egglog.clone_dyn() }
    }
}
impl fmt::Debug for SymbolNode{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymbolNode")
            .field("preds", &self.preds)
            .field("sym", &self.egglog.cur_sym())
            .field("succs", &self.egglog.succs())
            .finish()
    }
}
impl SymbolNode{
    pub fn new(_:Sym, node:Box<dyn EgglogNode>) -> Self{
        Self{
            preds: Syms::default(),
            egglog: node,
            next: None,
        }
    }
    pub fn succs_mut(&mut self) -> impl Iterator<Item = &mut Sym>{
        self.egglog.succs_mut().into_iter()
    }
    pub fn preds_mut(&mut self) -> impl Iterator<Item = &mut Sym>{
        self.preds.iter_mut()
    }
    pub fn preds(&self) -> impl Iterator<Item = &Sym>{
        self.preds.iter()
    }
}
// impl Hash for SymbolNode{
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         self.egglog.cur_sym().hash(state);
//         self.preds.hash(state);
//     }
// }



#[derive(Deref,DerefMut)]
pub struct RxInner{ 
    egraph: EGraph,
    #[deref] #[deref_mut]
    world : World,
}
impl Borrow<GlobalSymbol> for Sym{
    fn borrow(&self) -> &GlobalSymbol {
        &self.inner
    }
}

pub struct Rx{
    inner : Mutex<RxInner>,
    map : DashMap<Sym,SymbolNode>
}
impl Rx{
    pub fn interpret(&self,s:String){
        let mut guard = self.inner.lock().unwrap();
        guard.egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    pub fn to_dot(&self,file_name:PathBuf){
        let guard = self.inner.lock().unwrap();
        let serialized = guard.egraph.serialize(SerializeConfig::default());
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
    // collect all ancestors of cur_sym, without cur_sym
    pub fn collect_symnode(cur_sym:Sym, index_set:&mut IndexSet<Sym>){
        let singleton = Self::singleton();
        let sym_node = singleton.map.get(&cur_sym).unwrap();
        let v = sym_node.preds.clone();
        drop(sym_node);
        for pred in v{
            if index_set.contains(&pred) || singleton.map.get(&pred).unwrap().next.is_some(){
                // do nothing
            }else {
                index_set.insert(pred.clone());
                Rx::collect_symnode(pred,index_set)
            }
        }
    }
    /// start node is asserted to be zero input degree 
    pub fn topo_sort(starts : IndexSet<Sym> ,index_set:&IndexSet<Sym>)-> Vec<Sym>{
        let singleton = Self::singleton();
        let map = &singleton.map;
        // init in degrees and out degrees 
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        ins.resize(index_set.len(), 0);
        outs.resize(index_set.len(), 0);
        for (i,(in_degree,out_degree)) in ins.iter_mut().zip(outs.iter_mut()).enumerate(){
            let sym = index_set[i];
            print!("{} ",sym);
            let node = map.get(&sym).unwrap();
            *in_degree = Rx::degree_in_subgraph(node.preds().into_iter().map(|x|*x), index_set);
            *out_degree = Rx::degree_in_subgraph(node.succs().into_iter(), index_set);
        }
        println!("");
        let mut rst = Vec::new();
        let mut wait_for_release = Vec::new();
        // start node should not have any out edges in subgraph
        for start in starts{
            assert_eq!(0, outs[index_set.get_index_of(&start).unwrap()]);
            wait_for_release.push(start);
        }
        while !wait_for_release.is_empty(){
            let popped = wait_for_release.pop().unwrap();
            for target in &map.get(&popped).unwrap().preds {
                let idx = index_set.get_index_of(target).unwrap();
                outs[idx] -= 1;
                println!("{:?}",outs);
                if outs[idx] == 0{
                    wait_for_release.push(*target);
                }
            }
            rst.push(popped);
        }
        rst
    }
    /// calculate the edges in the subgraph 
    pub fn degree_in_subgraph(nodes:impl Iterator<Item = Sym>, index_set: &IndexSet<Sym>) -> u32{
        nodes.fold(0,|acc,item| if index_set.contains(&item) {acc+1} else {acc})
    }
}


unsafe impl Send for Rx{ }
unsafe impl Sync for Rx{ }
// MARK: Receiver
impl LetStmtRx for Rx{
    /// locate the lastest version of the symbol
    fn locate_latest(old:&mut Sym) -> Sym{
        let map = &Self::singleton().map;
        let mut cur = *old;
        while let Some(newer) = map.get(&cur).unwrap().next{
            cur = newer;
        }
        *old = cur;
        cur
    }
    fn receive(received:String) {
        println!("start to receive");
        Self::singleton().interpret(received);
        println!("receive end");
    }

    fn add_symnode(mut symnode:SymbolNode){
        let singleton = Self::singleton();
        let sym = symnode.cur_sym();
        for node in symnode.succs_mut(){
            let node = &Rx::locate_latest( node);
            singleton.map.get_mut(node)
                .unwrap_or_else(||panic!("node {} not found", node.as_str()))
                .preds.push(sym);
        }
        singleton.map.insert(symnode.cur_sym(), symnode);
    }

    /// update all predecessor recursively in guest and send updated term by egglog repr to host
    /// when you update the node
    fn update_symnode(mut old:Sym, mut updated_symnode:SymbolNode){
        let mut index_set = IndexSet::default();
        let singleton = Self::singleton();

        let old = Rx::locate_latest(&mut old);

        // collect all syms that will change
        Rx::collect_symnode(old, &mut index_set);
        let mut old_node = singleton.map.get_mut(&old).unwrap();
        // chain old version and new version
        old_node.next = Some(updated_symnode.egglog.cur_sym());
        println!("set {} some",old_node.cur_sym());
        updated_symnode.preds = old_node.preds.clone();
        drop(old_node);
        let mut new_syms = vec![];
        // update all succs
        for &old_sym in index_set.iter(){
            let mut sym_node = singleton.map.get(&old_sym).unwrap().clone();
            let new_sym = sym_node.next_sym();

            // chain old version and new version
            singleton.map.get_mut(&old_sym).unwrap().next = Some(new_sym);
            println!("set {} some",old_sym);

            new_syms.push(new_sym);
            singleton.map.insert(new_sym, sym_node);
        }
        index_set.insert(old);
        let new_sym = updated_symnode.cur_sym();
        new_syms.push(updated_symnode.cur_sym());
        singleton.map.insert(updated_symnode.cur_sym() ,updated_symnode);
        // update all preds
        for &new_sym in &new_syms{
            let mut sym_node = singleton.map.get_mut(&new_sym).unwrap();
            for sym in sym_node.preds_mut(){
                if let Some(idx) =  index_set.get_index_of(&*sym){
                    *sym = new_syms[idx];
                }
            }
            for sym in sym_node.succs_mut(){
                if let Some(idx) =  index_set.get_index_of(&*sym){
                    *sym = new_syms[idx];
                }
            }
        }
        let mut s = "".to_owned();
        let topo = Rx::topo_sort(
            IndexSet::from_iter(Some(new_sym).into_iter()),
            &IndexSet::from_iter(new_syms.into_iter()));
        for new_sym in topo{
            println!("topo string plus");
            s += singleton.map.get(&new_sym).unwrap().egglog.to_egglog().as_str();
        }
        println!("start receive");
        Rx::receive(s);
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
                    world: World::new(),
                }),
                map: DashMap::default(),
            }
        })
    }
    
    fn update_symnodes(start_iter:impl Iterator<Item=(Sym,SymbolNode)>) {}
        
    //     let mut index_set = IndexSet::default();
    //     let mut starts_and_update = IndexMap::default();
    //     let mut guard = Self::singleton().inner.lock().unwrap();

    //     // collect all syms that will change
    //     for (old,updated_symnode) in start_iter{
    //         starts_and_update.insert(old,updated_symnode);
    //         Rx::collect_symnode(old, &mut guard.map, &mut index_set);
    //         let old_node = guard.map.swap_remove(&old).unwrap();
    //         updated_symnode.preds = old_node.preds;
    //     }
    //     let mut new_syms = vec![];
    //     // update relevant nodes to next version
    //     for &old_sym in index_set.iter(){
    //         let mut sym_node = guard.map.swap_remove(&old_sym).unwrap();
    //         let new_sym = sym_node.next_sym();
    //         new_syms.push(new_sym);
    //         guard.map.insert(new_sym, sym_node);
    //     }
    //     let new_sym = updated_symnode.cur_sym();
    //     new_syms.push(updated_symnode.cur_sym());
    //     guard.map.insert(updated_symnode.cur_sym() ,updated_symnode);
    //     // update all preds & succs
    //     for &new_sym in &new_syms{
    //         let sym_node = guard.map.get_mut(&new_sym).unwrap();
    //         for sym in sym_node.preds_mut(){
    //             if let Some(idx) =  index_set.get_index_of(&*sym){
    //                 *sym = new_syms[idx];
    //             }
    //         }
    //         for sym in sym_node.succs_mut(){
    //             if let Some(idx) =  index_set.get_index_of(&*sym){
    //                 *sym = new_syms[idx];
    //             }
    //         }
    //     }
    //     let mut s = "".to_owned();
    //     let topo = Rx::topo_sort(
    //         IndexSet::from_iter(Some(new_sym).into_iter()),
    //         &guard.map,
    //         &IndexSet::from_iter(new_syms.into_iter()));
    //     for new_sym in topo{
    //         s += guard.map.get(&new_sym).unwrap().egglog.to_egglog().as_str();
    //     }
    //     drop(guard);
    //     Rx::receive(s);
    // }
}


#[derive(Clone,Deref,DerefMut,IntoIterator,Debug,Default)]
pub struct Syms<T=()>{
    #[into_iterator(owned, ref,  ref_mut)]
    inner:SmallVec<[Sym<T>;4]>
}

impl From<SmallVec<[Sym;4]>> for Syms {
    fn from(value: SmallVec<[Sym;4]>) -> Self {
        Syms { inner: value }
    }
}

impl<S> FromIterator<Sym<S>> for Syms<S> {
    fn from_iter<T: IntoIterator<Item = Sym<S>>>(iter: T) -> Self {
        Syms{
            inner: iter.into_iter().collect()
        }
    }
}
impl Syms{
    pub fn new() -> Self{
        Syms { inner: SmallVec::new() }
    }
}