extern crate proc_macro;
use core::panic;

use heck::{ ToSnakeCase};
use helper::*;
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, Data, DeriveInput, Type};
mod helper;

/// generate `egglog` language from `rust native structure`   
/// 
/// # Example:  
///     
/// ```
/// #[allow(unused)]
/// #[derive(Debug, Clone, EgglogTy)]
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
/// impl crate::EgglogTy for Duration {
///     const SORT_DEF: crate::Sort =
///         crate::Sort(stringify!((Duration(DurationBySecs f64)(DurationByMili f64))));
/// }
/// ```
/// so that you can directly use to_egglog to generate let statement in eggglog
/// 
/// also there is a type def statement generated and specialized new function
/// 
/// 
#[proc_macro_attribute]
pub fn egglog_ty(attr: TokenStream, item:TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    let name_egglogty_impl = format_ident!("{}Ty",name);
    let egglog_wrapper_path = egglog_wrapper_path();
    let inventory_path = inventory_wrapper_path();

    let type_def_expanded = match &input.data {
        Data::Enum(data_enum) => {
            let variants_egglog = data_enum.variants.iter().map(|variant|{
                let tys = variant_to_tys(&variant);
                let variant_name = &variant.ident;
                quote!{  (#variant_name #(#tys )* )}
            }).collect::<Vec<_>>();
            let expanded = quote! {
                use #egglog_wrapper_path::wrap::*;
                impl EgglogTy for #name_egglogty_impl {
                    const TY_NAME:&'static str = stringify!(#name);
                    const SORT_DEF: #egglog_wrapper_path::wrap::Sort= 
                        #egglog_wrapper_path::Sort(stringify!(
                            (#name
                                #(#variants_egglog)*
                            )
                        ));
                }
                #inventory_path::submit!{#name_egglogty_impl::SORT_DEF}
            };
            expanded
        },
        Data::Struct(data_struct) => {
            // process (sort A (Vec M))  such things ..
            let f = data_struct.fields.iter().nth(0).expect("Struct should only have one Vec field");
            let first_generic = get_first_generic(&f.ty);
            if is_vec_type(&f.ty){
                let vec_expanded = quote! {
                    impl #egglog_wrapper_path::wrap::EgglogTy for #name_egglogty_impl {
                        const TY_NAME:&'static str = stringify!(#name);
                        const SORT_DEF: #egglog_wrapper_path::wrap::Sort= 
                            #egglog_wrapper_path::wrap::Sort(stringify!(
                                (sort #name (Vec #first_generic))
                            ));
                    }
                    #inventory_path::submit!{#name_egglogty_impl::SORT_DEF}
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
            let name_node_alias = format_ident!("{}NodeAlias",name);
            let name_node = format_ident!("{}",name);
            let name_inner = format_ident!("{}Inner",name);
            let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            let name_counter = format_ident!("{}_COUNTER",name.to_string().to_uppercase());
            let derive_more_path  = derive_more_path();
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
                _=>{
                    let first_generic_ident = match &first_generic{
                        syn::Type::Path(type_path) => {type_path.path.segments.last().expect("impossible").clone().ident},
                        _ => panic!("{} type should be simple path",first_generic.to_token_stream().to_string()),
                    };
                    let _first_generic_node_alias = format_ident!("{}NodeAlias",first_generic_ident);
                    // postfix_type(&first_generic,"Node",Some("T")
                    quote!(#_first_generic_node_alias<R, impl EgglogEnumSubTy>)
                }
            };
            let field_ty = match first_generic.to_token_stream().to_string().as_str(){
                x if PANIC_TY_LIST.contains(&x) => {
                    panic!("{} not supported",x)
                }
                x if EGGLOG_BASIC_TY_LIST.contains(&x) => {
                    first_generic.to_token_stream()
                }
                _ =>{
                    let first_generic = match &first_generic{
                        Type::Path(type_path) => {
                            type_path.path.segments.last().expect("impossible").clone().ident
                        },
                        _=> panic!("{} keep the type simple!",first_generic.to_token_stream().to_string())
                    };
                    format_ident!("{}Ty",first_generic).to_token_stream()
                }
            };
            if is_vec_type(&f.ty){
                let vec_expanded = quote! {
                    pub type #name_node_alias<R,S> = #egglog_wrapper_path::wrap::Node<#name_egglogty_impl,R,#name_inner,S>;

                    #[derive(Clone,Debug)]
                    pub struct #name_egglogty_impl;
                    #[derive(derive_more::Deref)]
                    pub struct #name_node<R: LetStmtRx, S: EgglogEnumSubTy=()> {
                        node:#name_node_alias<R,S>
                    }
                    pub struct #name_inner {
                        v:Vec<#egglog_wrapper_path::wrap::Sym<#field_ty>>
                    }
                    const _:() = {
                        use #egglog_wrapper_path::wrap::*;
                        impl wrap::NodeInner<#name_egglogty_impl> for #name_inner{}
                        use std::marker::PhantomData;
                        static #name_counter: TyCounter<#name_egglogty_impl> = TyCounter::new();
                        impl<R:LetStmtRx> #name_node<R,()> {
                            pub fn new(#field_name:Vec<&#field_node_ty>) -> #name_node<R,()>{
                                let v = v.into_iter().map(|r| r.sym).collect::<Vec<_>>();
                                let node = Node{ ty: #name_inner{v} , sym: #name_counter.next_sym(),p:PhantomData, s:PhantomData};
                                R::receive(node.to_egglog());
                                #name_node {node}
                            }
                        }
                        impl<R:LetStmtRx, S:EgglogEnumSubTy> ToEgglog for #name_node_alias<R,S>{
                            fn to_egglog(&self) -> String{
                                format!("(let {} (vec-of {}))",self.sym,self.ty.v.iter().fold("".to_owned(), |s,item| s+item.as_str()+" " ))
                            }
                        }
                    };
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
        
        // 不好不好，最好是在 .send 那个时候一次性输出 
        // 这样 new 出来的可以 defref 成 ASym 出来的是 ANode 
        Data::Enum(data_enum) => {
            let name_node_alias = format_ident!("{}NodeAlias",name);
            let name_node = format_ident!("{}",name);
            let _name_node = format_ident!("_{}",name);
            let name_inner = format_ident!("{}Inner",name);
            let name_counter = format_ident!("{}_COUNTER",name.to_string().to_uppercase());
            let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            let derive_more_path  = derive_more_path();

            let variants_def_of_node_with_syms = data_enum.variants.iter().map(|variant|{
                let types_and_idents = variants_to_sym_list(variant);
                let variant_name = &variant.ident;
                quote! {#variant_name {#( #types_and_idents ),*  }}
            }).collect::<Vec<_>>();

            let match_arms = data_enum.variants.iter().map(|variant|{
                let variant_idents = variant_to_field_ident(variant).collect::<Vec<_>>();
                let variant_name = &variant.ident;
                let s = " {:.3}".repeat(variant_idents.len());
                let format_str = format!("(let {{}} ({} {}))",variant_name,s);
                quote! {#name_inner::#variant_name {#( #variant_idents ),*  } => {
                    format!(#format_str ,self.sym, #(#variant_idents),*)
                }}
            });

            let fns = data_enum.variants.iter().map(|variant|{
                let ref_node_list = variant_to_ref_node_list(&variant,&name);
                let field_idents = variants_to_assign_node_field_list(&variant);
                let variant_name = &variant.ident;
                let fn_name = format_ident!("new_{}",variant_name.to_string().to_snake_case());
                
                quote! { 
                    pub fn #fn_name(#(#ref_node_list),*) -> #name_node<R,#variant_name>{
                        let ty = #name_inner::#variant_name {#(#field_idents),*  };
                        let node = Node { ty, sym: #name_counter.next_sym(), p:PhantomData, s:PhantomData::<#variant_name>};
                        R::receive(node.to_egglog());
                        #name_node {node}
                    }
                } 
            });
            let enum_sub_tys_def = data_enum.variants.iter().map(|variant|{
                let variant_name = &variant.ident;
                
                quote! { 
                    pub struct #variant_name;
                    impl EgglogEnumSubTy for #variant_name {
                        const TY_NAME:&'static str = stringify!(#variant_name);
                    }
                }
            });

            let set_fns = data_enum.variants.iter().map(|variant|{
                let ref_node_list = variant_to_ref_node_list(&variant,&name);
                let assign_node_field_list = variants_to_assign_node_field_list_with_out_prefixed_ident(&variant);
                let field_idents = variant_to_field_ident(variant).collect::<Vec<_>>();
                let variant_name = &variant.ident;

                let set_fns = assign_node_field_list.iter().zip(ref_node_list.iter().zip(field_idents.iter()
                    )).map(
                    |(assign_node_field,(ref_node,field_ident))|{
                        let set_fn_name = format_ident!("set_{}",field_ident);
                        quote! {
                            pub fn #set_fn_name(&mut self,#ref_node) {
                                let ___sym = #assign_node_field;
                                if let #name_inner::#variant_name{ #(#field_idents),*} = &mut self.node.ty{
                                    *#field_ident = ___sym
                                }
                                self.node.sym = #name_counter.next_sym();
                                R::receive(self.to_egglog());
                            }
                        }
                    }
                );
                
                quote! { 
                    impl<R:LetStmtRx> #name_node<R,#variant_name>{
                        #(
                            #set_fns
                        )*
                    }
                } 
            });

            // let match_arm_fields = variants_to_field_ident(&variant);
            let expanded = quote! {
                pub type #name_node_alias<R,S> = #egglog_wrapper_path::wrap::Node<#name_egglogty_impl,R,#name_inner,S>;
                #[derive(derive_more::Deref)]
                pub struct #name_node<R: LetStmtRx,S:EgglogEnumSubTy=()> {
                    node:#name_node_alias<R,S>
                }
                #[derive(Debug,Clone)]
                pub struct #name_egglogty_impl;
                #[derive(Debug,Clone)]
                pub enum #name_inner {
                    #(#variants_def_of_node_with_syms),*
                }
                const _:() = {
                    use std::marker::PhantomData;
                    use #egglog_wrapper_path::*;
                    #(#enum_sub_tys_def)*
                    impl<R:LetStmtRx> #name_node<R,()> {
                        #(#fns)*
                    }
                    impl<R:LetStmtRx, S:EgglogEnumSubTy> ToEgglog for #name_node_alias<R,S>{
                        fn to_egglog(&self) -> String{
                            match &self.ty{
                                #(#match_arms),*
                            }
                        }
                    }
                    impl NodeInner<#name_egglogty_impl> for #name_inner {}
                    static #name_counter: TyCounter<#name_egglogty_impl> = TyCounter::new();
                    #(#set_fns)*
                };
            };
            expanded
        },
        Data::Union(_) => todo!(),
    };

    TokenStream::from(quote!{
        #type_def_expanded 
        #struct_def_expanded 
        }
    )
}
