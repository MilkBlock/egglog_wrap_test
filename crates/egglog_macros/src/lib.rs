// extern crate proc_macro;
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
pub fn egglog_ty(_attr: TokenStream, item:TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    let name_lowercase = format_ident!("{}",name.to_string().to_lowercase());
    let name_egglogty_impl = format_ident!("{}Ty",name);
    let egglog_wrapper_path = egglog_wrapper_path();
    let inventory_path = inventory_wrapper_path();

    // MARK: TYPE_DEF
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
                    const TY_NAME_LOWER:&'static str = stringify!(#name_lowercase);
                    const SORT_DEF: Sort= 
                        Sort(stringify!(
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
                        const TY_NAME_LOWER:&'static str = stringify!(#name_lowercase);
                        const SORT_DEF: Sort= 
                            Sort(stringify!(
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
            // let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            let name_counter = format_ident!("{}_COUNTER",name.to_string().to_uppercase());
            // let derive_more_path  = derive_more_path();
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
                    let _first_generic = format_ident!("{}",first_generic_ident);
                    // postfix_type(&first_generic,"Node",Some("T")
                    quote!(dyn AsRef<#_first_generic<R, ()>>)
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
            // MARK: VEC_DEF
            if is_vec_type(&f.ty){
                let vec_expanded = quote! {
                    pub type #name_node_alias<R,V> = #egglog_wrapper_path::wrap::Node<#name_egglogty_impl,R,#name_inner,V>;

                    #[derive(Clone,Debug)]
                    pub struct #name_egglogty_impl;
                    #[derive(derive_more::Deref)]
                    pub struct #name_node<R: LetStmtRx, V: EgglogEnumVariantTy=()> {
                        node:#name_node_alias<R,V>
                    }
                    #[derive(Clone,Debug)]
                    pub struct #name_inner {
                        v:#egglog_wrapper_path::wrap::Syms<#field_ty>
                    }
                    const _:() = {
                        use #egglog_wrapper_path::wrap::*;
                        impl NodeInner<#name_egglogty_impl> for #name_inner{}
                        use std::marker::PhantomData;
                        static #name_counter: TyCounter<#name_egglogty_impl> = TyCounter::new();
                        impl<R:LetStmtRx> #name_node<R,()> {
                            pub fn new(#field_name:Vec<&#field_node_ty>) -> #name_node<R,()>{
                                let #field_name = #field_name.into_iter().map(|r| r.as_ref().sym).collect();
                                let node = Node{ ty: #name_inner{v:#field_name}, sym: #name_counter.next_sym(),p: PhantomData, s: PhantomData};
                                let node = #name_node {node};
                                R::add_symnode(node.to_symnode());
                                R::receive(node.to_egglog());
                                node
                            }
                            pub fn to_symnode(&self) -> SymbolNode{
                                SymbolNode::new(self.node.sym.detype(), Box::new(self.clone()))
                            }
                        }
                        impl<R:LetStmtRx> EgglogNode for #name_node<R,()> {
                            fn succs_mut(&mut self) -> Vec<&mut Sym>{
                                self.node.ty.v.iter_mut().map(|s| s.detype_mut()).collect()
                            }
                            fn succs(&self) -> Vec<Sym>{
                                self.node.ty.v.iter().map(|s| s.detype()).collect()
                            }
                            fn next_sym(&mut self) -> Sym{
                                let next_sym = #name_counter.next_sym();
                                self.node.sym = next_sym;
                                next_sym.detype()
                            }
                            fn cur_sym(&self) -> Sym{
                                self.node.sym.detype()
                            }
                        }

                        impl<R: LetStmtRx,  V: EgglogEnumVariantTy> AsRef<#name_node<R, ()>> for #name_node<R, V> {
                            fn as_ref(&self) -> &#name_node<R, ()> {
                                unsafe {
                                    &*(self as *const #name_node<R,V> as *const #name_node<R,()>)
                                }

                            }
                        }
                        impl<R:LetStmtRx,V:EgglogEnumVariantTy > Clone for #name_node<R,V> {
                            fn clone(&self) -> Self {
                                Self { node: Node { ty: self.node.ty.clone(), sym: self.node.sym.clone(), p: PhantomData, s: PhantomData }  }
                            }
                        }
                        
                        impl<R:LetStmtRx, V:EgglogEnumVariantTy> ToEgglog for #name_node<R,V>{
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
            // let name_snakecase = format_ident!("{}",name.to_string().to_snake_case());
            // let derive_more_path  = derive_more_path();

            let variants_def_of_node_with_syms = data_enum.variants.iter().map(|variant|{
                let types_and_idents = variants_to_sym_typed_ident_list(variant);
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
                        let node = #name_node {node};
                        R::add_symnode(node.to_symnode());
                        R::receive(node.to_egglog());
                        node
                    }
                } 
            });
            // MARK: ENUM_DEF
            let enum_variant_tys_def = data_enum.variants.iter().map(|variant|{
                let variant_name = &variant.ident;
                
                quote! { 
                    #[derive(Clone)]
                    pub struct #variant_name;
                    impl EgglogEnumVariantTy for #variant_name {
                        const TY_NAME:&'static str = stringify!(#variant_name);
                    }
                }
            });

            let set_fns = data_enum.variants.iter().map(|variant|{
                let ref_node_list = variant_to_ref_node_list(&variant,&name);
                let assign_node_field_list = variants_to_assign_node_field_list_without_prefixed_ident(&variant);
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
                                let old = self.node.sym;
                                self.node.sym = #name_counter.next_sym();
                                R::update_symnode(old.detype(),self.to_symnode());
                            }
                        }
                    }
                );
                let sym_list = variants_to_sym_type_list(variant); 
                let get_sym_fns = sym_list.iter().zip(field_idents.iter()
                    ).map(
                    |(sym,field_ident)|{
                        let get_fn_name = format_ident!("{}_sym",field_ident);
                        quote! {
                            pub fn #get_fn_name(&self) -> #sym{
                                if let #name_inner::#variant_name{ #(#field_idents),*} = &self.node.ty{
                                    #field_ident.clone()
                                }else{
                                    panic!()
                                }
                            }
                        }
                    }
                );
                let get_mut_sym_fns = sym_list.iter().zip(field_idents.iter()
                    ).map(
                    |(sym,field_ident)|{
                        let get_fn_name = format_ident!("{}_sym_mut",field_ident);
                        quote! {
                            pub fn #get_fn_name(&mut self) -> &mut #sym{
                                if let #name_inner::#variant_name{ #(#field_idents),*} = &mut self.node.ty{
                                    #field_ident
                                }else{
                                    panic!()
                                }
                            }
                        }
                    }
                );

                let vec_needed_syms:Vec<_> = 
                    variant_to_field_list_without_prefixed_ident_filter_out_basic_ty(variant)
                    .into_iter()
                    .map(|x| format_ident!("{}",x.to_string())).collect();
                
                quote! { 
                    impl<R:LetStmtRx> #name_node<R,#variant_name>{
                        #(
                            #set_fns
                        )*
                        #(
                            #get_sym_fns
                        )*
                        #(
                            #get_mut_sym_fns
                        )*
                        pub fn to_symnode(&self) -> SymbolNode{
                            SymbolNode::new(self.node.sym.detype(), Box::new(self.clone()))
                        }
                    }
                    impl<R:LetStmtRx> EgglogNode for #name_node<R,#variant_name>{
                        fn succs_mut(&mut self) -> Vec<&mut Sym>{
                            if let #name_inner::#variant_name{ #(#field_idents),*} = &mut self.node.ty{
                                vec![#(#vec_needed_syms.detype_mut()),*]
                            }else{
                                panic!()
                            }
                        }
                        fn succs(&self) -> Vec<Sym>{
                            if let #name_inner::#variant_name{ #(#field_idents),*} = &self.node.ty{
                                vec![#((*#vec_needed_syms).detype()),*]
                            }else{
                                panic!()
                            }
                        }
                        fn next_sym(&mut self) -> Sym{
                            let next_sym = #name_counter.next_sym();
                            self.node.sym = next_sym;
                            next_sym.detype()
                        }
                        fn cur_sym(&self) -> Sym{
                            self.node.sym.detype()
                        }
                    }
                } 
            });

            // let match_arm_fields = variants_to_field_ident(&variant);
            let expanded = quote! {
                pub type #name_node_alias<R,V> = #egglog_wrapper_path::wrap::Node<#name_egglogty_impl,R,#name_inner,V>;
                #[derive(derive_more::Deref)]
                pub struct #name_node<R: LetStmtRx,V:EgglogEnumVariantTy=()> {
                    node:#name_node_alias<R,V>
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
                    #(#enum_variant_tys_def)*
                    impl<R:LetStmtRx> #name_node<R,()> {
                        #(#fns)*
                    }
                    impl<R:LetStmtRx, V:EgglogEnumVariantTy> ToEgglog for #name_node<R,V>{
                        fn to_egglog(&self) -> String{
                            match &self.node.ty{
                                #(#match_arms),*
                            }
                        }
                    }
                    impl<R: LetStmtRx,  V: EgglogEnumVariantTy> AsRef<#name_node<R, ()>> for #name_node<R, V> {
                        fn as_ref(&self) -> &#name_node<R, ()> {
                            unsafe {
                                &*(self as *const #name_node<R,V> as *const #name_node<R,()>)
                            }
                        }
                    }

                    impl<R:LetStmtRx,V:EgglogEnumVariantTy > Clone for #name_node<R,V> {
                        fn clone(&self) -> Self {
                            Self { node: Node { ty: self.ty.clone(), sym: self.sym.clone(), p: PhantomData, s: PhantomData }  }
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

// impl<R: LetStmtRx,  V: EgglogEnumVariantTy> AsRef<#name_node<R, ()>> for #name_node<R, V> {
//     fn as_ref(&self) -> &#name_node_alias<R, ()> {
//         self.node.as_ref()
//     }
// }
