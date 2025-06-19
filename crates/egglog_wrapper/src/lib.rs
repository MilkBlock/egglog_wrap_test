pub use derive_more;
pub mod tx;
pub mod tx_rx_vt;
pub mod tx_vt;
pub mod wrap;

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

/// macro to quickly define a Transimitter with no version control
#[macro_export]
macro_rules! basic_tx_no_vt {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx::TxNoVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx::TxNoVT;
            fn sgl() -> &'static egglog_wrapper::tx::TxNoVT {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            tx: egglog_wrapper::tx::TxNoVT::new(),
                        }
                    })
                    .tx
            }
        }
    };
}
/// macro to quickly define a Transimitter with version control
#[macro_export]
macro_rules! basic_tx_vt {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx_vt::TxVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx_vt::TxVT;
            fn sgl() -> &'static egglog_wrapper::tx_vt::TxVT {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            tx: egglog_wrapper::tx_vt::TxVT::new(),
                        }
                    })
                    .tx
            }
        }
    };
}
/// macro to quickly define a minimal Transimitter 
#[macro_export]
macro_rules! basic_tx_minimal {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx_minimal::TxMinimal,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx_minimal::TxMinimal;
            fn sgl() -> &'static egglog_wrapper::tx_minimal::TxMinimal {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            tx: egglog_wrapper::tx_minimal::TxMinimal::new(),
                        }
                    })
                    .tx
            }
        }
    };
}

#[macro_export]
macro_rules! basic_tx_rx_vt {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx_rx_vt::TxRxVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx_rx_vt::TxRxVT;
            fn sgl() -> &'static egglog_wrapper::tx_rx_vt::TxRxVT {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            tx: egglog_wrapper::tx_rx_vt::TxRxVT::new(),
                        }
                    })
                    .tx
            }
        }
    };
}
