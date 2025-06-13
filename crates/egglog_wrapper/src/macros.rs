#[macro_export]
macro_rules! basic_rx_no_vt {
    ($name:ident) => {
        struct $name {
            rx: egglog_wrapper::rx::RxNoVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::rx::RxNoVT;
            fn rx() -> &'static egglog_wrapper::rx::RxNoVT {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            rx: egglog_wrapper::rx::RxNoVT::new(),
                        }
                    })
                    .rx
            }
        }
    };
}
#[macro_export]
macro_rules! basic_rx_vt {
    ($name:ident) => {
        struct $name {
            rx: egglog_wrapper::rx_vt::RxVT,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::rx_vt::RxVT;
            fn rx() -> &'static egglog_wrapper::rx_vt::RxVT {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            rx: egglog_wrapper::rx_vt::RxVT::new(),
                        }
                    })
                    .rx
            }
        }
    };
}
#[macro_export]
macro_rules! basic_rx_minimal {
    ($name:ident) => {
        struct $name {
            rx: egglog_wrapper::rx_minimal::RxMinimal,
        }
        impl SingletonGetter for $name {
            type RetTy = egglog_wrapper::rx_minimal::RxMinimal;
            fn rx() -> &'static egglog_wrapper::rx_minimal::RxMinimal {
                static INSTANCE: std::sync::OnceLock<$name> = std::sync::OnceLock::new();
                &INSTANCE
                    .get_or_init(|| -> $name {
                        Self {
                            rx: egglog_wrapper::rx_minimal::RxMinimal::new(),
                        }
                    })
                    .rx
            }
        }
    };
}
