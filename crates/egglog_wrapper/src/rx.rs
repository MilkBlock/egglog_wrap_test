use std::{path::PathBuf, sync::Mutex};
use dashmap::DashMap;
use egglog::{util::IndexSet, EGraph, SerializeConfig};
use crate::{collect_type_defs, wrap::{Rx, Sym, SymbolNode}};

pub struct RxNoVT{
    egraph : Mutex<EGraph>,
    map : DashMap<Sym,SymbolNode>,
    latest_map : DashMap<Sym, Sym>
}

/// Rx with version ctl feature
impl RxNoVT{
    pub fn new_with_type_defs(type_defs:String) -> Self{
        Self {
            egraph: Mutex::new(
                {
                    let mut e = EGraph::default();
                    println!("{}",type_defs);
                    e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
                    e
                },
            ),
            map: DashMap::default(),
            latest_map: DashMap::default(),
        }
    }
    pub fn new() -> Self{
        Self::new_with_type_defs(collect_type_defs())
    }
    pub fn interpret(&self,s:String){
        let mut egraph = self.egraph.lock().unwrap();
        egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    pub fn to_dot(&self,file_name:PathBuf){
        let egraph = self.egraph.lock().unwrap();
        let serialized = egraph.serialize(SerializeConfig::default());
        let dot_path = file_name.with_extension("dot");
        serialized
            .to_dot_file(dot_path.clone())
            .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    }
    // collect all ancestors of cur_sym, without cur_sym
    pub fn collect_symnode(&self, cur_sym:Sym, index_set:&mut IndexSet<Sym>){
        let sym_node = self.map.get(&cur_sym).unwrap();
        let v = sym_node.preds.clone();
        drop(sym_node);
        for pred in v{
            if index_set.contains(&pred) || self.map.get(&pred).unwrap().next.is_some(){
                // do nothing
            }else {
                index_set.insert(pred.clone());
                self.collect_symnode(pred,index_set)
            }
        }
    }
    /// start nodes is asserted to be zero input degree 
    pub fn topo_sort(&self, starts : IndexSet<Sym> ,index_set:&IndexSet<Sym>)-> Vec<Sym>{
        let map = &self.map;
        // init in degrees and out degrees 
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        ins.resize(index_set.len(), 0);
        outs.resize(index_set.len(), 0);
        for (i,(in_degree,out_degree)) in ins.iter_mut().zip(outs.iter_mut()).enumerate(){
            let sym = index_set[i];
            let node = map.get(&sym).unwrap();
            *in_degree = RxNoVT::degree_in_subgraph(node.preds().into_iter().map(|x|*x), index_set);
            *out_degree = RxNoVT::degree_in_subgraph(node.succs().into_iter(), index_set);
        }
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

    pub fn map_latest(&self, sym:Sym) -> Sym{
        let mut cur = sym;
        while let Some(key ) = self.latest_map.get(&cur){
            cur = *key
        }
        cur
    }
    
}


unsafe impl Send for RxNoVT{ }
unsafe impl Sync for RxNoVT{ }
// MARK: Receiver
impl Rx for RxNoVT{
    fn receive(&self, received:String) {
        println!("{}",received);
        self.interpret(received);
    }

    fn add_symnode(&self, mut symnode:SymbolNode){
        self.receive(symnode.egglog.to_egglog());
        let sym = symnode.cur_sym();
        for node in symnode.succs_mut(){
            *node = self.map_latest(*node);
            self.map.get_mut(node)
                .unwrap_or_else(||panic!("node {} not found", node.as_str()))
                .preds.push(sym);
        }
        // println!("{:?}",self.map);
        self.map.insert(symnode.cur_sym(), symnode);
    }

    /// update all predecessor recursively in guest and send updated term by egglog repr to host
    /// when you update the node
    /// for non version control mode, update_symnode will update &mut old sym to latest
    fn update_symnode(&self, old:&mut Sym, mut updated_symnode:SymbolNode){
        let mut index_set = IndexSet::default();
        *old =self.map_latest(*old);
        // collect all syms that will change
        self.collect_symnode(*old, &mut index_set);
        let mut old_node = self.map.get_mut(old).unwrap();
        // chain old version and new version
        old_node.next = Some(updated_symnode.egglog.cur_sym());
        updated_symnode.preds = old_node.preds.clone();
        drop(old_node);
        let mut new_syms = vec![];
        // update all succs
        for &old_sym in index_set.iter(){
            let (_,mut sym_node) = self.map.remove(&old_sym).unwrap();
            let new_sym = sym_node.next_sym();
            self.latest_map.insert(old_sym, new_sym);

            new_syms.push(new_sym);
            self.map.insert(new_sym, sym_node);
        }
        index_set.insert(*old);
        let new_sym = updated_symnode.cur_sym();
        new_syms.push(updated_symnode.cur_sym());
        self.map.insert(updated_symnode.cur_sym() ,updated_symnode);
        // update all preds
        for &new_sym in &new_syms{
            let mut sym_node = self.map.get_mut(&new_sym).unwrap();
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
        let topo = self.topo_sort(
            IndexSet::from_iter(Some(new_sym).into_iter()),
            &IndexSet::from_iter(new_syms.into_iter()));
        for new_sym in topo{
            s += self.map.get(&new_sym).unwrap().egglog.to_egglog().as_str();
        }
        self.receive(s);
        *old = new_sym;
    }
    
    fn update_symnodes(&self, _start_iter:impl Iterator<Item=(Sym,SymbolNode)>) {
        todo!()
    }

    // fn rx() -> &'static impl LetStmtRxInner {
    //     static INSTANCE: OnceLock<RxInner> = OnceLock::new();
    //     INSTANCE.get_or_init(||{
    //         Self {
    //             egraph: Mutex::new(
    //                 {
    //                     let mut e = EGraph::default();
    //                     let type_defs = collect_type_defs();
    //                     println!("{}",type_defs);
    //                     e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
    //                     e
    //                 },
    //             ),
    //             map: DashMap::default(),
    //         }
    //     })
    // }
}
