use crate::{
    collect_type_defs,
    wrap::{EgglogNode, Rx, RxCommit, Sym, SymbolNode, VersionCtl},
};
use dashmap::DashMap;
use egglog::{EGraph, SerializeConfig, util::IndexSet};
use std::{path::PathBuf, sync::Mutex};

pub struct RxVT {
    egraph: Mutex<EGraph>,
    map: DashMap<Sym, SymbolNode>,
    checkpoints: Mutex<Vec<CommitCheckPoint>>,
}

pub struct CommitCheckPoint {
    committed_node_root: Sym,
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
    // collect all ancestors of cur_sym, without cur_sym
    pub fn collect_ancestors(&self, cur_sym: Sym, index_set: &mut IndexSet<Sym>) {
        let sym_node = self.map.get(&cur_sym).unwrap();
        let v = sym_node.preds.clone();
        drop(sym_node);
        for pred in v {
            if index_set.contains(&pred) || self.map.get(&pred).unwrap().next.is_some() {
                // do nothing
            } else {
                index_set.insert(pred.clone());
                self.collect_ancestors(pred, index_set)
            }
        }
    }
    // collect all descendants of cur_sym, without cur_sym
    pub fn collect_descendants(&self, cur_sym: Sym, index_set: &mut IndexSet<Sym>) {
        let sym_node = self.map.get(&cur_sym).unwrap();
        let v = sym_node.succs().clone();
        drop(sym_node);
        for succ in v {
            if index_set.contains(&succ) || self.map.get(&succ).unwrap().next.is_some() {
                // do nothing
            } else {
                index_set.insert(succ.clone());
                self.collect_descendants(succ, index_set)
            }
        }
    }
    /// start nodes is asserted to be zero input degree
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
            for target in &self.map.get(&popped).unwrap().preds {
                let idx = index_set.get_index_of(target).unwrap();
                outs[idx] -= 1;
                if outs[idx] == 0 {
                    wait_for_release.push(*target);
                }
            }
            rst.push(popped);
        }
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
            map: DashMap::default(),
            checkpoints: Mutex::new(Vec::new()),
        }
    }
    pub fn new() -> Self {
        Self::new_with_type_defs(collect_type_defs())
    }
    fn add_symnode(&self, mut symnode: SymbolNode) {
        let sym = symnode.cur_sym();
        for node in symnode.succs_mut() {
            let latest = &self.locate_latest(*node);
            self.map
                .get_mut(latest)
                .unwrap_or_else(|| panic!("node {} not found", latest.as_str()))
                .preds
                .push(sym);
        }
        self.map.insert(symnode.cur_sym(), symnode);
    }

    fn update_symnode(&self, old: Sym, mut updated_symnode: SymbolNode) -> Sym {
        let mut index_set = IndexSet::default();

        let lastest = self.locate_latest(old);

        // collect all syms that will change
        self.collect_ancestors(lastest, &mut index_set);
        let mut latest_node = self.map.get_mut(&lastest).unwrap();
        // chain old version and new version
        latest_node.next = Some(updated_symnode.egglog.cur_sym());
        updated_symnode.preds = latest_node.preds.clone();
        drop(latest_node);
        let mut new_syms = vec![];
        // update all succs
        for &old_sym in index_set.iter() {
            let mut sym_node = self.map.get(&old_sym).unwrap().clone();
            let new_sym = sym_node.next_sym();

            // chain old version and new version
            self.map.get_mut(&old_sym).unwrap().next = Some(new_sym);

            new_syms.push(new_sym);
            self.map.insert(new_sym, sym_node);
        }
        index_set.insert(lastest);
        let next_latest_sym = updated_symnode.cur_sym();
        new_syms.push(next_latest_sym);
        self.map.insert(updated_symnode.cur_sym(), updated_symnode);
        // update all preds
        for &new_sym in &new_syms {
            let mut sym_node = self.map.get_mut(&new_sym).unwrap();
            for sym in sym_node.preds_mut() {
                if let Some(idx) = index_set.get_index_of(&*sym) {
                    *sym = new_syms[idx];
                }
            }
            for sym in sym_node.succs_mut() {
                if let Some(idx) = index_set.get_index_of(&*sym) {
                    *sym = new_syms[idx];
                }
            }
        }
        let mut s = "".to_owned();
        let topo = self.topo_sort(
            &IndexSet::from_iter(new_syms.into_iter()),
            TopoDirection::Up,
        );
        for new_sym in topo {
            s += self.map.get(&new_sym).unwrap().egglog.to_egglog().as_str();
        }
        self.receive(s);
        next_latest_sym
    }

    /// update all ancestors recursively in guest and send updated term by egglog string repr to host
    /// when you update the node
    /// This is version control mode impl so we will not change &mut old sym.
    fn update_symnodes(&self, _start_iter: impl Iterator<Item = (Sym, SymbolNode)>) {}
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

    fn on_new(&self, symnode: SymbolNode) {
        self.add_symnode(symnode);
    }

    fn on_set(&self, old: &mut Sym, symnode: SymbolNode) {
        // do nothing, this operation has been delayed to commit
    }
}

impl RxCommit for RxVT {
    /// commit behavior:
    /// 1. commit all subnodes (if you also call set fn on subnodes they will also be committed)
    /// 2. commit basing the latest version of the working graph (working graph record all versions)
    /// 3. if RxCommit is implemented you can only change egraph by commit things. It's lazy.
    fn on_commit<T: EgglogNode + Into<SymbolNode> + Clone + 'static>(&self, node: &T) {
        self.checkpoints.lock().unwrap().push(CommitCheckPoint {
            committed_node_root: node.cur_sym(),
        });
        self.update_symnode(node.cur_sym(), node.clone().into());
        let mut starts = IndexSet::default();
        starts.insert(node.cur_sym());
        let mut nodes_to_topo = IndexSet::default();
        self.collect_descendants(node.cur_sym(), &mut nodes_to_topo);
        nodes_to_topo.insert(node.cur_sym());
        self.topo_sort(&nodes_to_topo, TopoDirection::Up)
            .into_iter()
            .for_each(|sym| self.receive(self.map.get(&sym).unwrap().egglog.to_egglog()));
    }
}
