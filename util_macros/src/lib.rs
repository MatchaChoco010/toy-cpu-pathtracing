use proc_macro::TokenStream;
use quote::quote;
use syn::{
    ItemFn, parse_macro_input, {Pat, PatIdent},
};

/// 二項演算子の各種参照の組み合わせの実装を自動生成するマクロ。
///
/// 次のようにして利用する。
/// ```
/// # use util_macros::impl_binary_ops;
/// # use std::marker::PhantomData;
/// # struct A<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct B<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct C<T, U>(PhantomData<T>, PhantomData<U>);
/// #[impl_binary_ops(Add)]
/// fn add<T, U, V>(lhs: &A<T, U>, rhs: &B<U, V>) -> C<T, V> {
///    // ...
/// #   C(PhantomData, PhantomData)
/// }
/// ```
///
/// 上記によって、`A<T, U>`と`B<U, V>`、`&A<T, U>`と`B<U, V>`、
/// `A<T, U>`と`&B<U, V>`、`&A<T, U>`と`&B<U, V>`の組み合わせの全てに二項演算が定義される。
#[proc_macro_attribute]
pub fn impl_binary_ops(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name = parse_macro_input!(attr as syn::Path);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;
    let generics = &func.sig.generics;

    let mut inputs = func.sig.inputs.iter();

    // 左辺と右辺の識別子と型
    let lhs = inputs.next().expect("Expected two arguments");
    let rhs = inputs.next().expect("Expected two arguments");

    let (lhs_ident, lhs_ty) = match lhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("lhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("lhs must be typed argument"),
    };

    let (rhs_ident, rhs_ty) = match rhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("rhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("rhs must be typed argument"),
    };

    let output_ty = match &func.sig.output {
        syn::ReturnType::Type(_, ty) => ty,
        _ => panic!("Expected return type"),
    };

    // &TからTを抽出する
    let extract_inner_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference { elem, .. }) = &**ty {
            elem.to_owned()
        } else {
            panic!("lhs must be &T");
        }
    };
    let lhs_ty = extract_inner_type(lhs_ty);
    let rhs_ty = extract_inner_type(rhs_ty);

    // 本体をself/rhs識別子にバインドして実行
    let replaced_block = quote! {{
        let #lhs_ident = &*self;
        let #rhs_ident = &*rhs;
        #fn_block
    }};

    let expanded = quote! {
        impl #generics std::ops::#trait_name<#rhs_ty> for #lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: #rhs_ty) -> Self::Output {
                (&self).#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for #lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: &#rhs_ty) -> Self::Output {
                (&self).#fn_name(rhs)
            }
        }

        impl #generics std::ops::#trait_name<#rhs_ty> for &#lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: #rhs_ty) -> Self::Output {
                self.#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for &#lhs_ty {
            type Output = #output_ty;
            #[inline(always)]
            fn #fn_name(self, rhs: &#rhs_ty) -> Self::Output #replaced_block
        }
    };

    TokenStream::from(expanded)
}

/// 二項演算子の代入演算子の各種参照の組み合わせの実装を自動生成するマクロ。
///
/// 次のようにして利用する。
/// ```
/// # use util_macros::impl_assign_ops;
/// # use std::marker::PhantomData;
/// # struct A<T, U>(PhantomData<T>, PhantomData<U>);
/// # struct B<T, U>(PhantomData<T>, PhantomData<U>);
/// #[impl_assign_ops(AddAssign)]
/// fn add_assign<T, U, V>(lhs: &mut A<T, U>, rhs: &B<U, V>){
///    // ...
/// }
/// ```
///
/// 上記によって、`&mut A<T, U>`と`B<U, V>`、`&mut A<T, U>`と`&B<U, V>`の組み合わせの両方に
/// 代入演算が定義される。
#[proc_macro_attribute]
pub fn impl_assign_ops(attr: TokenStream, item: TokenStream) -> TokenStream {
    let trait_name = parse_macro_input!(attr as syn::Path);
    let func = parse_macro_input!(item as ItemFn);

    let fn_name = &func.sig.ident;
    let fn_block = &func.block;
    let generics = &func.sig.generics;

    let mut inputs = func.sig.inputs.iter();

    let lhs = inputs.next().expect("Expected two arguments");
    let rhs = inputs.next().expect("Expected two arguments");

    let (lhs_ident, lhs_ty) = match lhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("lhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("lhs must be typed argument"),
    };

    let (rhs_ident, rhs_ty) = match rhs {
        syn::FnArg::Typed(pat) => {
            let ident = match &*pat.pat {
                Pat::Ident(PatIdent { ident, .. }) => ident.clone(),
                _ => panic!("rhs must be identifier"),
            };
            (ident, &pat.ty)
        }
        _ => panic!("rhs must be typed argument"),
    };

    // &mut T の型から T を抽出
    let extract_inner_mut_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference {
            mutability: Some(_),
            elem,
            ..
        }) = &**ty
        {
            elem.to_owned()
        } else {
            panic!("lhs must be &mut T");
        }
    };
    let lhs_ty = extract_inner_mut_type(lhs_ty);

    let extract_inner_ref_type = |ty: &Box<syn::Type>| -> Box<syn::Type> {
        if let syn::Type::Reference(syn::TypeReference { elem, .. }) = &**ty {
            elem.to_owned()
        } else {
            panic!("rhs must be &T");
        }
    };
    let rhs_ty = extract_inner_ref_type(rhs_ty);

    let replaced_block = quote! {{
        let #lhs_ident = &mut *self;
        let #rhs_ident = &*rhs;
        #fn_block
    }};

    let expanded = quote! {
        impl #generics std::ops::#trait_name<#rhs_ty> for #lhs_ty {
            #[inline(always)]
            fn #fn_name(&mut self, rhs: #rhs_ty) {
                self.#fn_name(&rhs)
            }
        }

        impl #generics std::ops::#trait_name<&#rhs_ty> for #lhs_ty {
            #[inline(always)]
            fn #fn_name(&mut self, rhs: &#rhs_ty) #replaced_block
        }

    };

    TokenStream::from(expanded)
}
