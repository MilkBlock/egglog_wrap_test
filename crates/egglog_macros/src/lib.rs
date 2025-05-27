extern crate proc_macro;
use core::panic;

use heck::{ ToSnakeCase};
use helper::*;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput};
mod helper;

/// generate `egglog` language from `rust native structure`   
/// 
/// # Example:  
///     
/// ```
/// #[allow(unused)]
/// #[derive(Debug, Clone, ToEgglog)]
/// enum Duration {
///     DurationBySecs {
///         seconds: f64,
///     },
///     DurationByMili {
///         milliseconds: f64,
///     },
/// }
/// ```
/// is transformed to 
/// 
/// 
/// ```
/// #[derive(Debug, Clone, ::derive_more::Deref)]
/// pub struct DurationNode {
///     ty: _DurationNode,
///     #[deref]
///     sym: DurationSym,
/// }
/// 
/// fn to_egglog(&self) -> String {
///     match &self.ty {
///         _DurationNode::DurationBySecs { seconds } => {
///             format!("(let {} (DurationBySecs  {:.3}))", self.sym, seconds)
///         }
///         _DurationNode::DurationByMili { milliseconds } => {
///             format!("(let {} (DurationByMili  {:.3}))", self.sym, milliseconds)
///         }
///     }
/// }
/// impl crate::ToEgglog for Duration {
///     const SORT_DEF: crate::Sort =
///         crate::Sort(stringify!((Duration(DurationBySecs f64)(DurationByMili f64))));
/// }
/// ```
/// so that you can directly use to_egglog to generate let statement in eggglog
/// 
/// also there is a type def statement generated and specialized new function
/// 
/// 
#[proc_macro_derive(ToEgglog)]
pub fn derive_to_egglog(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let egglog_wrapper_path = egglog_wrapper_path();
    let inventory_path = inventory_wrapper_path();

    let type_def_expanded = match &input.data {
        Data::Enum(data_enum) => {
            let variants_egglog = data_enum.variants.iter().map(|variant|{
                let tys = variants_to_tys(&variant);
                let variant_name = &variant.ident;
                quote!{  (#variant_name #(#tys )* )}
            }).collect::<Vec<_>>();
            let expanded = quote! {
                impl #egglog_wrapper_path::ToEgglog for #name {
                    const SORT_DEF: #egglog_wrapper_path::Sort= 
                        #egglog_wrapper_path::Sort(stringify!(
                            (#name
                                #(#variants_egglog)*
                            )
                        ));
                }
                #inventory_path::submit!{#name::SORT_DEF}
            };
            expanded
        },
        Data::Struct(data_struct) => {
            // process (sort A (Vec M))  such things ..
            let f = data_struct.fields.iter().nth(0).expect("Struct should only have one Vec field");
            let first_generic = get_first_generic(&f.ty);
            if is_vec_type(&f.ty){
                let vec_expanded = quote! {
                    impl #egglog_wrapper_path::ToEgglog for #name {
                        const SORT_DEF: #egglog_wrapper_path::Sort= 
                            #egglog_wrapper_path::Sort(stringify!(
                                (sort #name (Vec #first_generic))
                            ));
                    }
                    #inventory_path::submit!{#name::SORT_DEF}
                };
                vec_expanded
            }else {
                panic!("only support Vec for struct")
            }
        },
        _ => panic!("only support enum"),
    };
    let struct_def_expanded = match &input.data{
        Data::Struct(data_struct) => {
            // process (sort A (Vec M))  such things ..
            let f = data_struct.fields.iter().nth(0).expect("Struct should only have one Vec field");
            let field_name = &f.ident.as_ref().unwrap();
            let first_generic = get_first_generic(&f.ty);
            // let field_sym_ty = get_sym_type(first_generic);
            let field_node_ty = match first_generic.to_token_stream().to_string().as_str(){
                x if PANIC_TY_LIST.contains(&x) => {
                    panic!("{} not supported",x)
                }
                x if EGGLOG_BASIC_TY_LIST.contains(&x) => {
                    first_generic.to_token_stream()
                }
                _=>{postfix_type(&first_generic,"Node")}
            };
            let field_sym_ty = match first_generic.to_token_stream().to_string().as_str(){
                x if PANIC_TY_LIST.contains(&x) => {
                    panic!("{} not supported",x)
                }
                x if EGGLOG_BASIC_TY_LIST.contains(&x) => {
                    first_generic.to_token_stream()
                }
                _ =>{postfix_type(&first_generic,"Sym")}
            };
            // let field_ref_node_ty = match first_generic.to_token_stream().to_string().as_str(){
            //     x if PANIC_TY_LIST.contains(&x) => {
            //         panic!("{} not supported",x)
            //     }
            //     x if EGGLOG_BASIC_TY_LIST.contains(&x) => {
            //         first_generic.to_token_stream()
            //     }
            //     _=>{get_ref_type(&first_generic)}
            // };
            let name_sym = format_ident!("{}Sym",name);
            let name_node = format_ident!("{}Node",name);
            let _name_node = format_ident!("_{}Node",name);
            let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            let name_counter = format_ident!("{}_COUNTER",name.to_string().to_uppercase());
            let inner_sym = format_ident!("sym");
            let derive_more_path  = derive_more_path();
            if is_vec_type(&f.ty){
                let vec_expanded = quote! {
                    #[derive(Debug,Clone,#derive_more_path::Deref)]
                    pub struct #name_sym (
                        symbol_table::GlobalSymbol
                    );
                    #[derive(Debug,Clone,#derive_more_path::Deref)]
                    pub struct #name_node {
                        #field_name : Vec<#field_sym_ty>,
                        #[deref]
                        #inner_sym : #name_sym
                    }
                    const _:() = {
                        impl #name_node {
                            pub fn new(#field_name:Vec<&#field_node_ty>) -> #name_node{
                                let ___sym = format!("{}{}",stringify!(#name_snakecase),inc()).into();
                                let v = v.into_iter().map(|r| r.sym).collect::<Vec<_>>();
                                #name_node{ #field_name ,#inner_sym: #name_sym(___sym)}
                            }
                            fn to_egglog(&self) -> String{
                                format!("(vec-of {})",self.#field_name.iter().fold("".to_owned(), |s,item| s+item.as_str()+" " ))
                            }
                        }
                        static #name_counter: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                        pub fn get_counter() -> u32 {
                            #name_counter.load(std::sync::atomic::Ordering::Acquire)
                        }
                        /// 递增计数器
                        pub fn inc() -> u32{
                            #name_counter.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
                        }
                    };
                    impl From<#name_sym> for &str{
                        fn from(value: #name_sym)-> Self {
                            (value.0).into()
                        }
                    }
                    impl Copy for #name_sym{}
                    impl std::fmt::Display for #name_sym{
                        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            f.write_str(self.as_str())
                        }
                    }
                };
                vec_expanded
            }else {
                panic!("only support Vec for struct")
            }
        },
        // transform   enum A{ A1{a:i32} , A2 {b:B}} 
        // into struct enum ANode { A1{a:I32Sym} A2{b:BSym} }
        // so here we build 2 struct 
        // ANode to store nodes
        // ASym to store symbols 
        // we don't need to directly use ASym because they can be 
        // a = A::new(xxx ,xx b)  new 的一瞬间就注册怎么样 
        // 不好不好，最好是在 .send 那个时候一次性输出 
        // 这样 new 出来的可以 defref 成 ASym 出来的是 ANode 
        Data::Enum(data_enum) => {
            let name_node = format_ident!("{}Node",name);
            let _name_node = format_ident!("_{}Node",name);
            let name_sym = format_ident!("{}Sym",name);
            let name_counter = format_ident!("{}_COUNTER",name.to_string().to_uppercase());
            let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            let derive_more_path  = derive_more_path();

            let variants_def_of_node_with_syms = data_enum.variants.iter().map(|variant|{
                let types_and_idents = variants_to_sym_list(variant);
                let variant_name = &variant.ident;
                quote! {#variant_name {#( #types_and_idents ),*  }}
            }).collect::<Vec<_>>();

            let match_arms = data_enum.variants.iter().map(|variant|{
                let variant_idents = variants_to_field_ident(variant).collect::<Vec<_>>();
                let variant_name = &variant.ident;
                let s = " {:.3}".repeat(variant_idents.len());
                let format_str = format!("(let {{}} ({} {}))",variant_name,s);
                quote! {#_name_node::#variant_name {#( #variant_idents ),*  } => {
                    format!(#format_str ,self.sym, #(#variant_idents),*)
                }}
            });

            let fns = data_enum.variants.iter().map(|variant|{
                let ref_node_list = variants_to_ref_node_list(&variant);
                let field_idents = variants_to_assign_node_field_list(&variant);
                let variant_name = &variant.ident;
                let fn_name = format_ident!("new_{}",variant_name.to_string().to_snake_case());
                
                quote! { pub fn #fn_name<T:LetStmtRx>(#(#ref_node_list),*) -> #name_node{
                    let ty = #_name_node::#variant_name {#(#field_idents),*  };
                    let sym = #name_sym(format!("{}{}",stringify!(#name_snakecase),inc()).into());
                    let node = #name_node { ty,sym};
                    // T::receive(node.to_egglog()));
                    node
                }} 
            });

            // let match_arm_fields = variants_to_field_ident(&variant);
            let expanded = quote! {
                #[derive(Debug,Clone,#derive_more_path::Deref)]
                pub struct #name_sym (
                    symbol_table::GlobalSymbol
                );
                #[derive(Debug,Clone)]
                enum #_name_node {
                    #(#variants_def_of_node_with_syms),*
                }
                #[derive(Debug,Clone,#derive_more_path::Deref)]
                pub struct #name_node{
                    ty :#_name_node,
                    #[deref]
                    sym : #name_sym
                }
                const _:() = {
                    impl #name_node {
                        #(#fns)*
                        pub fn to_egglog(&self) -> String{
                            match &self.ty{
                                #(#match_arms),*
                            }
                        }
                    }
                    static #name_counter: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                    pub fn get_counter() -> u32 {
                        #name_counter.load(std::sync::atomic::Ordering::Acquire)
                    }
                    /// get next count
                    pub fn inc() -> u32{
                        #name_counter.fetch_add(1, std::sync::atomic::Ordering::AcqRel)
                    }
                };
                impl From<#name_sym> for &str{
                    fn from(value: #name_sym) -> Self {
                        (value.0).as_str()
                    }
                }
                impl Copy for #name_sym{ }
                impl std::fmt::Display for #name_sym{
                    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                        f.write_str(self.as_str())
                    }
                }
            };
            expanded
        },
        Data::Union(_) => todo!(),
    };

    TokenStream::from(quote!{#type_def_expanded #struct_def_expanded })
}
