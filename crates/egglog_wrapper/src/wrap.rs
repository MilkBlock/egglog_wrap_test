use std::{marker::PhantomData, sync::atomic::AtomicU32};

use symbol_table::GlobalSymbol;


pub trait LetStmtRx{
    fn receive(received:String);
    fn singleton() -> &'static Self;
}

pub trait EgglogTy{
    const TY_NAME : &'static str;
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

pub trait EgglogEnumSubTy{
    const TY_NAME:&'static str;
}
/// instance of EgglogTy
#[derive(Debug, Clone, ::derive_more::Deref)]
pub struct Node<T:EgglogTy, R:LetStmtRx, I:NodeInner<T> ,S:EgglogEnumSubTy>{
    pub ty : I,
    #[deref]
    pub sym : Sym<T>,
    pub p: PhantomData<R>,
    pub s: PhantomData<S>
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
        self.inc();
        Sym{  
            inner :format!("{}{}",T::TY_NAME,self.inc()).into(),
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

impl EgglogEnumSubTy for (){
    const TY_NAME:&'static str = "()";
}
impl LetStmtRx for (){
    fn receive(_received:String) {
        todo!()
    }
    fn singleton() -> &'static Self {
        todo!()
    }
}
