use crate::{
    collect_type_defs,
    wrap::{EgglogNode, Rx, RxCommit, Sym, SymbolNode, VersionCtl},
};
use dashmap::DashMap;
use derive_more::Display;
use egglog::{util::{IndexMap, IndexSet}, EGraph, SerializeConfig};
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

#[derive(Default)]
pub struct RxVT {
    egraph: Mutex<EGraph>,
    map: DashMap<Sym, SymbolNode>,
    /// used to store staged node of committed nodes (Not only the currently latest node but also nodes of old versions)
    staged_set_map : DashMap<Sym, Box<dyn EgglogNode>>,
    staged_new_map : Mutex<IndexMap<Sym,Box<dyn EgglogNode>>>,
    checkpoints: Mutex<Vec<CommitCheckPoint>>,
}

#[derive(Debug, Display)]
#[display("CheckPoint = {{ root:{}, staged_set_nodes:{:#?}, staged_new_nodes:{:#?}}}",
    committed_node_root,
    staged_set_nodes.iter().map(|x|x.as_str()).collect::<Vec<_>>(),
    staged_new_nodes.iter().map(|x|x.as_str()).collect::<Vec<_>>())
]
pub struct CommitCheckPoint {
    committed_node_root: Sym,
    staged_set_nodes: Vec<Sym>,
    staged_new_nodes: Vec<Sym>,
}

pub enum TopoDirection {
    Up,
    Down,
}
/// Rx with version ctl feature
impl RxVT {
    pub fn interpret(&self, s: String) {
        let mut egraph = self.egraph.lock().unwrap();
        egraph.parse_and_run_program(None, s.as_str()).unwrap();
    }
    pub fn to_dot(&self, file_name: PathBuf) {
        let egraph = self.egraph.lock().unwrap();
        let serialized = egraph.serialize(SerializeConfig::default());
        let dot_path = file_name.with_extension("dot");
        serialized
            .to_dot_file(dot_path.clone())
            .unwrap_or_else(|_| panic!("Failed to write dot file to {dot_path:?}"));
    }
    // collect all lastest ancestors of cur_sym, without cur_sym
    pub fn collect_latest_ancestors(&self, cur_sym: Sym, index_set: &mut IndexSet<Sym>) {
        let sym_node = self.map.get(&cur_sym).unwrap();
        let v = sym_node.preds.clone();
        drop(sym_node);
        for pred in v {
            // if pred has been accessed or it's not the lastest version
            if index_set.contains(&pred) || self.map.get(&pred).unwrap().next.is_some() {
                // do nothing
            } else {
                index_set.insert(pred.clone());
                self.collect_latest_ancestors(pred, index_set)
            }
        }
    }
    // collect all descendants of cur_sym, without cur_sym
    pub fn collect_descendants(&self, cur_sym: Sym, index_set: &mut IndexSet<Sym>) {
        let succs = self.staged_set_map.get(&cur_sym).map(|x|x.succs()).unwrap_or(self.map.get(&cur_sym).unwrap().succs()) ;
        for succ in succs {
            if index_set.contains(&succ) || self.map.get(&succ).unwrap().next.is_some() {
                // do nothing this succ node has been accessed
            } else {
                index_set.insert(succ.clone());
                self.collect_descendants(succ, index_set)
            }
        }
    }
    /// topo all input nodes
    pub fn topo_sort(&self, index_set: &IndexSet<Sym>, direction: TopoDirection) -> Vec<Sym> {
        // init in degrees and out degrees
        let mut ins = Vec::new();
        let mut outs = Vec::new();
        ins.resize(index_set.len(), 0);
        outs.resize(index_set.len(), 0);
        for (i, (in_degree, out_degree)) in ins.iter_mut().zip(outs.iter_mut()).enumerate() {
            let sym = index_set[i];
            let node = self.map.get(&sym).unwrap();
            *in_degree = RxVT::degree_in_subgraph(node.preds().into_iter().map(|x| *x), index_set);
            *out_degree = RxVT::degree_in_subgraph(node.succs().into_iter(), index_set);
        }
        let (mut _ins, mut outs) = match direction {
            TopoDirection::Up => (ins, outs),
            TopoDirection::Down => (outs, ins),
        };
        let mut rst = Vec::new();
        let mut wait_for_release = Vec::new();
        // start node should not have any out edges in subgraph
        for (idx, _value) in outs.iter().enumerate() {
            if 0 == outs[idx] {
                wait_for_release.push(index_set[idx]);
            }
        }
        while !wait_for_release.is_empty() {
            let popped = wait_for_release.pop().unwrap();
            println!("popped is {} preds:{:?}",popped, &self.map.get(&popped).unwrap().preds);
            for target in &self.map.get(&popped).unwrap().preds {
                let idx = index_set.get_index_of(target).unwrap();
                outs[idx] -= 1;
                if outs[idx] == 0 {
                    println!("{} found to be 0", target);
                    wait_for_release.push(*target);
                }
            }
            rst.push(popped);
        }
        println!("{:?}",rst);
        rst
    }
    /// calculate the edges in the subgraph
    pub fn degree_in_subgraph(nodes: impl Iterator<Item = Sym>, index_set: &IndexSet<Sym>) -> u32 {
        nodes.fold(0, |acc, item| {
            if index_set.contains(&item) {
                acc + 1
            } else {
                acc
            }
        })
    }
    pub fn new_with_type_defs(type_defs: String) -> Self {
        Self {
            egraph: Mutex::new({
                let mut e = EGraph::default();
                println!("{}", type_defs);
                e.parse_and_run_program(None, type_defs.as_ref()).unwrap();
                e
            }),
            ..Self::default()
        }
    }
    pub fn new() -> Self {
        Self::new_with_type_defs(collect_type_defs())
    }
    fn add_symnode(&self, mut symnode: SymbolNode, auto_latest:bool) {
        let sym = symnode.cur_sym();
        for node in symnode.succs_mut() {
            println!("succ is {}",node);
            let latest =if auto_latest{
                &self.locate_latest(*node)
            }else {
                &*node
            };
            self.map
                .get_mut(node)
                .unwrap_or_else(|| panic!("node {} not found", latest.as_str()))
                .preds
                .push(sym);
            *node = *latest;
        }
        self.map.insert(symnode.cur_sym(), symnode);
    }

    /// update all ancestors recursively in guest and send updated term by egglog string repr to host
    /// when you update the node
    /// This is version control mode impl so we will not change &mut old sym.
    /// return all symnodes created
    fn update_symnodes(&self, staged_latest_syms_and_staged_nodes: Vec<(Sym, Box<dyn EgglogNode>)>) -> IndexSet<Sym>{
        // collect all ancestors that need copy
        let mut latest_ancestors = IndexSet::default();
        for (latest_sym,_) in &staged_latest_syms_and_staged_nodes{
            println!("collect ancestors of {:?}",latest_sym);
            self.collect_latest_ancestors(*latest_sym, &mut latest_ancestors);
        }
        
        let mut staged_latest_sym_map = IndexMap::default();
        // here we insert all staged_latest_sym because latest_ancestors do may not include all of them
        for (staged_latest_sym, staged_node) in staged_latest_syms_and_staged_nodes{
            latest_ancestors.insert(staged_latest_sym);
            staged_latest_sym_map.insert(staged_latest_sym, staged_node);
        }
        println!("all latest_ancestors {:?}", latest_ancestors);

        let mut next_syms = IndexSet::default();
        for latest in latest_ancestors{
            let mut latest_node = self.map.get_mut(&latest).unwrap();
            let next_sym = latest_node.next_sym();
            let next_latest_node = latest_node.clone();
            // chain old version and new version
            latest_node.next = Some(next_sym);
            drop(latest_node);
            next_syms.insert(next_sym);
            if !staged_latest_sym_map.contains_key(&latest){
                self.map.insert(next_sym, next_latest_node);
            }else{
                let mut staged_node = staged_latest_sym_map.get(&latest).unwrap().clone_dyn();
                *staged_node.cur_sym_mut()  = next_sym;

                let mut staged_sym_node = SymbolNode::new(staged_node);
                staged_sym_node.preds = self.map.get(&latest).unwrap().preds.clone();
                self.map.insert(next_sym, staged_sym_node);
            }
        }

        // update all preds
        let mut succ_preds_map = HashMap::new();
        for &next_sym in &next_syms {
            let sym_node = self.map.get(&next_sym).unwrap();
            for &sym in  sym_node.preds(){
                let latest_sym = self.locate_latest(sym);
                if sym != latest_sym && !succ_preds_map.contains_key(&latest_sym){
                    succ_preds_map.insert(sym, latest_sym);
                }
            }
            for sym in sym_node.succs() {
                let latest_sym = self.locate_latest(sym);
                if sym != latest_sym && !succ_preds_map.contains_key(&latest_sym){
                    succ_preds_map.insert(sym, latest_sym);
                }
            }
        }

        for &next_sym in &next_syms {
            let mut sym_node = self.map.get_mut(&next_sym).unwrap();
            for sym in sym_node.preds_mut(){
                if let Some(found) = succ_preds_map.get(sym){
                    *sym = *found;
                }
            }
            for sym in sym_node.succs_mut(){
                if let Some(found) = succ_preds_map.get(sym){
                    *sym = *found;
                }
            }
        }
        println!("{:#?}",self.map);

        next_syms
    }
}

unsafe impl Send for RxVT {}
unsafe impl Sync for RxVT {}
impl VersionCtl for RxVT {
    /// locate the lastest version of the symbol
    fn locate_latest(&self, old: Sym) -> Sym {
        let map = &self.map;
        let mut cur = old;
        while let Some(newer) = map.get(&cur).unwrap().next {
            cur = newer;
        }
        cur
    }

    // locate next version
    fn locate_next(&self, old: Sym) -> Sym {
        let map = &self.map;
        let mut cur = old;
        if let Some(newer) = map.get(&cur).unwrap().next {
            cur = newer;
        } else {
            // do nothing because current version is the latest
        }
        cur
    }

    fn set_latest(&self, node: &mut Sym) {
        *node = self.locate_latest(*node);
    }

    fn set_next(&self, node: &mut Sym) {
        *node = self.locate_next(*node);
    }
}

// MARK: Receiver
impl Rx for RxVT {
    fn receive(&self, received: String) {
        println!("{}", received);
        self.interpret(received);
    }

    fn on_new(&self, node: &(impl EgglogNode + 'static)) {
        self.staged_new_map.lock().unwrap().insert(node.cur_sym(), node.clone_dyn());
    }

    fn on_set(&self, _node: &mut (impl EgglogNode + 'static)) {
        // do nothing, this operation has been delayed to commit
    }
}

impl RxCommit for RxVT {
    /// commit behavior:
    /// 1. commit all descendants (if you also call set fn on subnodes they will also be committed)
    /// 2. commit basing the latest version of the working graph (working graph record all versions)
    /// 3. if RxCommit is implemented you can only change egraph by commit things. It's lazy.
    fn on_commit<T: EgglogNode + 'static>(&self, node: &T) {
        let check_point = CommitCheckPoint {
            committed_node_root: node.cur_sym(),
            staged_set_nodes: self.staged_set_map.iter().map(|a| *a.key()).collect(),
            staged_new_nodes: self.staged_new_map.lock().unwrap().iter().map(|a| *a.0).collect(),
        };
        println!("{}",check_point);
        self.checkpoints.lock().unwrap().push(check_point);


        // process new nodes
        let mut news = self.staged_new_map.lock().unwrap();
        let mut backup_staged_new_syms = IndexSet::default();
        let len = news.len();
        for (new, new_node) in news.drain(0..len){
            self.add_symnode(SymbolNode::new(new_node.clone_dyn()),false);
            backup_staged_new_syms.insert(new);
        }
        // send egglog string to egraph
        backup_staged_new_syms.into_iter()
            .for_each(|sym| self.receive(self.map.get(&sym).unwrap().egglog.to_egglog()));

        let all_staged = IndexSet::from_iter(self.staged_set_map.iter().map(|a| *a.key()));
        // // check any absent node
        // let mut panic_list = IndexSet::default();
        // for &sym in &all_staged{
        //     if !self.map.contains_key(&sym){
        //         panic_list.insert(sym);
        //     }
        // }
        // if panic_list.len()>0 {panic!("node {:?} not exist",panic_list )};

        let mut descendants = IndexSet::default();
        self.collect_descendants(node.cur_sym(), &mut descendants);
        descendants.insert(node.cur_sym());


        let staged_descendants_old = descendants.intersection(&all_staged).collect::<Vec<_>>();
        let staged_descendants_latest = staged_descendants_old.iter().map(|x| self.locate_latest(**x)).collect::<Vec<_>>();

        let iter_impl = 
            staged_descendants_latest.iter().cloned().zip(
                staged_descendants_old.iter().map(|x|self.staged_set_map.remove(*x).unwrap().1));
        let created = self.update_symnodes(iter_impl.collect());
        println!("created {:#?}",created);

        println!("nodes to topo:{:?}",created);
        self.topo_sort(&created, TopoDirection::Up)
            .into_iter()
            .for_each(
                |sym| self.receive(self.map.get(&sym).unwrap().egglog.to_egglog())
            );

    }
    
    fn on_stage<T: EgglogNode + ?Sized>(&self, node: &T) {
        self.staged_set_map.insert(node.cur_sym(), node.clone_dyn());
    }
}
