#[macro_export]
macro_rules! basic_tx_no_vt {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx::TxNoVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx::TxNoVT;
            fn tx() -> &'static egglog_wrapper::tx::TxNoVT {
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
#[macro_export]
macro_rules! basic_tx_vt {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx_vt::TxVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx_vt::TxVT;
            fn tx() -> &'static egglog_wrapper::tx_vt::TxVT {
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
#[macro_export]
macro_rules! basic_tx_minimal {
    ($name:ident) => {
        struct $name {
            tx: egglog_wrapper::tx_minimal::TxMinimal,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::tx_minimal::TxMinimal;
            fn tx() -> &'static egglog_wrapper::tx_minimal::TxMinimal {
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
