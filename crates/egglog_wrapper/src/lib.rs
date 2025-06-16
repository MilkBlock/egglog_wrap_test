pub use derive_more;
pub mod macros;
pub mod tx;
pub mod tx_vt;
pub mod wrap;
pub use smallvec;

use crate::wrap::{FuncSort, TySort};
pub mod tx_minimal;

pub fn collect_string_type_defs() -> String {
    let mut ty_defs = "".to_owned();
    for sort in inventory::iter::<TySort> {
        ty_defs.push_str(sort.0);
    }
    let mut func_defs = "".to_owned();
    for sort in inventory::iter::<FuncSort> {
        func_defs.push_str(sort.0);
    }
    format!("(datatype* {} ) {}", ty_defs, func_defs)
}
